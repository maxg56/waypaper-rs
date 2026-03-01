/// Configuration management - reads/writes config.ini in XDG config dir

use std::path::PathBuf;
use configparser::ini::Ini;

use crate::options::{
    check_installed_backends, FILL_OPTIONS, SORT_OPTIONS, SWWW_TRANSITION_TYPES,
};

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub home_path: PathBuf,

    // Folders and wallpapers
    pub image_folder_list: Vec<PathBuf>,
    pub wallpapers: Vec<PathBuf>,
    pub monitors: Vec<String>,

    // Currently selected
    pub selected_wallpaper: Option<PathBuf>,
    pub selected_monitor: String,

    // Display options
    pub fill_option: String,
    pub sort_option: String,
    pub color: String,
    pub number_of_columns: usize,

    // swww/awww transitions
    pub swww_transition_type: String,
    pub swww_transition_step: u32,
    pub swww_transition_angle: u32,
    pub swww_transition_duration: f32,
    pub swww_transition_fps: u32,

    // mpvpaper
    pub mpvpaper_sound: bool,
    pub mpvpaper_options: String,

    // Display toggles
    pub include_subfolders: bool,
    pub include_all_subfolders: bool,
    pub show_hidden: bool,
    pub show_gifs_only: bool,
    pub zen_mode: bool,
    pub show_path_in_tooltip: bool,

    // Backend
    pub backend: String,
    pub installed_backends: Vec<String>,

    // Post command
    pub post_command: String,
    pub use_post_command: bool,

    // XDG state support
    pub use_xdg_state: bool,

    // Paths
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub state_dir: PathBuf,
    pub state_file: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let home_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home_path.join(".config"))
            .join("waypaper");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| home_path.join(".cache"))
            .join("waypaper");
        let state_dir = dirs::state_dir()
            .unwrap_or_else(|| home_path.join(".local/state"))
            .join("waypaper");

        // Create directories
        let _ = std::fs::create_dir_all(&config_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(&state_dir);

        let installed_backends = check_installed_backends();
        let default_backend = installed_backends.last().cloned().unwrap_or_else(|| "none".to_string());

        let image_folder_fallback = dirs::picture_dir()
            .unwrap_or_else(|| home_path.join("Pictures"));

        Config {
            name: "waypaper".to_string(),
            home_path: home_path.clone(),
            image_folder_list: vec![image_folder_fallback],
            wallpapers: vec![],
            monitors: vec!["All".to_string()],
            selected_wallpaper: None,
            selected_monitor: "All".to_string(),
            fill_option: FILL_OPTIONS[0].to_string(),
            sort_option: SORT_OPTIONS[0].to_string(),
            color: "#ffffff".to_string(),
            number_of_columns: 3,
            swww_transition_type: SWWW_TRANSITION_TYPES[0].to_string(),
            swww_transition_step: 63,
            swww_transition_angle: 0,
            swww_transition_duration: 2.0,
            swww_transition_fps: 60,
            mpvpaper_sound: false,
            mpvpaper_options: String::new(),
            include_subfolders: false,
            include_all_subfolders: false,
            show_hidden: false,
            show_gifs_only: false,
            zen_mode: false,
            show_path_in_tooltip: true,
            backend: default_backend,
            installed_backends,
            post_command: String::new(),
            use_post_command: true,
            use_xdg_state: false,
            config_file: config_dir.join("config.ini"),
            state_file: state_dir.join("state.ini"),
            config_dir,
            cache_dir,
            state_dir,
        }
    }

    pub fn read(&mut self) {
        let mut ini = Ini::new();
        if ini.load(&self.config_file).is_err() {
            return;
        }

        let get = |key: &str| -> Option<String> { ini.get("settings", key) };

        if let Some(v) = get("fill") { self.fill_option = v; }
        if let Some(v) = get("sort") { self.sort_option = v; }
        if let Some(v) = get("backend") { self.backend = v; }
        if let Some(v) = get("color") { self.color = v; }
        if let Some(v) = get("post_command") { self.post_command = v; }
        if let Some(v) = get("swww_transition_type") { self.swww_transition_type = v; }
        if let Some(v) = get("swww_transition_step") { self.swww_transition_step = v.parse().unwrap_or(63); }
        if let Some(v) = get("swww_transition_angle") { self.swww_transition_angle = v.parse().unwrap_or(0); }
        if let Some(v) = get("swww_transition_duration") { self.swww_transition_duration = v.parse().unwrap_or(2.0); }
        if let Some(v) = get("swww_transition_fps") { self.swww_transition_fps = v.parse().unwrap_or(60); }
        if let Some(v) = get("mpvpaper_sound") { self.mpvpaper_sound = v.to_lowercase() == "true"; }
        if let Some(v) = get("mpvpaper_options") { self.mpvpaper_options = v; }
        if let Some(v) = get("number_of_columns") { self.number_of_columns = v.parse().unwrap_or(3).max(1); }
        if let Some(v) = get("subfolders") { self.include_subfolders = v.to_lowercase() == "true"; }
        if let Some(v) = get("all_subfolders") { self.include_all_subfolders = v.to_lowercase() == "true"; }
        if let Some(v) = get("show_hidden") { self.show_hidden = v.to_lowercase() == "true"; }
        if let Some(v) = get("show_gifs_only") { self.show_gifs_only = v.to_lowercase() == "true"; }
        if let Some(v) = get("zen_mode") { self.zen_mode = v.to_lowercase() == "true"; }
        if let Some(v) = get("use_xdg_state") { self.use_xdg_state = v.to_lowercase() == "true"; }
        if let Some(v) = get("show_path_in_tooltip") { self.show_path_in_tooltip = v.to_lowercase() == "true"; }

        // Parse multi-line folder list
        if let Some(v) = get("folder") {
            self.image_folder_list = v
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| expand_tilde(s, &self.home_path))
                .collect();
        }

        // Parse monitors and wallpapers
        if let Some(v) = get("monitors") {
            self.monitors = v.split('\n').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(v) = get("wallpaper") {
            self.wallpapers = v
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| expand_tilde(s, &self.home_path))
                .collect();
        }
    }

    pub fn save(&self) {
        let mut ini = Ini::new();
        // Load existing to preserve unknown keys
        let _ = ini.load(&self.config_file);

        let set = |ini: &mut Ini, key: &str, val: &str| {
            ini.set("settings", key, Some(val.to_string()));
        };

        set(&mut ini, "fill", &self.fill_option);
        set(&mut ini, "sort", &self.sort_option);
        set(&mut ini, "backend", &self.backend);
        set(&mut ini, "color", &self.color);
        set(&mut ini, "post_command", &self.post_command);
        set(&mut ini, "swww_transition_type", &self.swww_transition_type);
        set(&mut ini, "swww_transition_step", &self.swww_transition_step.to_string());
        set(&mut ini, "swww_transition_angle", &self.swww_transition_angle.to_string());
        set(&mut ini, "swww_transition_duration", &self.swww_transition_duration.to_string());
        set(&mut ini, "swww_transition_fps", &self.swww_transition_fps.to_string());
        set(&mut ini, "mpvpaper_sound", &self.mpvpaper_sound.to_string());
        set(&mut ini, "mpvpaper_options", &self.mpvpaper_options);
        set(&mut ini, "number_of_columns", &self.number_of_columns.to_string());
        set(&mut ini, "subfolders", &self.include_subfolders.to_string());
        set(&mut ini, "all_subfolders", &self.include_all_subfolders.to_string());
        set(&mut ini, "show_hidden", &self.show_hidden.to_string());
        set(&mut ini, "show_gifs_only", &self.show_gifs_only.to_string());
        set(&mut ini, "zen_mode", &self.zen_mode.to_string());
        set(&mut ini, "use_xdg_state", &self.use_xdg_state.to_string());
        set(&mut ini, "show_path_in_tooltip", &self.show_path_in_tooltip.to_string());

        // Folder list (multi-line)
        let folder_str = self
            .image_folder_list
            .iter()
            .map(|p| self.shorten_path(p))
            .collect::<Vec<_>>()
            .join("\n     ");
        set(&mut ini, "folder", &folder_str);

        // Monitors and wallpapers
        let monitors_str = self.monitors.join("\n");
        set(&mut ini, "monitors", &monitors_str);
        let wallpapers_str = self
            .wallpapers
            .iter()
            .map(|p| self.shorten_path(p))
            .collect::<Vec<_>>()
            .join("\n");
        set(&mut ini, "wallpaper", &wallpapers_str);

        if let Err(e) = ini.write(&self.config_file) {
            eprintln!("Could not save config: {e}");
        }
    }

    pub fn shorten_path(&self, path: &PathBuf) -> String {
        if let Ok(rel) = path.strip_prefix(&self.home_path) {
            format!("~/{}", rel.display())
        } else {
            path.display().to_string()
        }
    }

    /// Assign the selected wallpaper to the selected monitor in the lists
    pub fn attribute_selected_wallpaper(&mut self) {
        let Some(ref wp) = self.selected_wallpaper.clone() else {
            return;
        };
        if self.selected_monitor == "All" {
            self.monitors = vec!["All".to_string()];
            self.wallpapers = vec![wp.clone()];
        } else if let Some(idx) = self.monitors.iter().position(|m| m == &self.selected_monitor) {
            if idx < self.wallpapers.len() {
                self.wallpapers[idx] = wp.clone();
            } else {
                self.wallpapers.push(wp.clone());
            }
        } else {
            self.monitors.push(self.selected_monitor.clone());
            self.wallpapers.push(wp.clone());
        }
    }

    pub fn check_validity(&mut self) {
        if !crate::options::FILL_OPTIONS.contains(&self.fill_option.as_str()) {
            self.fill_option = FILL_OPTIONS[0].to_string();
        }
        if !crate::options::SORT_OPTIONS.contains(&self.sort_option.as_str()) {
            self.sort_option = SORT_OPTIONS[0].to_string();
        }
        if !self.installed_backends.contains(&self.backend) {
            self.backend = self.installed_backends.last().cloned().unwrap_or_else(|| "none".to_string());
        }
        if !crate::options::SWWW_TRANSITION_TYPES.contains(&self.swww_transition_type.as_str()) {
            self.swww_transition_type = "any".to_string();
        }
        if self.number_of_columns == 0 {
            self.number_of_columns = 1;
        }
    }
}

fn expand_tilde(path: &str, home: &PathBuf) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home.join(rest)
    } else if path == "~" {
        home.clone()
    } else {
        PathBuf::from(path)
    }
}
