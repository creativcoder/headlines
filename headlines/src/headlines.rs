use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};

#[cfg(not(target_arch = "wasm32"))]
use crate::fetch_news;
use crate::APP_NAME;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[cfg(target_arch = "wasm32")]
use crate::fetch_web;
use eframe::{
    egui::{
        self, Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily,
        Hyperlink, Label, Layout, RichText, Separator, TextStyle, TopBottomPanel, Ui, Window,
    },
    CreationContext,
};

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub enum Msg {
    ApiKeySet(String),
    Refresh,
}

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}

#[derive(Default)]
pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub toggle_config: bool,
    pub toggle_about: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>,
}

pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String,
}

impl Headlines {
    pub fn init(mut self, cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            self.config = eframe::get_value(storage, APP_NAME).unwrap_or_default();
            self.api_key_initialized = !self.config.api_key.is_empty();
        }

        let api_key = self.config.api_key.to_string();
        let (app_tx, app_rx) = sync_channel(1);
        self.app_tx = Some(app_tx);
        #[allow(unused_mut)]
        let (mut news_tx, news_rx) = channel();
        self.news_rx = Some(news_rx);
        self.toggle_config = false;

        #[cfg(not(target_arch = "wasm32"))]
        {
            thread::spawn(move || {
                if !api_key.is_empty() {
                    fetch_news(&api_key, &mut news_tx);
                }

                loop {
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => {
                            tracing::info!("api key set recvd");
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Ok(Msg::Refresh) => {
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Err(_) => continue,
                    }
                }
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            let api_key_web = api_key.clone();
            let news_tx_web = news_tx.clone();
            gloo_timers::callback::Timeout::new(10, move || {
                wasm_bindgen_futures::spawn_local(async {
                    fetch_web(api_key_web, news_tx_web).await;
                });
            })
            .forget();

            gloo_timers::callback::Interval::new(2000, move || match app_rx.try_recv() {
                Ok(Msg::ApiKeySet(api_key)) => {
                    wasm_bindgen_futures::spawn_local(fetch_web(api_key.clone(), news_tx.clone()));
                }
                Ok(Msg::Refresh) => {
                    let api_key = api_key.clone();
                    wasm_bindgen_futures::spawn_local(fetch_web(api_key, news_tx.clone()));
                }
                Err(e) => {
                    tracing::warn!("failed receiving msg: {}", e);
                }
            })
            .forget();
        }

        self.configure_fonts(&cc.egui_ctx);

        self
    }

    pub fn configure_fonts(&self, ctx: &Context) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            FontData::from_static(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );

        font_def
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());

        ctx.set_fonts(font_def);
    }

    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.articles {
            ui.add_space(PADDING);
            // render title
            let title = format!("▶ {}", a.title);
            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }
            // render desc
            ui.add_space(PADDING);
            let desc =
                Label::new(RichText::new(&a.desc).text_style(eframe::egui::TextStyle::Button));
            ui.add(desc);

            // render hyperlinks
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }
            ui.add_space(PADDING);
            ui.with_layout(
                Layout::right_to_left(eframe::emath::Align::Min)
                    .with_cross_align(eframe::emath::Align::Min),
                |ui| {
                    ui.add(Hyperlink::from_label_and_url("read more ⤴", &a.url));
                },
            );
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    #[allow(unused)]
    pub(crate) fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(eframe::emath::Align::Min), |ui| {
                    ui.add(Label::new(
                        RichText::new("📰").text_style(egui::TextStyle::Heading),
                    ));
                });
                // controls
                ui.with_layout(Layout::right_to_left(eframe::emath::Align::Min), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let close_btn = ui.add(Button::new(
                            RichText::new("❌").text_style(egui::TextStyle::Body),
                        ));

                        if close_btn.clicked() {
                            frame.close();
                        }
                    }

                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄").text_style(egui::TextStyle::Body),
                    ));
                    if refresh_btn.clicked() {
                        self.articles.clear();
                        if let Some(tx) = &self.app_tx {
                            tx.send(Msg::Refresh)
                                .expect("Failed sending refresh event.");
                        }
                    }
                    // theme button
                    let theme_btn = ui.add(Button::new(
                        RichText::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(egui::TextStyle::Body),
                    ));
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }

                    // config button
                    let config_btn = ui.add(Button::new(
                        RichText::new("🛠").text_style(egui::TextStyle::Body),
                    ));

                    if config_btn.clicked() {
                        self.toggle_config = !self.toggle_config;
                    }

                    // about button
                    let about_btn =
                        ui.add(Button::new(RichText::new("ℹ").text_style(TextStyle::Body)));
                    if about_btn.clicked() {
                        self.toggle_about = !self.toggle_about;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            if let Ok(news_data) = rx.try_recv() {
                self.articles.push(news_data);
            }
        }
    }

    pub fn render_config(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |_| {
            let Headlines {
                toggle_config,
                config,
                app_tx,
                api_key_initialized,
                ..
            } = self;
            Window::new("App configuration")
                .open(toggle_config)
                .show(ctx, |ui| {
                    ui.label("Enter you API_KEY for newsapi.org");
                    let text_input = ui.text_edit_singleline(&mut config.api_key);
                    if text_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        if let Some(tx) = &app_tx {
                            tx.send(Msg::ApiKeySet(config.api_key.to_string()))
                                .expect("Failed sending ApiKeySet event");
                        }
                        *api_key_initialized = true;
                        tracing::info!("API_KEY set");
                        ui.close_menu();
                    }
                    ui.label("Don't have the API_KEY? register at:");
                    ui.hyperlink("https://newsapi.org/register");
                });
        });
    }

    pub fn render_about(&mut self, ctx: &Context) {
        let window = Window::new("About headlines").open(&mut self.toggle_about);
        window.show(ctx, |ui| {
            let info = Label::new("A simple news reading app that runs on all platforms.");
            ui.add(info);
        });
    }

    pub(crate) fn render_footer(&self, ctx: &Context) {
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

    pub(crate) fn render_header(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("headlines");
        });
        ui.add_space(PADDING);
        let sep = Separator::default().spacing(20.);
        ui.add(sep);
    }
}
