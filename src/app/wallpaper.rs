use std::path::PathBuf;
use std::thread;

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
