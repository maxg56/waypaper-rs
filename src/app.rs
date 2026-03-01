/// Main egui application

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2};

use crate::changer::change_wallpaper;
use crate::common::{cache_image, get_cached_image_path, get_image_name, get_image_paths, get_random_file};
use crate::config::Config;
use crate::options::{
    BACKEND_OPTIONS, FILL_OPTIONS, SORT_DISPLAYS, SORT_OPTIONS, SWWW_TRANSITION_TYPES,
    get_monitor_options,
};

/// State shared between the background loading thread and the UI
#[derive(Default)]
struct LoadState {
    /// Paths of all images found in the folder
    image_paths: Vec<PathBuf>,
    /// Display names for tooltips
    image_names: Vec<String>,
    /// True when the background thread is still processing
    loading: bool,
}

pub struct WaypaperApp {
    pub cf: Config,

    /// Current search query
    search_query: String,

    /// Index of the highlighted image in the (filtered) list
    selected_index: usize,

    /// Shared image loading state
    load_state: Arc<Mutex<LoadState>>,

    /// Loaded egui textures keyed by image path
    textures: std::collections::HashMap<PathBuf, TextureHandle>,

    /// Cached list of monitor options (refreshed when backend changes)
    monitor_options: Vec<String>,

    /// Whether the options popup is open
    show_options: bool,

    /// swww entry fields (stored as strings while editing)
    swww_angle_str: String,
    swww_steps_str: String,
    swww_duration_str: String,
    swww_fps_str: String,

    /// Show/hide the folder picker dialog (native via rfd)
    want_folder_dialog: bool,
}

impl WaypaperApp {
    pub fn new(cc: &eframe::CreationContext<'_>, cf: Config) -> Self {
        // Set a comfortable default font size
        cc.egui_ctx.set_pixels_per_point(1.0);

        let monitor_options = get_monitor_options(&cf.backend);
        let swww_angle_str = cf.swww_transition_angle.to_string();
        let swww_steps_str = cf.swww_transition_step.to_string();
        let swww_duration_str = cf.swww_transition_duration.to_string();
        let swww_fps_str = cf.swww_transition_fps.to_string();

        let mut app = WaypaperApp {
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
        };

        app.start_loading();
        app
    }

    // -------------------------------------------------------------------------
    // Image loading
    // -------------------------------------------------------------------------

