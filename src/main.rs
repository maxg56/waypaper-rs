/// waypaper-rs - GUI wallpaper setter for Wayland/Xorg
/// A Rust clone of waypaper (https://github.com/anufrievroman/waypaper)

mod app;
mod changer;
mod common;
mod config;
mod options;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use clap::Parser;

use app::WaypaperApp;
use changer::change_wallpaper;
use common::get_random_file;
use config::Config;

const VERSION: &str = "0.1.0";

#[derive(Parser, Debug)]
#[command(
    name = "waypaper-rs",
    about = "GUI wallpaper setter for Wayland - Rust edition",
    version = VERSION
)]
struct Args {
    /// Restore the previously saved wallpaper(s)
    #[arg(long)]
    restore: bool,

    /// Set a random wallpaper
    #[arg(long)]
    random: bool,

    /// Set a specific wallpaper file
    #[arg(long)]
    wallpaper: Option<PathBuf>,

    /// Set the fill/scaling mode
    #[arg(long)]
    fill: Option<String>,

    /// Override the image folder(s)
    #[arg(long, num_args = 1..)]
    folder: Vec<PathBuf>,

    /// Override the backend
    #[arg(long)]
    backend: Option<String>,

    /// Override the monitor
    #[arg(long)]
    monitor: Option<String>,

    /// Override the config file path
    #[arg(long)]
    config_file: Option<PathBuf>,

    /// Override the state file path
    #[arg(long)]
    state_file: Option<PathBuf>,

    /// Disable running the post-command
    #[arg(long)]
    no_post_command: bool,

    /// Print current wallpaper state as JSON and exit
    #[arg(long)]
    list: bool,
}

fn main() {
    let args = Args::parse();

    let mut cf = Config::new();

    // Apply CLI overrides before reading config
    if let Some(ref path) = args.config_file {
        cf.config_file = path.clone();
    }
    if let Some(ref path) = args.state_file {
        cf.state_file = path.clone();
        cf.use_xdg_state = true;
    }

    cf.read();
    cf.check_validity();

    // Apply remaining CLI overrides (take priority over config)
    if let Some(backend) = args.backend {
        cf.backend = backend;
    }
    if let Some(fill) = args.fill {
        cf.fill_option = fill;
    }
    if !args.folder.is_empty() {
        cf.image_folder_list = args.folder.clone();
    }
    if args.no_post_command {
        cf.use_post_command = false;
    }

    // --list: print JSON and exit
    if args.list {
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
        return;
    }

    // --monitor + optional --wallpaper: set wallpaper on one monitor and exit
    if let Some(monitor) = args.monitor {
        let wallpaper = if let Some(ref wp) = args.wallpaper {
            wp.clone()
        } else {
            match get_random_file(
                &cf.backend,
                &cf.image_folder_list,
                cf.include_subfolders,
                cf.include_all_subfolders,
                &cf.cache_dir,
                cf.show_hidden,
            ) {
                Some(p) => p,
                None => {
                    eprintln!("Could not find a random wallpaper.");
                    return;
                }
            }
        };

        let cf_clone = cf.clone();
        let monitor_clone = monitor.clone();
        let wp_clone = wallpaper.clone();
        thread::spawn(move || change_wallpaper(&wp_clone, &cf_clone, &monitor_clone));
        thread::sleep(Duration::from_millis(100));

        cf.selected_wallpaper = Some(wallpaper);
        cf.selected_monitor = monitor;
        cf.attribute_selected_wallpaper();
        cf.save();
        return;
    }

    // --restore or --random: restore/randomize for all monitors and exit
    if args.restore || args.random {
        let monitor_count = cf.monitors.len();
        for i in 0..monitor_count {
            let wallpaper = if args.random {
                match get_random_file(
                    &cf.backend,
                    &cf.image_folder_list,
                    cf.include_subfolders,
                    cf.include_all_subfolders,
                    &cf.cache_dir,
                    cf.show_hidden,
                ) {
                    Some(p) => {
                        if i < cf.wallpapers.len() {
                            cf.wallpapers[i] = p.clone();
                        } else {
                            cf.wallpapers.push(p.clone());
                        }
                        p
                    }
                    None => continue,
                }
            } else {
                match cf.wallpapers.get(i).cloned() {
                    Some(p) if !p.as_os_str().is_empty() => p,
                    _ => continue,
                }
            };

            let cf_clone = cf.clone();
            let monitor = cf.monitors.get(i).cloned().unwrap_or_else(|| "All".to_string());
            let wp = wallpaper.clone();
            thread::spawn(move || change_wallpaper(&wp, &cf_clone, &monitor));
            thread::sleep(Duration::from_millis(100));
        }
        cf.save();
        return;
    }

    // --wallpaper (without --monitor): set for all monitors and exit
    if let Some(ref wp) = args.wallpaper {
        let monitor = "All".to_string();
        let cf_clone = cf.clone();
        let wp_clone = wp.clone();
        let monitor_clone = monitor.clone();
        thread::spawn(move || change_wallpaper(&wp_clone, &cf_clone, &monitor_clone));
        thread::sleep(Duration::from_millis(100));

        cf.selected_wallpaper = Some(wp.clone());
        cf.selected_monitor = monitor;
        cf.attribute_selected_wallpaper();
        cf.save();
        return;
    }

    // Default: launch GUI
    launch_gui(cf);
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
