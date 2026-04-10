use eframe::egui;

const ICON_SIZE: u32 = 32;

/// Load an icon from a file path (supports PNG, JPG, ICO, SVG)
pub fn load_from_path(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "svg" => load_svg(ctx, path),
        _ => load_raster(ctx, path),
    }
}

/// Try to extract an icon from an executable file
pub fn extract_from_exe(ctx: &egui::Context, exe_path: &str) -> Option<egui::TextureHandle> {
    let path = std::path::Path::new(exe_path);

    // Try macOS .app bundle
    if let Some(icon) = extract_macos(ctx, path) {
        return Some(icon);
    }

    // Try Windows PE resources
    if let Some(icon) = extract_windows(ctx, path) {
        return Some(icon);
    }

    // Fallback: find icon files near the exe
    find_near_exe(ctx, path)
}

/// Create a default icon (rounded gray square)
pub fn create_default(ctx: &egui::Context) -> egui::TextureHandle {
    let size = ICON_SIZE as usize;
    let mut rgba = Vec::with_capacity(size * size * 4);
    let half = size as f32 / 2.0 - 2.0;
    let cr = 6.0f32;
    for y in 0..size {
        for x in 0..size {
            let cx = x as f32 - size as f32 / 2.0;
            let cy = y as f32 - size as f32 / 2.0;
            let in_body = cx.abs() <= half - cr || cy.abs() <= half - cr;
            let in_corner = {
                let dx = cx.abs() - (half - cr);
                let dy = cy.abs() - (half - cr);
                dx * dx + dy * dy <= cr * cr
            };
            let inside = cx.abs() <= half && cy.abs() <= half && (in_body || in_corner);
            if inside {
                rgba.extend_from_slice(&[110, 115, 140, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    let image = egui::ColorImage::from_rgba_unmultiplied([size, size], &rgba);
    ctx.load_texture("default_browser_icon", image, egui::TextureOptions::LINEAR)
}

// ---- Internal helpers ----

fn load_raster(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    let img = image::open(path)
        .ok()?
        .resize_to_fill(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    Some(make_texture_from_image(
        ctx,
        &format!("icon_{}", path.replace(|c: char| !c.is_alphanumeric(), "_")),
        &img,
    ))
}

fn load_svg(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    let data = std::fs::read(path).ok()?;
    let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).ok()?;
    let svg_size = tree.size();
    let scale_x = ICON_SIZE as f32 / svg_size.width();
    let scale_y = ICON_SIZE as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);
    let mut pixmap = tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE)?;
    let mut pm = pixmap.as_mut();
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pm);
    drop(pm);
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [ICON_SIZE as usize, ICON_SIZE as usize],
        pixmap.data(),
    );
    let name = format!("svg_{}", path.replace(|c: char| !c.is_alphanumeric(), "_"));
    Some(ctx.load_texture(&name, image, egui::TextureOptions::LINEAR))
}

// ---- macOS: extract from .app bundle ----

fn extract_macos(ctx: &egui::Context, exe_path: &std::path::Path) -> Option<egui::TextureHandle> {
    // Find .app bundle root
    let mut app_path = None;
    for ancestor in exe_path.ancestors() {
        if ancestor.extension().map_or(false, |e| e == "app") {
            app_path = Some(ancestor.to_path_buf());
            break;
        }
    }
    let app_path = app_path?;

    // Read Info.plist to find icon file name
    let plist_path = app_path.join("Contents").join("Info.plist");
    let plist_data = std::fs::read(&plist_path).ok()?;
    let plist = plist::Value::from_reader(std::io::Cursor::new(&plist_data)).ok()?;
    let dict = plist.as_dictionary()?;
    let icon_file = dict.get("CFBundleIconFile")?.as_string()?;

    // Look for .icns in Resources
    let resources = app_path.join("Contents").join("Resources");
    let base_name = icon_file.trim_end_matches(".icns");
    let icns_path = resources.join(format!("{}.icns", base_name));
    if !icns_path.exists() {
        return None;
    }

    // Read and decode .icns
    let data = std::fs::read(&icns_path).ok()?;
    let family = icns::IconFamily::read(&data[..]).ok()?;

    // Try icon sizes from large to small
    let types = [
        icns::IconType::RGBA32_256x256,
        icns::IconType::RGBA32_128x128,
        icns::IconType::RGBA32_512x512,
        icns::IconType::RGBA32_64x64,
        icns::IconType::RGBA32_32x32,
    ];

    for &icon_type in &types {
        if let Ok(icns_image) = family.get_icon_with_type(icon_type) {
            let w = icon_type.pixel_width() as usize;
            let h = icon_type.pixel_height() as usize;
            let name = format!("macos_{}", app_path.display().to_string().replace(|c: char| !c.is_alphanumeric(), "_"));
            return Some(make_texture(ctx, &name, icns_image.data(), w, h));
        }
    }

    None
}

// ---- Windows: extract from PE resources via pelite ----

fn extract_windows(ctx: &egui::Context, exe_path: &std::path::Path) -> Option<egui::TextureHandle> {
    let data = std::fs::read(exe_path).ok()?;
    let pe = pelite::PeFile::from_bytes(&data).ok()?;
    let resources = pe.resources().ok()?;

    // Use pelite's built-in icons iterator
    for icon_result in resources.icons() {
        let (_name, group) = icon_result.ok()?;

        // Get entries and find the best icon (prefer 256x256, then larger sizes)
        let entries = group.entries();
        let mut best_entry: Option<&pelite::resources::group::image::GRPICONDIRENTRY> = None;
        for entry in entries {
            let w = if entry.bWidth == 0 { 256u32 } else { entry.bWidth as u32 };
            let is_better = best_entry.map_or(true, |best| {
                let bw = if best.bWidth == 0 { 256u32 } else { best.bWidth as u32 };
                w > bw
            });
            if is_better {
                best_entry = Some(entry);
            }
        }

        let best = best_entry?;

        // Get the icon image data
        let icon_id = best.nId;
        let icon_bytes = group.image(icon_id).ok()?;

        // Decode: could be PNG or DIB
        if icon_bytes.len() >= 8 && &icon_bytes[1..4] == b"PNG" {
            if let Ok(img) = image::load_from_memory(icon_bytes) {
                let img = img.resize_to_fill(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3).to_rgba8();
                return Some(make_texture_from_image(
                    ctx,
                    &format!("pe_{}", exe_path.display().to_string().replace(|c: char| !c.is_alphanumeric(), "_")),
                    &img,
                ));
            }
        }

        // Construct ICO in memory for DIB format
        let mut ico = Vec::with_capacity(22 + icon_bytes.len());
        ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // header
        ico.extend_from_slice(&[best.bWidth, best.bHeight, 0, 0, 0, 0]); // dir entry
        ico.extend_from_slice(&[1, 0]); // planes
        ico.extend_from_slice(&[32, 0]); // bit count
        ico.extend_from_slice(&(icon_bytes.len() as u32).to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(icon_bytes);

        if let Ok(img) = image::load_from_memory(&ico) {
            let img = img.resize_to_fill(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3).to_rgba8();
            return Some(make_texture_from_image(
                ctx,
                &format!("pe_{}", exe_path.display().to_string().replace(|c: char| !c.is_alphanumeric(), "_")),
                &img,
            ));
        }
    }

    None
}

// ---- Fallback: find icon files near the exe ----

fn find_near_exe(ctx: &egui::Context, exe_path: &std::path::Path) -> Option<egui::TextureHandle> {
    let dir = exe_path.parent()?;

    let patterns = [
        "icon.png", "icon.svg", "icon.ico", "product_logo_128.png",
        "app_icon.png", "logo.png", "logo.svg",
    ];
    for name in &patterns {
        let candidate = dir.join(name);
        if candidate.exists() {
            return load_from_path(ctx, &candidate.display().to_string());
        }
    }

    // Search for any icon/logo file
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "png" | "svg" | "ico" | "jpg" | "jpeg") {
                    if let Some(fname) = path.file_name() {
                        let fname = fname.to_string_lossy().to_lowercase();
                        if fname.contains("icon") || fname.contains("logo") {
                            return load_from_path(ctx, &path.display().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

// ---- Texture helpers ----

fn make_texture(ctx: &egui::Context, name: &str, rgba: &[u8], w: usize, h: usize) -> egui::TextureHandle {
    let img = image::RgbaImage::from_raw(w as u32, h as u32, rgba.to_vec()).unwrap();
    let resized = image::DynamicImage::ImageRgba8(img)
        .resize_to_fill(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    make_texture_from_image(ctx, name, &resized)
}

fn make_texture_from_image(ctx: &egui::Context, name: &str, img: &image::RgbaImage) -> egui::TextureHandle {
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        &img.clone().into_raw(),
    );
    ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
}
