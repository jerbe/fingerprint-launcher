use eframe::egui;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::db;
use crate::models::*;
use crate::ui::main_view::Tab;

pub struct App {
    pub conn: Arc<Mutex<Connection>>,
    pub current_tab: Tab,

    // Profile list
    pub cached_profiles: Vec<Profile>,
    pub current_page: usize,
    pub page_size: usize,
    pub total_profiles: usize,
    pub search_query: String,

    // Browser cache
    pub cached_browsers: Vec<Browser>,
    pub cached_browser_params: HashMap<i64, Vec<BrowserParam>>,

    // Platform cache
    pub cached_platforms: Vec<Platform>,

    // Profile browsers cache
    pub profile_browsers_cache: HashMap<i64, Vec<ProfileBrowser>>,
    // Profile accounts cache: profile_id -> Vec<Account>
    pub profile_accounts_cache: HashMap<i64, Vec<Account>>,
    // account_id -> Vec<platform_id>
    pub account_platforms_cache: HashMap<i64, Vec<i64>>,

    // Profile edit state
    pub profile_edit_open: bool,
    pub editing_profile_id: Option<i64>,
    pub edit_profile_name: String,
    pub edit_selected_browsers: HashSet<i64>,
    pub edit_param_values: HashMap<(i64, i64), String>,
    pub edit_param_enabled: HashSet<i64>,
    pub edit_accounts: Vec<EditAccount>,

    // Settings - browser
    pub new_browser_name: String,
    pub new_browser_english_name: String,
    pub new_browser_path: String,
    pub new_browser_icon_path: Option<String>,
    pub new_param_fields: HashMap<i64, (String, usize, String)>,
    pub browser_delete_confirm: HashMap<i64, String>,

    // Settings - browser edit
    pub editing_browser_id: Option<i64>,
    pub edit_browser_name: String,
    pub edit_browser_english_name: String,
    pub edit_browser_path: String,
    pub edit_browser_icon_path: Option<String>,
    pub edit_browser_new_icon_src: Option<String>,

    // Settings - platform
    pub new_platform_name: String,

    // Browser icon textures
    pub browser_icon_textures: HashMap<i64, egui::TextureHandle>,
    pub default_browser_icon: Option<egui::TextureHandle>,
    pub icons_need_reload: bool,
}

#[derive(Debug, Clone)]
pub struct EditAccount {
    pub id: Option<i64>,
    pub platform_ids: HashSet<i64>,
    pub username: String,
    pub password: String,
    pub show_password: bool,
}

impl App {
    pub fn new(db_path: PathBuf) -> Self {
        let conn = db::init_db(&db_path).expect("Failed to initialize database");
        let conn = Arc::new(Mutex::new(conn));

        let mut app = Self {
            conn,
            current_tab: Tab::default(),
            cached_profiles: Vec::new(),
            current_page: 1,
            page_size: 20,
            total_profiles: 0,
            search_query: String::new(),
            cached_browsers: Vec::new(),
            cached_browser_params: HashMap::new(),
            cached_platforms: Vec::new(),
            profile_browsers_cache: HashMap::new(),
            profile_accounts_cache: HashMap::new(),
            account_platforms_cache: HashMap::new(),
            profile_edit_open: false,
            editing_profile_id: None,
            edit_profile_name: String::new(),
            edit_selected_browsers: HashSet::new(),
            edit_param_values: HashMap::new(),
            edit_param_enabled: HashSet::new(),
            edit_accounts: Vec::new(),
            new_browser_name: String::new(),
            new_browser_english_name: String::new(),
            new_browser_path: String::new(),
            new_browser_icon_path: None,
            new_param_fields: HashMap::new(),
            browser_delete_confirm: HashMap::new(),
            editing_browser_id: None,
            edit_browser_name: String::new(),
            edit_browser_english_name: String::new(),
            edit_browser_path: String::new(),
            edit_browser_icon_path: None,
            edit_browser_new_icon_src: None,
            new_platform_name: String::new(),
            browser_icon_textures: HashMap::new(),
            default_browser_icon: None,
            icons_need_reload: true,
        };
        app.refresh_all();
        app
    }

    pub fn refresh_all(&mut self) {
        self.refresh_browsers();
        self.refresh_platforms();
        self.refresh_profiles();
    }

