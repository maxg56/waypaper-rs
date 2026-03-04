use egui::Context;

use super::WaypaperApp;

/// Direction for grid navigation
enum NavDirection {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

impl WaypaperApp {
    /// Arrow / vim-key grid navigation
    pub(crate) fn handle_keys(&mut self, ctx: &Context, visible: &[usize]) {
        let n = visible.len();
        if n == 0 {
            return;
        }

        let cols = self.cf.number_of_columns;

        // Collect all navigation actions from this frame
        let direction = ctx.input(|i| {
            if i.modifiers.shift && i.key_pressed(egui::Key::G) {
                return Some(NavDirection::End);
            }
            if i.key_pressed(egui::Key::G) {
                return Some(NavDirection::Home);
            }
            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                return Some(NavDirection::Left);
            }
            if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                return Some(NavDirection::Right);
            }
            if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                return Some(NavDirection::Up);
            }
            if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                return Some(NavDirection::Down);
            }
            None
        });

        // Apply navigation outside the input closure
        if let Some(dir) = direction {
            self.apply_navigation(dir, n, cols);
        }
    }

    /// Apply a single navigation movement to the selected index
    fn apply_navigation(&mut self, dir: NavDirection, count: usize, cols: usize) {
        match dir {
            NavDirection::Left => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            NavDirection::Right => {
                if self.selected_index + 1 < count {
                    self.selected_index += 1;
                }
            }
            NavDirection::Up => {
                self.selected_index = self.selected_index.saturating_sub(cols);
            }
            NavDirection::Down => {
                self.selected_index = (self.selected_index + cols).min(count - 1);
            }
            NavDirection::Home => {
                self.selected_index = 0;
            }
            NavDirection::End => {
                self.selected_index = count.saturating_sub(1);
            }
        }
    }

    /// Global hotkeys: Enter, Q, R, Z, F, S, H
    pub(crate) fn handle_global_keys(&mut self, ctx: &Context) {
        self.handle_enter_key(ctx);
        self.handle_shortcut_keys(ctx);
    }

    /// Handle Enter key — sets the currently selected wallpaper
    fn handle_enter_key(&mut self, ctx: &Context) {
        let maybe_path: Option<std::path::PathBuf> = ctx.input(|i| {
            if !i.key_pressed(egui::Key::Enter) {
                return None;
            }
            let state = self.load_state.lock().unwrap();
            let q = self.search_query.to_lowercase();
            let visible: Vec<usize> = state
                .image_names
                .iter()
                .enumerate()
                .filter(|(_, n)| q.is_empty() || n.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect();
            visible
                .get(self.selected_index)
                .and_then(|&idx| state.image_paths.get(idx))
                .cloned()
        });
        if let Some(p) = maybe_path {
            self.set_wallpaper(&p);
        }
    }

    /// Handle single-key shortcuts: Q, R, Z, F, S, H
    fn handle_shortcut_keys(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Q) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if i.key_pressed(egui::Key::R) {
                self.refresh_cache();
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

    /// Clear thumbnail cache and trigger reload
    fn refresh_cache(&mut self) {
        let _ = std::fs::remove_dir_all(&self.cf.cache_dir);
        let _ = std::fs::create_dir_all(&self.cf.cache_dir);
        self.textures.clear();
        self.start_loading();
    }
}
