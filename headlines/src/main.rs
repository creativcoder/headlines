use eframe::{egui::Vec2, run_native, NativeOptions};
use headlines::Headlines;

fn main() {
    tracing_subscriber::fmt::init();

    let app = Headlines::new();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(540., 960.));
    run_native(Box::new(app), win_option);
}
