use std::thread;

use crate::common::{cache_image, get_cached_image_path, get_image_name, get_image_paths};

use super::{LoadState, WaypaperApp};

impl WaypaperApp {
    pub(crate) fn start_loading(&self) {
        let state = std::sync::Arc::clone(&self.load_state);
        let backend = self.cf.backend.clone();
        let folders = self.cf.image_folder_list.clone();
        let include_sub = self.cf.include_subfolders;
        let include_all = self.cf.include_all_subfolders;
        let show_hidden = self.cf.show_hidden;
        let show_gifs = self.cf.show_gifs_only;
        let cache_dir = self.cf.cache_dir.clone();
        let folder_list = folders.clone();
        let show_path = self.cf.show_path_in_tooltip;
        let sort = self.cf.sort_option.clone();

        {
            let mut s = state.lock().unwrap();
            s.loading = true;
            s.image_paths.clear();
            s.image_names.clear();
        }

        thread::spawn(move || {
            let mut paths = get_image_paths(
                &backend,
                &folders,
                include_sub,
                include_all,
                show_hidden,
                show_gifs,
            );

            match sort.as_str() {
                "name" => paths.sort(),
                "namerev" => {
                    paths.sort();
                    paths.reverse();
                }
                "date" => paths.sort_by_key(|p| {
                    std::fs::metadata(p).and_then(|m| m.modified()).ok()
                }),
                "daterev" => {
                    paths.sort_by_key(|p| {
                        std::fs::metadata(p).and_then(|m| m.modified()).ok()
                    });
                    paths.reverse();
                }
                "random" => {
                    use rand::seq::SliceRandom;
                    paths.shuffle(&mut rand::thread_rng());
                }
                _ => {}
            }

            paths.retain(|p| {
                if std::fs::metadata(p).map(|m| m.len()).unwrap_or(0) == 0 {
                    return false;
                }
                let cached = get_cached_image_path(p, &cache_dir);
                if !cached.exists() {
                    if !cache_image(p, &cache_dir) {
                        return false;
                    }
                }
                true
            });

            let names: Vec<String> = paths
                .iter()
                .map(|p| get_image_name(p, &folder_list, show_path))
                .collect();

            let mut s = state.lock().unwrap();
            s.image_paths = paths;
            s.image_names = names;
            s.loading = false;
        });
    }

    pub(crate) fn filtered_images(&self, state: &LoadState) -> Vec<usize> {
        let q = self.search_query.to_lowercase();
        state
            .image_names
            .iter()
            .enumerate()
            .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }
}
