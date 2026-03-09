/// Main egui application

mod helpers;
mod input;
mod loading;
mod panels;
mod texture;
mod wallpaper;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Context, TextureHandle};

use crate::config::Config;
use crate::options::get_monitor_options;

/// State shared between the background loading thread and the UI
#[derive(Default)]
pub(crate) struct LoadState {
    pub(crate) image_paths: Vec<PathBuf>,
    pub(crate) image_names: Vec<String>,
    pub(crate) loading: bool,
}

pub struct WaypaperApp {
    pub cf: Config,

    pub(crate) search_query: String,
    pub(crate) selected_index: usize,
    pub(crate) load_state: Arc<Mutex<LoadState>>,
    pub(crate) textures: std::collections::HashMap<PathBuf, TextureHandle>,
    pub(crate) monitor_options: Vec<String>,
    pub(crate) show_options: bool,

    /// swww entry fields (stored as strings while editing)
    pub(crate) swww_angle_str: String,
    pub(crate) swww_steps_str: String,
    pub(crate) swww_duration_str: String,
    pub(crate) swww_fps_str: String,

    pub(crate) want_folder_dialog: bool,

    pub(crate) carousel_active: bool,
    pub(crate) carousel_next_change: Option<std::time::Instant>,
    pub(crate) carousel_index: usize,
    pub(crate) carousel_interval_str: String,
}

impl WaypaperApp {
    pub fn new(cc: &eframe::CreationContext<'_>, cf: Config) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.0);

        let monitor_options = get_monitor_options(&cf.backend);
        let swww_angle_str = cf.swww_transition_angle.to_string();
        let swww_steps_str = cf.swww_transition_step.to_string();
        let swww_duration_str = cf.swww_transition_duration.to_string();
        let swww_fps_str = cf.swww_transition_fps.to_string();
        let carousel_interval_str = cf.carousel_interval.to_string();

        let app = WaypaperApp {
            cf,
            search_query: String::new(),
            selected_index: 0,
            load_state: Arc::new(Mutex::new(LoadState::default())),
            textures: Default::default(),
            monitor_options,
            show_options: false,
            swww_angle_str,
            swww_steps_str,
            swww_duration_str,
            swww_fps_str,
            want_folder_dialog: false,
            carousel_active: false,
            carousel_next_change: None,
            carousel_index: 0,
            carousel_interval_str,
        };

        app.start_loading();
        app
    }
}

impl eframe::App for WaypaperApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let is_loading = self.load_state.lock().unwrap_or_else(|e| e.into_inner()).loading;
        if is_loading {
            ctx.request_repaint();
        }

        let (image_paths, image_names, loading) = {
            let s = self.load_state.lock().unwrap_or_else(|e| e.into_inner());
            (s.image_paths.clone(), s.image_names.clone(), s.loading)
        };

        let local_state = LoadState { image_paths, image_names, loading };
        let visible = self.filtered_images(&local_state);

        self.handle_keys(ctx, &visible);
        self.tick_carousel(ctx, &local_state);
        self.show_top_panel(ctx);
        self.show_options_popup(ctx);
        self.show_bottom_panel(ctx, loading);

        if self.want_folder_dialog {
            self.want_folder_dialog = false;
            if let Some(folder) = helpers::rfd_pick_folder() {
                self.cf.image_folder_list = vec![folder];
                self.cf.save();
                self.textures.clear();
                self.start_loading();
            }
        }

        self.show_image_grid(ctx, &local_state, &visible, loading);
        self.handle_global_keys(ctx);
    }
}
