use eframe::egui;
use crate::app::App;

pub fn show(app: &mut App, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Fingerprint Launcher");
            ui.separator();
            if ui.selectable_label(app.current_tab == Tab::Profiles, "启动项").clicked() {
                app.current_tab = Tab::Profiles;
            }
            if ui.selectable_label(app.current_tab == Tab::Settings, "系统设置").clicked() {
                app.current_tab = Tab::Settings;
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        match app.current_tab {
            Tab::Profiles => show_profiles(app, ui),
            Tab::Settings => super::settings::show(app, ui),
        }
    });

    super::profile_edit::show(app, ctx);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Profiles,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Profiles
    }
}

fn show_profiles(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("+ 新建启动项").clicked() {
            app.open_new_profile();
        }
        ui.separator();
        ui.label("搜索:");
        ui.text_edit_singleline(&mut app.search_query);
        if ui.button("搜索").clicked() {
            app.refresh_profiles();
        }
    });

    ui.separator();

    let profiles = app.cached_profiles.clone();
    let browsers = app.cached_browsers.clone();

    if profiles.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("暂无启动项，点击上方按钮创建");
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for profile in &profiles {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(format!("[#{}] {}", profile.id, &profile.name));
                            // Show account summary
                            let accs = app.profile_accounts_cache.get(&profile.id);
                            if let Some(accs) = accs {
                                for a in accs {
                                    let plat_names = app.get_account_platform_names(a.id);
                                    let plat_str = if plat_names.is_empty() { "无平台".to_string() } else { plat_names.join(",") };
                                    ui.label(format!("{} [{}]", &a.username, plat_str));
                                }
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("删除").clicked() {
                                app.delete_profile(profile.id);
                            }
                            if ui.button("编辑").clicked() {
                                app.open_edit_profile(profile.id);
                            }

                            let profile_browsers = app.get_profile_browsers(profile.id);
                            for b in &browsers {
                                let has_config = profile_browsers.iter().any(|pb| pb.browser_id == b.id);

                                let tex = app.browser_icon_textures.get(&b.id)
                                    .or(app.default_browser_icon.as_ref());

                                let resp = if let Some(tex) = tex {
                                    let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(28.0, 28.0)));
                                    ui.add_enabled(has_config, egui::ImageButton::new(img))
                                } else {
                                    ui.add_enabled(has_config, egui::Button::new(&b.name))
                                };

                                let resp = resp.on_hover_text(if has_config {
                                    &b.name
                                } else {
                                    "未配置"
                                });

                                if resp.clicked() {
                                    if let Some(pb) = profile_browsers.iter().find(|pb| pb.browser_id == b.id) {
                                        app.launch(b, &pb.launch_args);
                                    }
                                }
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            }
        });
    }

    // Pagination
    ui.separator();
    ui.horizontal(|ui| {
        let total = app.total_profiles;
        let total_pages = (total + app.page_size - 1) / app.page_size.max(1);
        let total_pages = total_pages.max(1);

        if ui.add_enabled(app.current_page > 1, egui::Button::new("< 上一页")).clicked() {
            app.current_page -= 1;
            app.refresh_profiles();
        }
        ui.label(format!("第 {} / {} 页 (共 {} 条)", app.current_page, total_pages, total));
        if ui.add_enabled(app.current_page < total_pages, egui::Button::new("下一页 >")).clicked() {
            app.current_page += 1;
            app.refresh_profiles();
        }
    });
}
