mod headlines;

use eframe::{NativeOptions, egui::{CentralPanel, CtxRef, Hyperlink, Label, ScrollArea, Separator, TextStyle, TopBottomPanel, Ui, Vec2, Visuals}, epi::App, run_native};
use headlines::{Headlines, PADDING};

fn fetch_news() {
    
}

impl App for Headlines {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {

        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
        
        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.render_top_panel(ctx, frame);
            CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::auto_sized().show(ui, |ui| {
                    self.render_news_cards(ui);
                });
                render_footer(ctx);
            });
        }
    }

    fn name(&self) -> &str {
        "Headlines"
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

fn main() {
    tracing_subscriber::fmt::init();

    let app = Headlines::new();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(540., 960.));
    run_native(Box::new(app), win_option);
}
