mod headlines;

use eframe::{
    egui::{CentralPanel, ScrollArea, Spinner, Visuals},
    App,
};
pub use headlines::{Headlines, Msg, NewsCardData, PADDING};
use newsapi::NewsAPI;

const APP_NAME: &str = "headlines";

impl App for Headlines {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);

        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }

        if self.toggle_about {
            self.render_about(ctx);
            return;
        }

        if self.toggle_config {
            self.render_config(ctx);
        } else {
            self.preload_articles();
            self.render_top_panel(ctx, frame);
            self.render_footer(ctx);
            CentralPanel::default().show(ctx, |ui| {
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| ui.add(Spinner::new()));
                } else {
                    self.render_header(ui);
                    ScrollArea::vertical().show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_NAME, &self.config);
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

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let headlines = Headlines::default();
    tracing_wasm::set_as_global_default();
    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        canvas_id,
        web_options,
        Box::new(|cc| Box::new(headlines.init(cc))),
    )
    .expect("Failed to launch headlines");
}
