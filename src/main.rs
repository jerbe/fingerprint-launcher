mod app;
mod db;
mod icon;
mod launcher;
mod models;
mod ui;

fn main() -> eframe::Result<()> {
    let db_path = std::env::current_dir()
        .unwrap_or_default()
        .join("fingerprint_launcher.db");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 700.0])
            .with_title("Fingerprint Launcher"),
        ..Default::default()
    };

    eframe::run_native(
        "Fingerprint Launcher",
        options,
        Box::new(move |cc| {
            let mut fonts = eframe::egui::FontDefinitions::default();
            let font_paths = [
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "C:\\Windows\\Fonts\\msyh.ttc",
                "/System/Library/Fonts/PingFang.ttc",
            ];
            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        eframe::egui::FontData::from_owned(data).into(),
                    );
                    fonts
                        .families
                        .entry(eframe::egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese_font".to_owned());
                    fonts
                        .families
                        .entry(eframe::egui::FontFamily::Monospace)
                        .or_default()
                        .push("chinese_font".to_owned());
                    break;
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(app::App::new(db_path)))
        }),
    )
}
