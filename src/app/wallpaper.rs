use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::changer::change_wallpaper;
use crate::common::get_random_file;

use super::WaypaperApp;

impl WaypaperApp {
    pub(crate) fn set_wallpaper(&mut self, path: &PathBuf) {
        let path = path.clone();
        let cf = self.cf.clone();
        self.cf.selected_wallpaper = Some(path.clone());
        self.cf.attribute_selected_wallpaper();
        self.cf.save();

        thread::spawn(move || {
            let monitor = cf.selected_monitor.clone();
            change_wallpaper(&path, &cf, &monitor);
        });
    }

    pub(crate) fn tick_carousel(&mut self, ctx: &egui::Context, state: &super::LoadState) {
        if !self.carousel_active {
            return;
        }
        let interval = Duration::from_secs(self.cf.carousel_interval);
        if let Some(next) = self.carousel_next_change {
            if Instant::now() >= next {
                self.advance_carousel(state);
                self.carousel_next_change = Some(Instant::now() + interval);
            } else {
                ctx.request_repaint_after(next - Instant::now());
                return;
            }
        } else {
            self.carousel_next_change = Some(Instant::now() + interval);
        }
        ctx.request_repaint_after(interval);
    }

    fn advance_carousel(&mut self, state: &super::LoadState) {
        if state.image_paths.is_empty() {
            return;
        }
        if self.cf.carousel_random {
            self.set_random_wallpaper();
        } else {
            let idx = self.carousel_index % state.image_paths.len();
            let path = state.image_paths[idx].clone();
            self.set_wallpaper(&path);
            self.carousel_index = (idx + 1) % state.image_paths.len();
        }
    }

    pub(crate) fn set_random_wallpaper(&mut self) {
        let cf = &self.cf;
        if let Some(path) = get_random_file(
            &cf.backend,
            &cf.image_folder_list,
            cf.include_subfolders,
            cf.include_all_subfolders,
            &cf.cache_dir,
            cf.show_hidden,
        ) {
            self.set_wallpaper(&path.clone());
        }
    }
}
