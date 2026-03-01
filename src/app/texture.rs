use std::path::PathBuf;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};

use crate::common::get_cached_image_path;

use super::WaypaperApp;

impl WaypaperApp {
    pub(crate) fn get_or_load_texture(
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
}
