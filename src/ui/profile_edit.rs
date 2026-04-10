use eframe::egui;
use crate::app::{App, EditAccount};
use crate::models::ParamValueType;

pub fn show(app: &mut App, ctx: &egui::Context) {
    if !app.profile_edit_open {
        return;
    }

    let title = if let Some(id) = app.editing_profile_id {
        format!("编辑启动项 (ID: {})", id)
    } else {
        "新建启动项".to_string()
    };

    let mut open = app.profile_edit_open;
    egui::Window::new(title)
        .open(&mut open)
        .resizable(true)
        .default_width(700.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // === Profile name ===
                ui.horizontal(|ui| {
                    ui.label("用户画像名称:");
                    ui.add(egui::TextEdit::singleline(&mut app.edit_profile_name).desired_width(300.0));
                });

                // === Browser config ===
                ui.heading("浏览器配置");
                ui.separator();

                let browsers = app.cached_browsers.clone();
                let all_params = app.cached_browser_params.clone();

                if browsers.is_empty() {
                    ui.label("请先在系统设置中添加浏览器");
                } else {
                    for b in &browsers {
                        let mut selected = app.edit_selected_browsers.contains(&b.id);
                        if ui.checkbox(&mut selected, format!("{} [{}]", &b.name, &b.english_name)).changed() {
                            if selected {
                                app.edit_selected_browsers.insert(b.id);
                            } else {
                                app.edit_selected_browsers.remove(&b.id);
                            }
                        }

                        if selected {
                            let params = all_params.get(&b.id).cloned().unwrap_or_default();
                            if !params.is_empty() {
                                ui.indent(format!("bp_{}", b.id), |ui| {
                                    for p in &params {
                                        ui.horizontal(|ui| {
                                            let mut enabled = app.edit_param_enabled.contains(&p.id);
                                            if ui.checkbox(&mut enabled, "").changed() {
                                                if enabled {
                                                    app.edit_param_enabled.insert(p.id);
                                                } else {
                                                    app.edit_param_enabled.remove(&p.id);
                                                }
                                            }

                                            let resp = ui.label(&p.param_name);
                                            if !p.description.is_empty() {
                                                resp.on_hover_text(&p.description);
                                            }

                                            if p.value_type == ParamValueType::None {
                                                ui.label(format!("({})", p.value_type.label()));
                                            } else if enabled {
                                                let val = app.edit_param_values
                                                    .entry((b.id, p.id))
                                                    .or_insert_with(String::new);
                                                ui.add(egui::TextEdit::singleline(val).desired_width(200.0));
                                                ui.weak(format!("({})", p.value_type.label()));
                                            } else {
                                                ui.label("-");
                                            }
                                        });
                                    }
                                });
                            }
                        }
                        ui.add_space(2.0);
                    }
                }

                // === Account management ===
                ui.separator();
                ui.heading("帐号管理");

                let platforms = app.cached_platforms.clone();
                if platforms.is_empty() {
                    ui.label("请先在系统设置中添加平台");
                } else {
                    let mut to_remove = Vec::new();
                    for (idx, ea) in app.edit_accounts.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Multi-select platform dropdown
                                ui.label("平台:");
                                let selected_names: Vec<&str> = platforms.iter()
                                    .filter(|p| ea.platform_ids.contains(&p.id))
                                    .map(|p| p.name.as_str())
                                    .collect();
                                let display = if selected_names.is_empty() {
                                    "选择平台".to_string()
                                } else {
                                    selected_names.join(", ")
                                };
                                egui::ComboBox::new(format!("acc_plat_{}", idx), "")
                                    .selected_text(&display)
                                    .show_ui(ui, |ui| {
                                        for p in &platforms {
                                            let mut checked = ea.platform_ids.contains(&p.id);
                                            if ui.checkbox(&mut checked, &p.name).changed() {
                                                if checked {
                                                    ea.platform_ids.insert(p.id);
                                                } else {
                                                    ea.platform_ids.remove(&p.id);
                                                }
                                            }
                                        }
                                    });

                                ui.label("用户名:");
                                ui.add(egui::TextEdit::singleline(&mut ea.username).desired_width(100.0));

                                ui.label("密码:");
                                ui.add(egui::TextEdit::singleline(&mut ea.password).desired_width(100.0).password(!ea.show_password));
                                if ui.small_button(if ea.show_password { "隐藏" } else { "显示" }).clicked() {
                                    ea.show_password = !ea.show_password;
                                }
                            });
                            if ui.button("删除帐号").clicked() {
                                to_remove.push(idx);
                            }
                        });
                    }
                    for idx in to_remove.into_iter().rev() {
                        app.edit_accounts.remove(idx);
                    }

                    ui.add_space(4.0);
                    if ui.button("+ 添加帐号").clicked() {
                        app.edit_accounts.push(EditAccount {
                            id: None,
                            platform_ids: std::collections::HashSet::new(),
                            username: String::new(),
                            password: String::new(),
                            show_password: false,
                        });
                    }
                }

                // === Validation ===
                ui.separator();

                let mut errors = Vec::new();
                if app.edit_profile_name.trim().is_empty() {
                    errors.push("用户画像名称不能为空");
                } else {
                    let conn = app.conn.lock().unwrap();
                    if crate::db::profile::profile_name_exists(&conn, &app.edit_profile_name, app.editing_profile_id) {
                        errors.push("用户画像名称已存在");
                    }
                }
                if app.edit_selected_browsers.is_empty() {
                    errors.push("至少选择一个浏览器");
                }
                for ea in &app.edit_accounts {
                    if ea.platform_ids.is_empty() {
                        errors.push("每个帐号至少选择一个平台");
                        break;
                    }
                    if ea.username.trim().is_empty() || ea.password.trim().is_empty() {
                        errors.push("帐号的用户名和密码不能为空");
                        break;
                    }
                }
                // Check enabled params have values
                for &bid in &app.edit_selected_browsers {
                    if let Some(params) = all_params.get(&bid) {
                        for p in params {
                            if app.edit_param_enabled.contains(&p.id) && p.value_type != ParamValueType::None {
                                if let Some(val) = app.edit_param_values.get(&(bid, p.id)) {
                                    if val.trim().is_empty() {
                                        errors.push("已启用的参数值不能为空");
                                        break;
                                    }
                                } else {
                                    errors.push("已启用的参数值不能为空");
                                    break;
                                }
                            }
                        }
                    }
                }

                if !errors.is_empty() {
                    for e in &errors {
                        ui.colored_label(egui::Color32::RED, *e);
                    }
                }

                ui.horizontal(|ui| {
                    if ui.add_enabled(errors.is_empty(), egui::Button::new("保存")).clicked() {
                        app.save_profile();
                    }
                    if ui.button("取消").clicked() {
                        app.profile_edit_open = false;
                    }
                });
            });
        });

    app.profile_edit_open = open;
}
