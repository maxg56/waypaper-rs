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
    } else {
        launch_gui(cf);
    }
}

/// Build a Config from defaults, config file, and CLI overrides
fn init_config(args: &Args) -> Config {
    let mut cf = Config::new();

    // Path overrides must come before reading the config file
    if let Some(ref path) = args.config_file {
        cf.config_file = path.clone();
    }
    if let Some(ref path) = args.state_file {
        cf.state_file = path.clone();
        cf.use_xdg_state = true;
    }

    cf.read();
    cf.check_validity();

    // Runtime overrides take priority over the config file
    if let Some(ref backend) = args.backend {
        cf.backend = backend.clone();
    }
    if let Some(ref fill) = args.fill {
        cf.fill_option = fill.clone();
    }
    if !args.folder.is_empty() {
        cf.image_folder_list = args.folder.clone();
    }
    if args.no_post_command {
        cf.use_post_command = false;
    }

    cf
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

/// Spawn the wallpaper change on a background thread and wait briefly
fn spawn_wallpaper_change(wallpaper: &PathBuf, cf: &Config, monitor: &str) {
    let cf_clone = cf.clone();
    let wp_clone = wallpaper.clone();
    let monitor_clone = monitor.to_string();
    thread::spawn(move || change_wallpaper(&wp_clone, &cf_clone, &monitor_clone));
    thread::sleep(Duration::from_millis(100));
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
