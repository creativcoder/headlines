use eframe::{egui::Vec2, run_native, NativeOptions};
use headlines::Headlines;

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    let headlines = Headlines::default();
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(540., 960.));
    run_native(
        "Headlines",
        win_option,
        Box::new(|cc| Box::new(headlines.init(cc))),
    );
}
