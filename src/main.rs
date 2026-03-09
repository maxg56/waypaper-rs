/// waypaper-rs - GUI wallpaper setter for Wayland/Xorg
/// A Rust clone of waypaper (https://github.com/anufrievroman/waypaper)

mod app;
mod carousel;
mod changer;
mod cli;
mod common;
mod config;
mod options;

use std::path::PathBuf;

use clap::Parser;

use app::WaypaperApp;
use changer::spawn_wallpaper_change;
use cli::{Args, init_config};
use common::get_random_file;
use config::Config;

fn main() {
    let args = Args::parse();
    let mut cf = init_config(&args);

    // Dispatch to the appropriate CLI command, or launch the GUI
    if args.list {
        print_wallpaper_list(&cf);
    } else if let Some(ref monitor) = args.monitor {
        set_wallpaper_on_monitor(&mut cf, &args, monitor);
    } else if args.restore || args.random {
        restore_or_randomize(&mut cf, args.random);
    } else if let Some(ref wp) = args.wallpaper {
        set_wallpaper_all_monitors(&mut cf, wp);
    } else if args.carousel {
        carousel::run_carousel(&mut cf);
    } else {
        launch_gui(cf);
    }
}

/// --list: print JSON and exit
fn print_wallpaper_list(cf: &Config) {
    let items: Vec<String> = cf
        .monitors
        .iter()
        .zip(cf.wallpapers.iter())
        .map(|(m, w)| {
            format!(
                r#"{{"monitor":"{}","wallpaper":"{}","backend":"{}"}}"#,
                m,
                w.display(),
                cf.backend
            )
        })
        .collect();
    println!("[{}]", items.join(","));
}

/// --monitor: set wallpaper on a specific monitor and exit
fn set_wallpaper_on_monitor(cf: &mut Config, args: &Args, monitor: &str) {
    let wallpaper = args.wallpaper.clone().or_else(|| {
        get_random_file(
            &cf.backend,
            &cf.image_folder_list,
            cf.include_subfolders,
            cf.include_all_subfolders,
            &cf.cache_dir,
            cf.show_hidden,
        )
    });

    let Some(wallpaper) = wallpaper else {
        eprintln!("Could not find a random wallpaper.");
        return;
    };

    spawn_wallpaper_change(&wallpaper, cf, monitor);

    cf.selected_wallpaper = Some(wallpaper);
    cf.selected_monitor = monitor.to_string();
    cf.attribute_selected_wallpaper();
    cf.save();
}

/// --restore / --random: apply wallpapers to all configured monitors
fn restore_or_randomize(cf: &mut Config, randomize: bool) {
    let monitor_count = cf.monitors.len();
    for i in 0..monitor_count {
        let wallpaper = if randomize {
            resolve_random_wallpaper(cf, i)
        } else {
            cf.wallpapers.get(i).cloned().filter(|p| !p.as_os_str().is_empty())
        };

        let Some(wallpaper) = wallpaper else { continue };
        let monitor = cf.monitors.get(i).cloned().unwrap_or_else(|| "All".to_string());
        spawn_wallpaper_change(&wallpaper, cf, &monitor);
    }
    cf.save();
}

/// Pick a random wallpaper and store it in the wallpapers list at the given index
fn resolve_random_wallpaper(cf: &mut Config, index: usize) -> Option<PathBuf> {
    let p = get_random_file(
        &cf.backend,
        &cf.image_folder_list,
        cf.include_subfolders,
        cf.include_all_subfolders,
        &cf.cache_dir,
        cf.show_hidden,
    )?;
    if index < cf.wallpapers.len() {
        cf.wallpapers[index] = p.clone();
    } else {
        cf.wallpapers.push(p.clone());
    }
    Some(p)
}

/// --wallpaper (without --monitor): set for all monitors and exit
fn set_wallpaper_all_monitors(cf: &mut Config, wp: &PathBuf) {
    let monitor = "All".to_string();
    spawn_wallpaper_change(wp, cf, &monitor);

    cf.selected_wallpaper = Some(wp.clone());
    cf.selected_monitor = monitor;
    cf.attribute_selected_wallpaper();
    cf.save();
}

fn launch_gui(cf: Config) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("waypaper-rs")
            .with_inner_size([820.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "waypaper-rs",
        native_options,
        Box::new(|cc| Ok(Box::new(WaypaperApp::new(cc, cf)))),
    )
    .expect("Failed to launch GUI");
}
