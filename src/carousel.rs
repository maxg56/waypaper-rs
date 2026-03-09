use std::thread;
use std::time::Duration;

use crate::changer::spawn_wallpaper_change;
use crate::common::get_image_paths;
use crate::config::Config;

/// --carousel: cycle through images at configured interval until Ctrl+C
pub(crate) fn run_carousel(cf: &mut Config) {
    let mut paths = get_image_paths(
        &cf.backend,
        &cf.image_folder_list,
        cf.include_subfolders,
        cf.include_all_subfolders,
        cf.show_hidden,
        cf.show_gifs_only,
    );

    if paths.is_empty() {
        eprintln!("No images found in configured folders.");
        return;
    }

    let random = cf.carousel_random || cf.sort_option == "random";
    if !random {
        match cf.sort_option.as_str() {
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
            _ => {}
        }
    }

    let interval = Duration::from_secs(cf.carousel_interval);
    let mut index = 0usize;

    loop {
        let wallpaper = if random {
            use rand::seq::SliceRandom;
            paths.choose(&mut rand::thread_rng()).cloned()
        } else {
            paths.get(index).cloned()
        };

        if let Some(wp) = wallpaper {
            let monitor_count = cf.monitors.len();
            for i in 0..monitor_count {
                let monitor = cf.monitors.get(i).cloned()
                    .unwrap_or_else(|| "All".to_string());
                spawn_wallpaper_change(&wp, cf, &monitor);
            }
            cf.selected_wallpaper = Some(wp.clone());
            cf.selected_monitor = cf.monitors.first().cloned()
                .unwrap_or_else(|| "All".to_string());
            cf.attribute_selected_wallpaper();
            cf.save();
            println!("Carousel: {}", wp.display());
        }

        if !random {
            index = (index + 1) % paths.len();
        }

        thread::sleep(interval);
    }
}
