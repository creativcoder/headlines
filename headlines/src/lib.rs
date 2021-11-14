mod headlines;

use std::{
    sync::mpsc::{channel, sync_channel},
    thread,
};

use eframe::{
    egui::{
        CentralPanel, CtxRef, Hyperlink, Label, Rgba, ScrollArea, Separator, TextStyle,
        TopBottomPanel, Ui, Vec2, Visuals,
    },
    epi::App,
};
pub use headlines::{Headlines, Msg, NewsCardData, PADDING};
use newsapi::NewsAPI;

impl App for Headlines {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        storage: Option<&dyn eframe::epi::Storage>,
    ) {
        if let Some(storage) = storage {
            self.config = eframe::epi::get_value(storage, "headlines").unwrap_or_default();
            self.api_key_initialized = !self.config.api_key.is_empty();
        }

        let api_key = self.config.api_key.to_string();

        let (mut news_tx, news_rx) = channel();
        let (app_tx, app_rx) = sync_channel(1);

        self.app_tx = Some(app_tx);

        self.news_rx = Some(news_rx);

        let api_key_web = api_key.clone();
        let news_tx_web = news_tx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx);
            } else {
                loop {
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => {
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Ok(Msg::Refresh) => {
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Err(e) => {
                            tracing::error!("failed receiving msg: {}", e);
                        }
                    }
                }
            }
        });

        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Timeout::new(10, move || {
            wasm_bindgen_futures::spawn_local(async {
                fetch_web(api_key_web, news_tx_web).await;
            });
        })
        .forget();

        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Interval::new(500, move || match app_rx.try_recv() {
            Ok(Msg::ApiKeySet(api_key)) => {
                wasm_bindgen_futures::spawn_local(fetch_web(api_key.clone(), news_tx.clone()));
            }
            Ok(Msg::Refresh) => {
                let api_key = api_key.clone();
                wasm_bindgen_futures::spawn_local(fetch_web(api_key, news_tx.clone()));
            }
            Err(e) => {
                tracing::error!("failed receiving msg: {}", e);
            }
        })
        .forget();

        self.configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        ctx.request_repaint();

        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }

        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.preload_articles();

            self.render_top_panel(ctx, frame);
            CentralPanel::default().show(ctx, |ui| {
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("Loading ⌛");
                    });
                } else {
                    render_header(ui);
                    ScrollArea::auto_sized().show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                    render_footer(ctx);
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        eframe::epi::set_value(storage, "headlines", &self.config);
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn fetch_news(api_key: &str, news_tx: &mut std::sync::mpsc::Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch() {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            }
        }
    } else {
        tracing::error!("failed fetching news");
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_web(api_key: String, news_tx: std::sync::mpsc::Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch_web().await {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            }
        }
    } else {
        tracing::error!("failed fetching news");
    }
}

fn render_footer(ctx: &CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API source: newsapi.org").monospace());
            ui.add(
                Hyperlink::new("https://github.com/emilk/egui")
                    .text("Made with egui")
                    .text_style(TextStyle::Monospace),
            );
            ui.add(
                Hyperlink::new("https://github.com/creativcoder/headlines")
                    .text("creativcoder/headlines")
                    .text_style(TextStyle::Monospace),
            );
            ui.add_space(10.);
        })
    });
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("headlines");
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let headlines = Headlines::new();
    tracing_wasm::set_as_global_default();
    eframe::start_web(canvas_id, Box::new(headlines));
}