    pub fn refresh_browsers(&mut self) {
        let conn = self.conn.lock().unwrap();
        self.cached_browsers = db::browser::list_browsers(&conn).unwrap_or_default();
        self.cached_browser_params.clear();
        for b in &self.cached_browsers {
            let params = db::browser::list_browser_params(&conn, b.id).unwrap_or_default();
            self.cached_browser_params.insert(b.id, params);
        }
        self.icons_need_reload = true;
    }

    pub fn refresh_platforms(&mut self) {
        let conn = self.conn.lock().unwrap();
        self.cached_platforms = db::platform::list_platforms(&conn).unwrap_or_default();
    }

    pub fn refresh_profiles(&mut self) {
        let conn = self.conn.lock().unwrap();
        self.total_profiles = db::profile::count_profiles(&conn).unwrap_or(0);
        self.cached_profiles =
            db::profile::list_profiles(&conn, self.current_page, self.page_size).unwrap_or_default();
        self.profile_browsers_cache.clear();
        self.profile_accounts_cache.clear();
        self.account_platforms_cache.clear();
        for p in &self.cached_profiles {
            if let Ok(pbs) = db::profile::list_profile_browsers(&conn, p.id) {
                self.profile_browsers_cache.insert(p.id, pbs);
            }
            if let Ok(accs) = db::account::list_accounts_by_profile(&conn, p.id) {
                for a in &accs {
                    let pids = db::account::get_account_platform_ids(&conn, a.id).unwrap_or_default();
                    self.account_platforms_cache.insert(a.id, pids);
                }
                self.profile_accounts_cache.insert(p.id, accs);
            }
        }
    }

    pub fn get_profile_browsers(&self, profile_id: i64) -> Vec<ProfileBrowser> {
        self.profile_browsers_cache.get(&profile_id).cloned().unwrap_or_default()
    }

    pub fn get_account_platform_names(&self, account_id: i64) -> Vec<String> {
        let pids = self.account_platforms_cache.get(&account_id).cloned().unwrap_or_default();
        pids.iter()
            .filter_map(|pid| self.cached_platforms.iter().find(|p| p.id == *pid).map(|p| p.name.clone()))
            .collect()
    }

    pub fn platform_name(&self, platform_id: i64) -> String {
        self.cached_platforms.iter()
            .find(|p| p.id == platform_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("#{}", platform_id))
    }

    // ========== Browser management ==========
    pub fn add_browser(&mut self) {
        let icon_path = self.process_browser_icon();
        let conn = self.conn.lock().unwrap();
        let _ = db::browser::insert_browser(
            &conn, &self.new_browser_name, &self.new_browser_english_name,
            &self.new_browser_path, icon_path.as_deref(),
        );
        drop(conn);
        self.new_browser_name.clear();
        self.new_browser_english_name.clear();
        self.new_browser_path.clear();
        self.new_browser_icon_path = None;
        self.refresh_browsers();
    }

    fn process_browser_icon(&self) -> Option<String> {
        let src = self.new_browser_icon_path.as_ref()?;
        let src_path = std::path::Path::new(src);
        if !src_path.exists() { return None; }
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let exe_dir = std::path::Path::new(&self.new_browser_path)
            .parent().unwrap_or(std::path::Path::new("."));
        let icon_dir = exe_dir.join("icon").join("browser");
        let _ = std::fs::create_dir_all(&icon_dir);
        let dest = icon_dir.join(format!("{}.{}", self.new_browser_english_name, ext));
        if std::fs::copy(src_path, &dest).is_ok() {
            Some(dest.display().to_string())
        } else { None }
    }

    pub fn can_delete_browser_directly(&self, browser_id: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        db::browser::browser_reference_count(&conn, browser_id) == 0
    }

    pub fn delete_browser(&mut self, id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = db::browser::delete_browser(&conn, id);
        drop(conn);
        self.browser_delete_confirm.remove(&id);
        self.refresh_browsers();
        self.refresh_profiles();
    }

    pub fn add_browser_param(&mut self, browser_id: i64) {
        if let Some((name, vt_idx, desc)) = self.new_param_fields.get(&browser_id).cloned() {
            if !name.trim().is_empty() {
                let vt = ParamValueType::all()[vt_idx].clone();
                let conn = self.conn.lock().unwrap();
                let _ = db::browser::insert_browser_param(&conn, browser_id, &name, &vt, &desc);
                drop(conn);
                self.new_param_fields.remove(&browser_id);
                self.refresh_browsers();
            }
        }
    }

