use eframe::egui;
use crate::app::App;
use crate::models::{Browser, ParamValueType};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        show_browser_management(app, ui);
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        show_platform_management(app, ui);
    });
}

fn show_browser_management(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("浏览器管理");
    ui.separator();

    // Add browser form
    egui::Grid::new("add_browser_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("名称:");
            ui.text_edit_singleline(&mut app.new_browser_name);
            ui.end_row();

            ui.label("英文名:");
            ui.text_edit_singleline(&mut app.new_browser_english_name);
            ui.end_row();

            ui.label("执行路径:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.new_browser_path);
                if ui.button("浏览...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        app.new_browser_path = path.display().to_string();
                    }
                }
            });
            ui.end_row();

            ui.label("图标(可选):");
            ui.horizontal(|ui| {
                let icon_text = app.new_browser_icon_path.clone().unwrap_or_default();
                ui.label(if icon_text.is_empty() { "未选择" } else { &icon_text });
                if ui.button("选择图标").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("图片", &["png", "jpg", "ico", "svg"])
                        .pick_file()
                    {
                        app.new_browser_icon_path = Some(path.display().to_string());
                    }
                }
                if app.new_browser_icon_path.is_some() && ui.button("清除").clicked() {
                    app.new_browser_icon_path = None;
                }
            });
            ui.end_row();
        });

    // Validation
    let name_ok = !app.new_browser_name.trim().is_empty();
    let ename_ok = !app.new_browser_english_name.trim().is_empty();
    let path_ok = !app.new_browser_path.trim().is_empty();
    let ename_dup = {
        let conn = app.conn.lock().unwrap();
        crate::db::browser::english_name_exists(&conn, app.new_browser_english_name.trim())
    };

    if ename_dup && ename_ok {
        ui.colored_label(egui::Color32::RED, "英文名已存在，请使用其他名称");
    }

    if ui.add_enabled(name_ok && ename_ok && path_ok && !ename_dup, egui::Button::new("添加浏览器")).clicked() {
        app.add_browser();
    }

    ui.separator();

    // Browser list
    let browsers = app.cached_browsers.clone();
    let all_params = app.cached_browser_params.clone();

    if browsers.is_empty() {
        ui.label("暂无浏览器配置");
        return;
    }

    for b in &browsers {
        ui.horizontal(|ui| {
            // Show browser icon
            let tex = app.browser_icon_textures.get(&b.id)
                .or(app.default_browser_icon.as_ref());
            if let Some(tex) = tex {
                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                    tex.id(), egui::vec2(20.0, 20.0),
                )));
            }

            egui::CollapsingHeader::new(format!("{} [{}]", &b.name, &b.english_name))
                .id_salt(format!("browser_{}", b.id))
                .default_open(false)
                .show(ui, |ui| {
                    let is_editing = app.editing_browser_id == Some(b.id);

                    if is_editing {
                        show_browser_edit_form(app, ui, b);
                    } else {
                        ui.label(format!("路径: {}", &b.exe_path));
                        if let Some(icon) = &b.icon_path {
                            ui.label(format!("图标: {}", icon));
                        }
                        if ui.button("编辑").clicked() {
                            app.open_edit_browser(b.id);
                        }
                    }

                ui.add_space(4.0);
                ui.strong("启动参数列表:");

                let params = all_params.get(&b.id).cloned().unwrap_or_default();
                if !params.is_empty() {
                    egui::Grid::new(format!("params_{}", b.id))
                        .num_columns(4)
                        .spacing([8.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("参数名");
                            ui.strong("值类型");
                            ui.strong("说明");
                            ui.strong("操作");
                            ui.end_row();

                            for p in &params {
                                ui.label(&p.param_name);
                                ui.label(p.value_type.label());
                                ui.label(&p.description);
                                if ui.button("删除").clicked() {
                                    app.delete_browser_param(p.id);
                                }
                                ui.end_row();
                            }
                        });
                } else {
                    ui.label("暂无启动参数");
                }

                ui.add_space(4.0);

                // Add param form
                let mut local_fields = app
                    .new_param_fields
                    .remove(&b.id)
                    .unwrap_or_else(|| (String::new(), 1, String::new()));

                let mut should_add = false;
                ui.horizontal(|ui| {
                    ui.label("参数:");
                    ui.add(egui::TextEdit::singleline(&mut local_fields.0).desired_width(120.0));
                    ui.label("类型:");
                    egui::ComboBox::new(format!("vt_combo_{}", b.id), "")
                        .selected_text(ParamValueType::all()[local_fields.1].label())
                        .show_ui(ui, |ui| {
                            for (i, vt) in ParamValueType::all().iter().enumerate() {
                                ui.selectable_value(&mut local_fields.1, i, vt.label());
                            }
                        });
                    ui.label("说明:");
                    ui.add(egui::TextEdit::singleline(&mut local_fields.2).desired_width(150.0));
                    if ui.button("添加参数").clicked() && !local_fields.0.trim().is_empty() {
                        should_add = true;
                    }
                });

                app.new_param_fields.insert(b.id, local_fields);
                if should_add {
                    app.add_browser_param(b.id);
                }

                ui.add_space(4.0);

                // Delete browser
                let can_delete_directly = app.can_delete_browser_directly(b.id);
                if can_delete_directly {
                    if ui.button("删除浏览器").clicked() {
                        app.delete_browser(b.id);
                    }
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "该浏览器已被启动项引用，删除请输入浏览器名称确认:");
                    let mut confirm_text = app.browser_delete_confirm.remove(&b.id).unwrap_or_default();
                    let mut should_delete = false;
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut confirm_text);
                        let matches = confirm_text.trim() == b.name;
                        if ui.add_enabled(matches, egui::Button::new("确认删除")).clicked() {
                            should_delete = true;
                        }
                    });
                    if should_delete {
                        app.delete_browser(b.id);
                    } else {
                        app.browser_delete_confirm.insert(b.id, confirm_text);
                    }
                }
            });
        }); // end horizontal

        ui.add_space(4.0);
    }
}

