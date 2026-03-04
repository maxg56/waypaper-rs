/// Common utilities: image discovery, thumbnail caching, random selection

use std::path::{Path, PathBuf};
use std::fs;

use crate::options::get_valid_extensions;

/// Strategy for subfolder traversal
enum TraversalMode {
    DirectOnly,
    OneLevel,
    Recursive,
}

/// Collect all image (and optionally video) paths from the given folders
pub fn get_image_paths(
    backend: &str,
    folders: &[PathBuf],
    include_subfolders: bool,
    include_all_subfolders: bool,
    show_hidden: bool,
    show_gifs_only: bool,
) -> Vec<PathBuf> {
    let valid_exts = get_valid_extensions(backend);
    let mode = match (include_all_subfolders, include_subfolders) {
        (true, _) => TraversalMode::Recursive,
        (_, true) => TraversalMode::OneLevel,
        _ => TraversalMode::DirectOnly,
    };

    let mut paths = Vec::new();
    for folder in folders.iter().filter(|f| f.is_dir()) {
        collect_from_folder(folder, &valid_exts, show_hidden, show_gifs_only, &mode, &mut paths);
    }
    paths
}

/// Collect images from a single folder using the specified traversal mode
fn collect_from_folder(
    folder: &Path,
    valid_exts: &[&str],
    show_hidden: bool,
    show_gifs_only: bool,
    mode: &TraversalMode,
    out: &mut Vec<PathBuf>,
) {
    match mode {
        TraversalMode::Recursive => {
            collect_recursive(folder, valid_exts, show_hidden, show_gifs_only, out);
        }
        TraversalMode::OneLevel => {
            collect_one_level(folder, valid_exts, show_hidden, show_gifs_only, out);
        }
        TraversalMode::DirectOnly => {
            collect_direct(folder, valid_exts, show_hidden, show_gifs_only, out);
        }
    }
}

/// One level of subfolders: direct children + immediate subdirectories
fn collect_one_level(
    dir: &Path,
    valid_exts: &[&str],
    show_hidden: bool,
    show_gifs_only: bool,
    out: &mut Vec<PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if is_hidden(&p) && !show_hidden {
            continue;
        }
        if p.is_dir() {
            collect_direct(&p, valid_exts, show_hidden, show_gifs_only, out);
        } else if is_valid_image(&p, valid_exts, show_gifs_only) {
            out.push(p);
        }
    }
}

fn collect_direct(
    dir: &Path,
    valid_exts: &[&str],
    show_hidden: bool,
    show_gifs_only: bool,
    out: &mut Vec<PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if is_hidden(&p) && !show_hidden {
            continue;
        }
        if p.is_file() && is_valid_image(&p, valid_exts, show_gifs_only) {
            out.push(p);
        }
    }
}

fn collect_recursive(
    dir: &Path,
    valid_exts: &[&str],
    show_hidden: bool,
    show_gifs_only: bool,
    out: &mut Vec<PathBuf>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if is_hidden(&p) && !show_hidden {
            continue;
        }
        if p.is_dir() {
            collect_recursive(&p, valid_exts, show_hidden, show_gifs_only, out);
        } else if p.is_file() && is_valid_image(&p, valid_exts, show_gifs_only) {
            out.push(p);
        }
    }
}

fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn is_valid_image(path: &Path, valid_exts: &[&str], gifs_only: bool) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let ext_dot = format!(".{}", ext.to_lowercase());
    if gifs_only {
        return ext_dot == ".gif";
    }
    valid_exts.contains(&ext_dot.as_str())
}

/// Get the display name of an image (relative path or just filename)
pub fn get_image_name(
    path: &Path,
    folder_list: &[PathBuf],
    show_path: bool,
) -> String {
    if show_path {
        // Try to strip the base folder prefix
        for folder in folder_list {
            if let Ok(rel) = path.strip_prefix(folder) {
                return rel.display().to_string();
            }
        }
    }
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// Return the path where the cached (resized) thumbnail should live
pub fn get_cached_image_path(image_path: &Path, cache_dir: &Path) -> PathBuf {
    // Use a hash of the full path as the cache filename
    let hash = format!("{:016x}", fxhash(image_path.to_str().unwrap_or("")));
    cache_dir.join(format!("{hash}.png"))
}

/// Simple non-cryptographic hash (FNV-1a-like)
fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Generate and save a thumbnail to the cache directory.
/// Returns true on success.
pub fn cache_image(image_path: &Path, cache_dir: &Path) -> bool {
    let cached = get_cached_image_path(image_path, cache_dir);
    if cached.exists() {
        return true;
    }

    let Ok(img) = image::open(image_path) else {
        return false;
    };

    // Resize to fit within 240×240
    let thumb = img.thumbnail(240, 240);
    thumb.save(&cached).is_ok()
}

/// Pick a random valid image from the configured folders
pub fn get_random_file(
    backend: &str,
    folders: &[PathBuf],
    include_subfolders: bool,
    include_all_subfolders: bool,
    _cache_dir: &Path,
    show_hidden: bool,
) -> Option<PathBuf> {
    let paths = get_image_paths(
        backend,
        folders,
        include_subfolders,
        include_all_subfolders,
        show_hidden,
        false,
    );
    if paths.is_empty() {
        return None;
    }
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    paths.choose(&mut rng).cloned()
}
