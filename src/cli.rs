use std::path::PathBuf;

use clap::Parser;

use crate::config::Config;

const VERSION: &str = "0.1.0";

#[derive(Parser, Debug)]
#[command(
    name = "waypaper-rs",
    about = "GUI wallpaper setter for Wayland - Rust edition",
    version = VERSION
)]
pub(crate) struct Args {
    /// Restore the previously saved wallpaper(s)
    #[arg(long)]
    pub(crate) restore: bool,

    /// Set a random wallpaper
    #[arg(long)]
    pub(crate) random: bool,

    /// Set a specific wallpaper file
    #[arg(long)]
    pub(crate) wallpaper: Option<PathBuf>,

    /// Set the fill/scaling mode
    #[arg(long)]
    pub(crate) fill: Option<String>,

    /// Override the image folder(s)
    #[arg(long, num_args = 1..)]
    pub(crate) folder: Vec<PathBuf>,

    /// Override the backend
    #[arg(long)]
    pub(crate) backend: Option<String>,

    /// Override the monitor
    #[arg(long)]
    pub(crate) monitor: Option<String>,

    /// Override the config file path
    #[arg(long)]
    pub(crate) config_file: Option<PathBuf>,

    /// Override the state file path
    #[arg(long)]
    pub(crate) state_file: Option<PathBuf>,

    /// Disable running the post-command
    #[arg(long)]
    pub(crate) no_post_command: bool,

    /// Print current wallpaper state as JSON and exit
    #[arg(long)]
    pub(crate) list: bool,

    /// Run carousel (slideshow) mode - cycles through images automatically
    #[arg(long)]
    pub(crate) carousel: bool,

    /// Seconds between wallpaper changes in carousel mode (default: 60)
    #[arg(long)]
    pub(crate) interval: Option<u64>,
}

/// Build a Config from defaults, config file, and CLI overrides
pub(crate) fn init_config(args: &Args) -> Config {
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
    if let Some(i) = args.interval {
        cf.carousel_interval = i.max(1);
    }

    cf
}