fn show_platform_management(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("平台管理");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("平台名称:");
        ui.text_edit_singleline(&mut app.new_platform_name);
        if ui.button("添加平台").clicked() && !app.new_platform_name.trim().is_empty() {
            app.add_platform();
        }
    });

    ui.add_space(4.0);

    let platforms = app.cached_platforms.clone();
    if platforms.is_empty() {
        ui.label("暂无平台");
    } else {
        egui::Grid::new("platform_list")
            .num_columns(3)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("名称");
                ui.strong("创建时间");
                ui.strong("操作");
                ui.end_row();

                for p in &platforms {
                    ui.label(&p.name);
                    ui.label(&p.created_at);
                    if ui.button("删除").clicked() {
                        app.delete_platform(p.id);
                    }
                    ui.end_row();
                }
            });
    }
}

fn show_browser_edit_form(app: &mut App, ui: &mut egui::Ui, browser: &Browser) {
    egui::Grid::new(format!("edit_browser_{}", browser.id))
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("名称:");
            ui.text_edit_singleline(&mut app.edit_browser_name);
            ui.end_row();

            ui.label("英文名:");
            ui.text_edit_singleline(&mut app.edit_browser_english_name);
            ui.end_row();

            ui.label("执行路径:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.edit_browser_path);
                if ui.button("浏览...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        app.edit_browser_path = path.display().to_string();
                    }
                }
            });
            ui.end_row();

            ui.label("图标:");
            ui.horizontal(|ui| {
                let icon_display = app.edit_browser_new_icon_src.as_ref()
                    .or(app.edit_browser_icon_path.as_ref())
                    .map(|s| s.clone())
                    .unwrap_or_default();
                ui.label(if icon_display.is_empty() { "未设置" } else { &icon_display });
                if ui.button("选择图标").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("图片", &["png", "jpg", "ico", "svg"])
                        .pick_file()
                    {
                        app.edit_browser_new_icon_src = Some(path.display().to_string());
                    }
                }
                if (app.edit_browser_icon_path.is_some() || app.edit_browser_new_icon_src.is_some())
                    && ui.button("清除").clicked()
                {
                    app.edit_browser_icon_path = None;
                    app.edit_browser_new_icon_src = None;
                }
            });
            ui.end_row();
        });

    // Validation
    let name_ok = !app.edit_browser_name.trim().is_empty();
    let ename_ok = !app.edit_browser_english_name.trim().is_empty();
    let path_ok = !app.edit_browser_path.trim().is_empty();
    let ename_dup = {
        let conn = app.conn.lock().unwrap();
        crate::db::browser::english_name_exists_exclude(&conn, app.edit_browser_english_name.trim(), browser.id)
    };

    if ename_dup && ename_ok {
        ui.colored_label(egui::Color32::RED, "英文名已存在");
    }

    ui.horizontal(|ui| {
        if ui.add_enabled(name_ok && ename_ok && path_ok && !ename_dup, egui::Button::new("保存")).clicked() {
            app.save_edit_browser();
        }
        if ui.button("取消").clicked() {
            app.cancel_edit_browser();
        }
    });
}
