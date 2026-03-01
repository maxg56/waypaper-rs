use egui::Context;

use super::WaypaperApp;

impl WaypaperApp {
    /// Arrow / vim-key grid navigation
    pub(crate) fn handle_keys(&mut self, ctx: &Context, visible: &[usize]) {
        let n = visible.len();
        if n == 0 {
            return;
        }
        let cols = self.cf.number_of_columns;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                if self.selected_index + 1 < n {
                    self.selected_index += 1;
                }
            }
            if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                self.selected_index = self.selected_index.saturating_sub(cols);
            }
            if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                self.selected_index = (self.selected_index + cols).min(n - 1);
            }
            if i.key_pressed(egui::Key::G) {
                self.selected_index = 0;
            }
            // Shift+G = end
            if i.modifiers.shift && i.key_pressed(egui::Key::G) {
                self.selected_index = n.saturating_sub(1);
            }
        });
    }

    /// Global hotkeys: Enter, Q, R, Z, F, S, H
    pub(crate) fn handle_global_keys(&mut self, ctx: &Context) {
        // Collect path for Enter without holding the mutex guard across set_wallpaper
        let maybe_path: Option<std::path::PathBuf> = ctx.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                let state = self.load_state.lock().unwrap();
                let q = self.search_query.to_lowercase();
                let visible: Vec<usize> = state
                    .image_names
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| q.is_empty() || n.to_lowercase().contains(&q))
                    .map(|(i, _)| i)
                    .collect();
                return visible
                    .get(self.selected_index)
                    .and_then(|&idx| state.image_paths.get(idx))
                    .cloned();
            }
            None
        });
        if let Some(p) = maybe_path {
            self.set_wallpaper(&p);
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Q) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if i.key_pressed(egui::Key::R) {
                let _ = std::fs::remove_dir_all(&self.cf.cache_dir);
                let _ = std::fs::create_dir_all(&self.cf.cache_dir);
                self.textures.clear();
                self.start_loading();
            }
            if i.key_pressed(egui::Key::Z) {
                self.cf.zen_mode = !self.cf.zen_mode;
            }
            if i.key_pressed(egui::Key::F) {
                self.want_folder_dialog = true;
            }
            if i.key_pressed(egui::Key::S) {
                self.cf.include_subfolders = !self.cf.include_subfolders;
                self.start_loading();
            }
            if i.key_pressed(egui::Key::H) && !i.modifiers.any() {
                self.cf.show_hidden = !self.cf.show_hidden;
                self.start_loading();
            }
        });
    }
}
