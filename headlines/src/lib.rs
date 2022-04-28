mod headlines;

use eframe::{
    egui::{
        CentralPanel, Context, Hyperlink, Label, RichText, ScrollArea, Separator, TextStyle,
        TopBottomPanel, Ui, Visuals,
    },
    epi::App,
};
pub use headlines::{Headlines, Msg, NewsCardData, PADDING};
use newsapi::NewsAPI;

const APP_NAME: &str = "headlines";

impl App for Headlines {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::epi::Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(true);

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
            render_footer(ctx);

            CentralPanel::default().show(ctx, |ui| {
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("Loading ⌛");
                    });
                } else {
                    render_header(ui);
                    ScrollArea::vertical().show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        eframe::epi::set_value(storage, "headlines", &self.config);
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

fn render_footer(ctx: &Context) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new(
                RichText::new("API source: newsapi.org")
                    .small()
                    .text_style(TextStyle::Monospace),
            ));
            ui.add(Hyperlink::from_label_and_url(
                "Made with egui",
                "https://github.com/emilk/egui",
            ));
            ui.add(Hyperlink::from_label_and_url(
                "creativcoder/headlines",
                "https://github.com/creativcoder/headlines",
            ));
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
    eframe::start_web(canvas_id, Box::new(|cc| Box::new(headlines.init(cc))))
        .expect("Failed to launch app");
}