    pub fn delete_browser_param(&mut self, param_id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = db::browser::delete_browser_param(&conn, param_id);
        drop(conn);
        self.refresh_browsers();
    }

    // ========== Browser edit ==========
    pub fn open_edit_browser(&mut self, id: i64) {
        if let Some(b) = self.cached_browsers.iter().find(|b| b.id == id) {
            self.editing_browser_id = Some(id);
            self.edit_browser_name = b.name.clone();
            self.edit_browser_english_name = b.english_name.clone();
            self.edit_browser_path = b.exe_path.clone();
            self.edit_browser_icon_path = b.icon_path.clone();
            self.edit_browser_new_icon_src = None;
        }
    }

    pub fn cancel_edit_browser(&mut self) {
        self.editing_browser_id = None;
    }

    pub fn save_edit_browser(&mut self) {
        let id = match self.editing_browser_id {
            Some(id) => id,
            None => return,
        };

        // Process new icon if selected
        let icon_path = if let Some(ref new_src) = self.edit_browser_new_icon_src {
            Self::copy_browser_icon(&self.edit_browser_english_name, &self.edit_browser_path, new_src)
                .or(self.edit_browser_icon_path.clone())
        } else {
            self.edit_browser_icon_path.clone()
        };

        let conn = self.conn.lock().unwrap();
        let _ = db::browser::update_browser(
            &conn, id,
            &self.edit_browser_name, &self.edit_browser_english_name,
            &self.edit_browser_path, icon_path.as_deref(),
        );
        drop(conn);
        self.editing_browser_id = None;
        self.refresh_browsers();
        self.refresh_profiles();
    }

    fn copy_browser_icon(english_name: &str, exe_path: &str, src: &str) -> Option<String> {
        let src_path = std::path::Path::new(src);
        if !src_path.exists() { return None; }
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let exe_dir = std::path::Path::new(exe_path)
            .parent().unwrap_or(std::path::Path::new("."));
        let icon_dir = exe_dir.join("icon").join("browser");
        let _ = std::fs::create_dir_all(&icon_dir);
        let dest = icon_dir.join(format!("{}.{}", english_name, ext));
        if std::fs::copy(src_path, &dest).is_ok() {
            Some(dest.display().to_string())
        } else { None }
    }

    // ========== Platform management ==========
    pub fn add_platform(&mut self) {
        let conn = self.conn.lock().unwrap();
        let _ = db::platform::insert_platform(&conn, &self.new_platform_name, None);
        drop(conn);
        self.new_platform_name.clear();
        self.refresh_platforms();
    }

    pub fn delete_platform(&mut self, id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = db::platform::delete_platform(&conn, id);
        drop(conn);
        self.refresh_platforms();
    }

    // ========== Profile edit ==========
    pub fn open_new_profile(&mut self) {
        self.profile_edit_open = true;
        self.editing_profile_id = None;
        self.edit_profile_name.clear();
        self.edit_selected_browsers.clear();
        self.edit_param_values.clear();
        self.edit_param_enabled.clear();
        self.edit_accounts.clear();
    }

    pub fn open_edit_profile(&mut self, id: i64) {
        let conn = self.conn.lock().unwrap();

        self.profile_edit_open = true;
        self.editing_profile_id = Some(id);

        // Load profile name
        self.edit_profile_name.clear();
        if let Ok(name) = conn.query_row(
            "SELECT name FROM profiles WHERE id=?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            self.edit_profile_name = name;
        }

        // Load browsers
        self.edit_selected_browsers.clear();
        self.edit_param_values.clear();
        self.edit_param_enabled.clear();
        if let Ok(pbs) = db::profile::list_profile_browsers(&conn, id) {
            for pb in pbs {
                self.edit_selected_browsers.insert(pb.browser_id);
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&pb.launch_args) {
                    for (param_id_str, value) in map {
                        if let Ok(param_id) = param_id_str.parse::<i64>() {
                            if value == "\x01" {
                                self.edit_param_enabled.insert(param_id);
                            } else {
                                self.edit_param_values.insert((pb.browser_id, param_id), value);
                                self.edit_param_enabled.insert(param_id);
                            }
                        }
                    }
                }
            }
        }

