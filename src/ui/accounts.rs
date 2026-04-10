use eframe::egui;
use crate::app::App;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("帐号列表");
    ui.separator();

    if ui.button("刷新").clicked() {
        app.refresh_all_accounts();
    }

    ui.add_space(4.0);

    let accounts = app.cached_all_accounts.clone();
    if accounts.is_empty() {
        ui.label("暂无帐号，请在启动项中添加");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("all_accounts_readonly")
            .num_columns(6)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("所属启动项");
                ui.strong("平台");
                ui.strong("用户名");
                ui.strong("密码");
                ui.strong("TOTP");
                ui.strong("创建时间");
                ui.end_row();

                for a in &accounts {
                    // Find profile name
                    let profile_name = app.cached_profiles.iter()
                        .find(|p| p.id == a.profile_id)
                        .map(|p| format!("[#{}] {}", p.id, p.name))
                        .unwrap_or_else(|| format!("#{}", a.profile_id));
                    ui.label(profile_name);

                    ui.label(app.platform_name(a.platform_id));
                    ui.label(&a.username);

                    // Password with toggle
                    let show = app.show_password.contains(&a.id);
                    ui.horizontal(|ui| {
                        if show {
                            ui.label(&a.password);
                        } else {
                            ui.label("••••••");
                        }
                        if ui.small_button(if show { "隐藏" } else { "显示" }).clicked() {
                            if show {
                                app.show_password.remove(&a.id);
                            } else {
                                app.show_password.insert(a.id);
                            }
                        }
                    });

                    // TOTP
                    if let Some(secret) = &a.totp_secret {
                        if !secret.is_empty() {
                            if let Some(code) = App::generate_totp(secret) {
                                ui.label(code);
                            } else {
                                ui.colored_label(egui::Color32::RED, "密钥无效");
                            }
                        } else {
                            ui.label("-");
                        }
                    } else {
                        ui.label("-");
                    }

                    ui.label(&a.created_at);
                    ui.end_row();
                }
            });
    });

    // Request repaint every second for TOTP refresh
    ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
}