    fn start_loading(&self) {
        let state = Arc::clone(&self.load_state);
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

            // Sort
            match sort.as_str() {
                "name" => paths.sort(),
                "namerev" => { paths.sort(); paths.reverse(); }
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

            // Filter zero-byte files and cache thumbnails
            paths.retain(|p| {
                if std::fs::metadata(p).map(|m| m.len()).unwrap_or(0) == 0 {
                    return false;
                }
                // Try to cache; skip files that can't be loaded as images
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

    // -------------------------------------------------------------------------
    // Filtered view
    // -------------------------------------------------------------------------

    fn filtered_images(&self, state: &LoadState) -> Vec<usize> {
        let q = self.search_query.to_lowercase();
        state
            .image_names
            .iter()
            .enumerate()
            .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    // -------------------------------------------------------------------------
    // Wallpaper setting
    // -------------------------------------------------------------------------

    fn set_wallpaper(&mut self, path: &PathBuf) {
        let path = path.clone();
        let mut cf = self.cf.clone();
        self.cf.selected_wallpaper = Some(path.clone());
        self.cf.attribute_selected_wallpaper();
        self.cf.save();

        thread::spawn(move || {
            let monitor = cf.selected_monitor.clone();
            change_wallpaper(&path, &cf, &monitor);
        });
    }

    fn set_random_wallpaper(&mut self) {
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

    // -------------------------------------------------------------------------
    // Texture loading
    // -------------------------------------------------------------------------

    fn get_or_load_texture(
        &mut self,
        ctx: &Context,
        image_path: &PathBuf,
    ) -> Option<&TextureHandle> {
        if !self.textures.contains_key(image_path) {
            let cached = get_cached_image_path(image_path, &self.cf.cache_dir);
            if let Ok(img) = image::open(&cached) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let pixels: Vec<_> = rgba
                    .pixels()
                    .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                    .collect();
                let color_image = ColorImage {
                    size: [w as usize, h as usize],
                    pixels,
                };
                let tex = ctx.load_texture(
                    image_path.to_str().unwrap_or(""),
                    color_image,
                    TextureOptions::LINEAR,
                );
                self.textures.insert(image_path.clone(), tex);
            } else {
                return None;
            }
        }
        self.textures.get(image_path)
    }

    // -------------------------------------------------------------------------
    // Keyboard navigation
    // -------------------------------------------------------------------------

    fn handle_keys(&mut self, ctx: &Context, visible: &[usize]) {
        let n = visible.len();
        if n == 0 { return; }
        let cols = self.cf.number_of_columns;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                if self.selected_index > 0 { self.selected_index -= 1; }
            }
            if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                if self.selected_index + 1 < n { self.selected_index += 1; }
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
            // shift+G = end
            if i.modifiers.shift && i.key_pressed(egui::Key::G) {
                self.selected_index = n.saturating_sub(1);
            }
        });
    }
}

impl eframe::App for WaypaperApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint while loading
        let is_loading = self.load_state.lock().unwrap().loading;
        if is_loading {
            ctx.request_repaint();
        }

        // Clone paths needed from shared state
        let (image_paths, image_names, loading) = {
            let s = self.load_state.lock().unwrap();
            (s.image_paths.clone(), s.image_names.clone(), s.loading)
        };

        let local_state = LoadState {
            image_paths,
            image_names,
            loading,
        };

        let visible: Vec<usize> = self.filtered_images(&local_state);

        // Handle keyboard nav
        self.handle_keys(ctx, &visible);

        // ---- TOP PANEL ----
        if !self.cf.zen_mode {
            egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Folder button
                    if ui.button("📁 Folder").clicked() {
                        self.want_folder_dialog = true;
                    }

                    // Search
                    ui.label("🔍");
                    let resp = ui.text_edit_singleline(&mut self.search_query);
                    if resp.changed() {
                        self.selected_index = 0;
                    }
                    if ui.button("✕").clicked() {
                        self.search_query.clear();
                    }

                    ui.separator();

                    // Sort combo
                    egui::ComboBox::from_id_salt("sort")
                        .selected_text(
                            SORT_DISPLAYS
                                .iter()
                                .find(|(k, _)| *k == self.cf.sort_option.as_str())
                                .map(|(_, v)| *v)
                                .unwrap_or("Sort"),
                        )
                        .show_ui(ui, |ui| {
                            for (key, label) in SORT_DISPLAYS {
                                if ui.selectable_label(self.cf.sort_option == *key, *label).clicked() {
                                    self.cf.sort_option = key.to_string();
                                    self.start_loading();
                                }
                            }
                        });

                    // Refresh
                    if ui.button("⟳ Refresh").on_hover_text("Clear cache and reload").clicked() {
                        // Clear thumbnail cache
                        let _ = std::fs::remove_dir_all(&self.cf.cache_dir);
                        let _ = std::fs::create_dir_all(&self.cf.cache_dir);
                        self.textures.clear();
                        self.start_loading();
                    }

                    // Random
                    if ui.button("🎲 Random").on_hover_text("Set a random wallpaper").clicked() {
                        self.set_random_wallpaper();
                    }

                    // Options
                    if ui.button("⚙ Options").clicked() {
                        self.show_options = !self.show_options;
                    }

                    // Quit
                    if ui.button("✕ Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        } else {
            // In zen mode, pressing 'z' exits zen mode
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Z) {
                    self.cf.zen_mode = false;
                }
            });
        }

        // ---- OPTIONS POPUP ----
        if self.show_options {
            egui::Window::new("Options")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.cf.show_gifs_only, "GIFs only");
                    ui.checkbox(&mut self.cf.include_subfolders, "Include subfolders");
                    if ui.checkbox(&mut self.cf.include_all_subfolders, "Include all subfolders").changed()
                        && self.cf.include_all_subfolders
                    {
                        self.cf.include_subfolders = true;
                    }
                    ui.checkbox(&mut self.cf.show_hidden, "Show hidden files");
                    ui.checkbox(&mut self.cf.show_path_in_tooltip, "Show path in tooltip");
                    ui.checkbox(&mut self.cf.zen_mode, "Zen mode");

                    if ui.button("Apply & Close").clicked() {
                        self.cf.save();
                        self.start_loading();
                        self.show_options = false;
                    }
                });
        }

        // ---- BOTTOM PANEL ----
        if !self.cf.zen_mode {
            egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Backend selector
                    egui::ComboBox::from_id_salt("backend")
                        .selected_text(&self.cf.backend)
                        .show_ui(ui, |ui| {
                            for b in &self.cf.installed_backends.clone() {
                                if ui.selectable_label(self.cf.backend == *b, b).clicked() {
                                    self.cf.backend = b.clone();
                                    self.cf.selected_monitor = "All".to_string();
                                    self.monitor_options = get_monitor_options(&self.cf.backend);
                                }
                            }
                        });

                    // Monitor selector (not for feh/wallutils/none)
                    if !["feh", "wallutils", "none"].contains(&self.cf.backend.as_str()) {
                        egui::ComboBox::from_id_salt("monitor")
                            .selected_text(&self.cf.selected_monitor)
                            .show_ui(ui, |ui| {
                                for m in &self.monitor_options.clone() {
                                    if ui.selectable_label(self.cf.selected_monitor == *m, m).clicked() {
                                        self.cf.selected_monitor = m.clone();
                                    }
                                }
                            });
                    }

                    // Fill selector (not for hyprpaper/none)
                    if !["hyprpaper", "none"].contains(&self.cf.backend.as_str()) {
                        egui::ComboBox::from_id_salt("fill")
                            .selected_text(capitalize(&self.cf.fill_option))
                            .show_ui(ui, |ui| {
                                for f in FILL_OPTIONS {
                                    if ui.selectable_label(self.cf.fill_option == *f, capitalize(f)).clicked() {
                                        self.cf.fill_option = f.to_string();
                                    }
                                }
                            });

                        // Color picker
                        let mut color = hex_to_color32(&self.cf.color);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            self.cf.color = color32_to_hex(color);
                        }
                    }

                    // swww/awww transition options
                    if ["swww", "awww"].contains(&self.cf.backend.as_str()) {
                        egui::ComboBox::from_id_salt("transition")
                            .selected_text(&self.cf.swww_transition_type)
                            .show_ui(ui, |ui| {
                                for t in SWWW_TRANSITION_TYPES {
                                    if ui.selectable_label(self.cf.swww_transition_type == *t, *t).clicked() {
                                        self.cf.swww_transition_type = t.to_string();
                                    }
                                }
                            });

                        ui.add(egui::TextEdit::singleline(&mut self.swww_angle_str).desired_width(40.0).hint_text("angle"));
                        ui.add(egui::TextEdit::singleline(&mut self.swww_steps_str).desired_width(40.0).hint_text("steps"));
                        ui.add(egui::TextEdit::singleline(&mut self.swww_duration_str).desired_width(55.0).hint_text("duration"));
                        ui.add(egui::TextEdit::singleline(&mut self.swww_fps_str).desired_width(40.0).hint_text("fps"));

                        // Apply swww values when they change
                        if let Ok(v) = self.swww_angle_str.parse::<u32>() { self.cf.swww_transition_angle = v; }
                        if let Ok(v) = self.swww_steps_str.parse::<u32>() { self.cf.swww_transition_step = v; }
                        if let Ok(v) = self.swww_duration_str.parse::<f32>() { self.cf.swww_transition_duration = v; }
                        if let Ok(v) = self.swww_fps_str.parse::<u32>() { self.cf.swww_transition_fps = v; }
                    }

                    // mpvpaper controls
                    if self.cf.backend == "mpvpaper" {
                        if ui.button("⏸ Pause").clicked() {
                            let monitor = self.cf.selected_monitor.clone();
                            let cmd = format!("echo 'cycle pause' | socat - /tmp/mpv-socket-{monitor}");
                            let _ = std::process::Command::new("sh").args(["-c", &cmd]).spawn();
                        }
                        if ui.button("⏹ Stop").clicked() {
                            let _ = std::process::Command::new("killall").arg("mpvpaper").spawn();
                        }
                        if ui.checkbox(&mut self.cf.mpvpaper_sound, "Sound").changed() {
                            let monitor = self.cf.selected_monitor.clone();
                            let cmd = format!("echo 'cycle mute' | socat - /tmp/mpv-socket-{monitor}");
                            let _ = std::process::Command::new("sh").args(["-c", &cmd]).spawn();
                        }
                    }

                    // Columns
                    ui.separator();
                    ui.label("Cols:");
                    let mut cols = self.cf.number_of_columns as i32;
                    if ui.add(egui::DragValue::new(&mut cols).range(1..=10)).changed() {
                        self.cf.number_of_columns = cols as usize;
                    }
                });

                if loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Caching thumbnails...");
                    });
                }
            });
        }

        // ---- FOLDER DIALOG ----
        if self.want_folder_dialog {
            self.want_folder_dialog = false;
            if let Some(folder) = rfd_pick_folder() {
                self.cf.image_folder_list = vec![folder];
                self.cf.save();
                self.textures.clear();
                self.start_loading();
            }
        }

        // ---- CENTRAL PANEL: IMAGE GRID ----
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let cols = self.cf.number_of_columns;
                let available_width = ui.available_width();
                let thumb_size = ((available_width - (cols as f32 - 1.0) * 4.0) / cols as f32).max(80.0);

                // Build rows
                let chunks: Vec<&[usize]> = visible.chunks(cols).collect();
                for (row_idx, chunk) in chunks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        for (col_idx, &img_idx) in chunk.iter().enumerate() {
                            let flat_idx = row_idx * cols + col_idx;
                            let path = &local_state.image_paths[img_idx];
                            let name = &local_state.image_names[img_idx];

                            let is_selected = flat_idx == self.selected_index;

                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::splat(thumb_size),
                                egui::Sense::click(),
                            );

                            // Draw selection highlight
                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 180, 255)),
                                );
                            }

                            // Draw thumbnail or placeholder
                            if let Some(tex) = self.get_or_load_texture(ctx, path) {
                                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
                            } else {
                                ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(60));
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "?",
                                    egui::FontId::proportional(24.0),
                                    egui::Color32::GRAY,
                                );
                            }

                            // Tooltip
                            if resp.hovered() {
                                egui::show_tooltip_text(ctx, ui.layer_id(), egui::Id::new(img_idx), name);
                            }

                            // Click to set wallpaper
                            if resp.clicked() {
                                self.selected_index = flat_idx;
                                let path_clone = path.clone();
                                self.set_wallpaper(&path_clone);
                            }

                            // Spacing between images
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(4.0);
                }

                if local_state.image_paths.is_empty() && !loading {
                    ui.centered_and_justified(|ui| {
                        ui.label("No images found. Choose a folder with the button above.");
                    });
                }
            });
        });

        // Handle Enter key to set wallpaper
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                let state = self.load_state.lock().unwrap();
                let visible_local: Vec<usize> = {
                    let q = self.search_query.to_lowercase();
                    state.image_names.iter().enumerate()
                        .filter(|(_, n)| q.is_empty() || n.to_lowercase().contains(&q))
                        .map(|(i, _)| i)
                        .collect()
                };
                if let Some(&idx) = visible_local.get(self.selected_index) {
                    if let Some(path) = state.image_paths.get(idx) {
                        let p = path.clone();
                        drop(state);
                        self.set_wallpaper(&p);
                    }
                }
            }
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

// ---- Helpers ----

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        egui::Color32::from_rgb(r, g, b)
    } else {
        egui::Color32::WHITE
    }
}

fn color32_to_hex(c: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// Native folder picker (cross-platform) using rfd crate
fn rfd_pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}