        // Load accounts
        self.edit_accounts.clear();
        if let Ok(accs) = db::account::list_accounts_by_profile(&conn, id) {
            for a in accs {
                let pids = db::account::get_account_platform_ids(&conn, a.id).unwrap_or_default();
                self.edit_accounts.push(EditAccount {
                    id: Some(a.id),
                    platform_ids: pids.into_iter().collect(),
                    username: a.username,
                    password: a.password,
                    show_password: false,
                });
            }
        }
    }

    pub fn save_profile(&mut self) {
        let conn = self.conn.lock().unwrap();

        // Check name uniqueness
        if db::profile::profile_name_exists(&conn, &self.edit_profile_name, self.editing_profile_id) {
            return;
        }

        let profile_id = if let Some(id) = self.editing_profile_id {
            let _ = conn.execute(
                "UPDATE profiles SET name=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![self.edit_profile_name, id],
            );
            id
        } else {
            db::profile::insert_profile(&conn, &self.edit_profile_name).unwrap_or(0)
        };

        if profile_id > 0 {
            // Save browsers
            let _ = conn.execute("DELETE FROM profile_browsers WHERE profile_id=?1", rusqlite::params![profile_id]);
            for &browser_id in &self.edit_selected_browsers {
                let mut args_map: HashMap<String, String> = HashMap::new();
                if let Some(params) = self.cached_browser_params.get(&browser_id) {
                    for p in params {
                        if self.edit_param_enabled.contains(&p.id) {
                            if p.value_type == ParamValueType::None {
                                args_map.insert(p.id.to_string(), "\x01".to_string());
                            } else if let Some(val) = self.edit_param_values.get(&(browser_id, p.id)) {
                                if !val.trim().is_empty() {
                                    args_map.insert(p.id.to_string(), val.clone());
                                }
                            }
                        }
                    }
                }
                let launch_args = serde_json::to_string(&args_map).unwrap_or_default();
                let _ = db::profile::upsert_profile_browser(&conn, profile_id, browser_id, &launch_args);
            }

            // Save accounts: delete old, insert all
            let _ = conn.execute("DELETE FROM accounts WHERE profile_id=?1", rusqlite::params![profile_id]);
            for ea in &self.edit_accounts {
                if let Ok(acc_id) = db::account::insert_account(&conn, profile_id, &ea.username, &ea.password) {
                    let pids: Vec<i64> = ea.platform_ids.iter().copied().collect();
                    let _ = db::account::set_account_platforms(&conn, acc_id, &pids);
                }
            }
        }

        drop(conn);
        self.profile_edit_open = false;
        self.refresh_profiles();
    }

    pub fn delete_profile(&mut self, id: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = db::profile::delete_profile(&conn, id);
        drop(conn);
        self.refresh_profiles();
    }

    pub fn launch(&self, browser: &Browser, launch_args_json: &str) {
        let params = self.cached_browser_params.get(&browser.id).cloned().unwrap_or_default();
        let args_map: HashMap<String, String> = serde_json::from_str(launch_args_json).unwrap_or_default();
        let mut cli_args = Vec::new();
        for p in &params {
            if let Some(val) = args_map.get(&p.id.to_string()) {
                if p.value_type == ParamValueType::None {
                    cli_args.push(p.param_name.clone());
                } else {
                    cli_args.push(format!("{}={}", p.param_name, val));
                }
            }
        }
        let args_str = cli_args.join(" ");
        if let Err(e) = crate::launcher::launch_browser(browser, &args_str) {
            eprintln!("Launch error: {}", e);
        }
    }

    // ========== Browser icon loading ==========
    pub fn ensure_browser_icons(&mut self, ctx: &egui::Context) {
        if !self.icons_need_reload {
            return;
        }
        self.icons_need_reload = false;

        if self.default_browser_icon.is_none() {
            self.default_browser_icon = Some(crate::icon::create_default(ctx));
        }

        self.browser_icon_textures.clear();
        for b in &self.cached_browsers {
            let texture = if let Some(ref icon_path) = b.icon_path {
                // Manual icon takes priority
                crate::icon::load_from_path(ctx, icon_path)
            } else {
                // Try to extract from exe
                crate::icon::extract_from_exe(ctx, &b.exe_path)
            };
            if let Some(texture) = texture {
                self.browser_icon_textures.insert(b.id, texture);
            }
        }
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_browser_icons(ctx);
        crate::ui::main_view::show(self, ctx);
    }
}
