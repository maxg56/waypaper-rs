/// Wallpaper backend implementations - calls external programs to set wallpaper

use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::Config;

/// Find the PID of a running process matching the given command string
fn find_pid(pattern: &str) -> Option<u32> {
    let output = Command::new("ps").args(["aux"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains(pattern) {
            let pid: u32 = line.split_whitespace().nth(1)?.parse().ok()?;
            return Some(pid);
        }
    }
    None
}

/// Kill a process by PID
fn kill_pid(pid: u32) {
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
}

/// Kill all instances of a process, or only the one for a specific monitor
fn seek_and_destroy(process: &str, monitor: &str) {
    if monitor == "All" {
        if let Ok(output) = Command::new("pgrep").arg(process).output() {
            if output.status.success() {
                let _ = Command::new("killall").arg(process).spawn();
                thread::sleep(Duration::from_millis(100));
            }
        }
    } else {
        let pattern = match process {
            "mpvpaper" => format!("mpvpaper -f socket-{monitor}"),
            "swaybg" => format!("swaybg -o {monitor}"),
            _ => return,
        };
        if let Some(pid) = find_pid(&pattern) {
            kill_pid(pid);
        }
    }
}

fn change_with_swaybg(image_path: &Path, cf: &Config, monitor: &str) {
    // Find existing swaybg PID before launching new one
    let old_pid = if monitor == "All" {
        find_pid("swaybg")
    } else {
        find_pid(&format!("swaybg -o {monitor}"))
    };

    let fill = cf.fill_option.to_lowercase();
    let mut cmd = Command::new("swaybg");
    if monitor != "All" {
        cmd.args(["-o", monitor]);
    }
    cmd.args(["-i", image_path.to_str().unwrap_or("")]);
    cmd.args(["-m", &fill, "-c", &cf.color]);
    let _ = cmd.spawn();

    // Kill old instance after new one started
    if let Some(pid) = old_pid {
        thread::sleep(Duration::from_millis(200));
        kill_pid(pid);
    }
}

fn change_with_mpvpaper(image_path: &Path, cf: &Config, monitor: &str) {
    let fill_map = [
        ("fill", "panscan=1.0"),
        ("fit", "panscan=0.0"),
        ("center", ""),
        ("stretch", "--keepaspect=no"),
        ("tile", ""),
    ];
    let fill = fill_map
        .iter()
        .find(|(k, _)| *k == cf.fill_option.as_str())
        .map(|(_, v)| *v)
        .unwrap_or("");

    // Check if mpvpaper already running on this monitor
    let already_running = Command::new("pgrep")
        .args(["-f", &format!("socket-{monitor}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if already_running {
        thread::sleep(Duration::from_millis(200));
        let cmd = format!(
            "echo 'loadfile \"{}\"' | socat - /tmp/mpv-socket-{}",
            image_path.display(),
            monitor
        );
        let _ = Command::new("sh").args(["-c", &cmd]).spawn();
    } else {
        let sound_flag = if cf.mpvpaper_sound { "" } else { "--mute=yes" };
        let opts = format!(
            "input-ipc-server=/tmp/mpv-socket-{monitor} {} loop {fill} {sound_flag} --background-color='{}'",
            cf.mpvpaper_options,
            cf.color
        );
        let mut cmd = Command::new("mpvpaper");
        cmd.args(["--fork", "-o", &opts]);
        if monitor == "All" {
            cmd.arg("*");
        } else {
            cmd.arg(monitor);
        }
        cmd.arg(image_path);
        let _ = cmd.spawn();
    }
}

/// Build swww/awww transition arguments from config
fn build_transition_args(cf: &Config) -> Vec<String> {
    vec![
        "--fill-color".to_string(), cf.color.trim_start_matches('#').to_string(),
        "--transition-type".to_string(), cf.swww_transition_type.clone(),
        "--transition-step".to_string(), cf.swww_transition_step.to_string(),
        "--transition-angle".to_string(), cf.swww_transition_angle.to_string(),
        "--transition-duration".to_string(), cf.swww_transition_duration.to_string(),
        "--transition-fps".to_string(), cf.swww_transition_fps.to_string(),
    ]
}

/// Resolve fill option to backend-specific resize value
fn resolve_fill(cf: &Config, fill_map: &[(&str, &str)], default: &str) -> String {
    fill_map
        .iter()
        .find(|(k, _)| *k == cf.fill_option.as_str())
        .map(|(_, v)| *v)
        .unwrap_or(default)
        .to_string()
}

/// Ensure a daemon process is running, starting it if needed
fn ensure_daemon_running(daemon: &str, wait_ms: u64) {
    let running = Command::new("pgrep")
        .arg(daemon)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !running {
        let _ = Command::new(daemon).spawn();
        thread::sleep(Duration::from_millis(wait_ms));
    }
}

/// Common fill map for swww/awww backends
const SWWW_FILL_MAP: &[(&str, &str)] = &[
    ("fill", "crop"),
    ("fit", "fit"),
    ("center", "no"),
    ("stretch", "crop"),
    ("tile", "no"),
];

fn change_with_swww(image_path: &Path, cf: &Config, monitor: &str) {
    seek_and_destroy("swaybg", "All");
    seek_and_destroy("hyprpaper", "All");
    seek_and_destroy("awww-daemon", "All");

    let fill = resolve_fill(cf, SWWW_FILL_MAP, "crop");
    ensure_daemon_running("swww-daemon", 500);

    let mut cmd = Command::new("swww");
    cmd.arg("img");
    cmd.arg(image_path);
    cmd.args(["--resize", &fill]);
    for arg in build_transition_args(cf) {
        cmd.arg(arg);
    }
    if monitor != "All" {
        cmd.args(["--outputs", monitor]);
    }
    let _ = cmd.status();
}

fn change_with_awww(image_path: &Path, cf: &Config, monitor: &str) {
    seek_and_destroy("swaybg", "All");
    seek_and_destroy("hyprpaper", "All");
    seek_and_destroy("swww-daemon", "All");

    let fill = resolve_fill(cf, SWWW_FILL_MAP, "crop");
    ensure_daemon_running("awww-daemon", 500);

    let mut cmd = Command::new("awww");
    cmd.arg("img");
    cmd.arg(image_path);
    cmd.args(["--resize", &fill]);
    for arg in build_transition_args(cf) {
        cmd.arg(arg);
    }
    if monitor != "All" {
        cmd.args(["--outputs", monitor]);
    }
    let _ = cmd.status();
}

fn change_with_feh(image_path: &Path, cf: &Config, _monitor: &str) {
    let fill_map = [
        ("fill", "--bg-fill"),
        ("fit", "--bg-max"),
        ("center", "--bg-center"),
        ("stretch", "--bg-scale"),
        ("tile", "--bg-tile"),
    ];
    let fill = resolve_fill(cf, &fill_map, "--bg-fill");
    let _ = Command::new("feh")
        .args([&fill, "--image-bg", &cf.color])
        .arg(image_path)
        .spawn();
}

fn change_with_xwallpaper(image_path: &Path, cf: &Config, monitor: &str) {
    let fill_map = [
        ("fill", "--zoom"),
        ("fit", "--maximize"),
        ("center", "--center"),
        ("stretch", "--stretch"),
        ("tile", "--tile"),
    ];
    let fill = resolve_fill(cf, &fill_map, "--zoom");
    let mon = if monitor == "All" { "all" } else { monitor };
    let _ = Command::new("xwallpaper")
        .args(["--output", mon, &fill])
        .arg(image_path)
        .spawn();
}

fn change_with_wallutils(image_path: &Path, cf: &Config, _monitor: &str) {
    let fill_map = [
        ("fill", "scale"),
        ("fit", "scale"),
        ("center", "center"),
        ("stretch", "stretch"),
        ("tile", "tile"),
    ];
    let fill = resolve_fill(cf, &fill_map, "scale");
    let _ = Command::new("setwallpaper")
        .args(["--mode", &fill])
        .arg(image_path)
        .spawn();
}

fn change_with_hyprpaper(image_path: &Path, _cf: &Config, monitor: &str) {
    // Ensure hyprpaper is running
    ensure_daemon_running("hyprpaper", 1000);

    // Determine affected monitors
    let monitors: Vec<String> = if monitor == "All" {
        crate::options::get_monitor_options("hyprpaper")
            .into_iter()
            .filter(|m| m != "All")
            .collect()
    } else {
        vec![monitor.to_string()]
    };

    let path_str = image_path.to_str().unwrap_or("");

    for m in &monitors {
        // Unload and preload
        let _ = Command::new("hyprctl")
            .args(["hyprpaper", "unload", "all"])
            .output();
        let _ = Command::new("hyprctl")
            .args(["hyprpaper", "preload", path_str])
            .output();

        // Set wallpaper with retries
        for _ in 0..10 {
            let result = Command::new("hyprctl")
                .args(["hyprpaper", "wallpaper", &format!("{m},{path_str}")])
                .output();
            if result.map(|o| o.status.success()).unwrap_or(false) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Main dispatch function - sets the wallpaper using the configured backend
pub fn change_wallpaper(image_path: &Path, cf: &Config, monitor: &str) {
    println!("Setting wallpaper: {} on {monitor} via {}", image_path.display(), cf.backend);

    let result = dispatch_backend(image_path, cf, monitor);

    if result.is_ok() && cf.backend != "none" {
        let filename = image_path.file_name().unwrap_or_default().to_string_lossy();
        println!("Set {filename} on {monitor} via {}", cf.backend);
    }

    run_post_command(image_path, cf, monitor);
}

/// Dispatch to the correct backend implementation
fn dispatch_backend(image_path: &Path, cf: &Config, monitor: &str) -> Result<(), ()> {
    match cf.backend.as_str() {
        "swaybg" => { change_with_swaybg(image_path, cf, monitor); Ok(()) }
        "mpvpaper" => { change_with_mpvpaper(image_path, cf, monitor); Ok(()) }
        "swww" => { change_with_swww(image_path, cf, monitor); Ok(()) }
        "awww" => { change_with_awww(image_path, cf, monitor); Ok(()) }
        "feh" => { change_with_feh(image_path, cf, monitor); Ok(()) }
        "xwallpaper" => { change_with_xwallpaper(image_path, cf, monitor); Ok(()) }
        "wallutils" => { change_with_wallutils(image_path, cf, monitor); Ok(()) }
        "hyprpaper" => { change_with_hyprpaper(image_path, cf, monitor); Ok(()) }
        "none" => Ok(()),
        b => {
            eprintln!("Unknown backend: {b}");
            Err(())
        }
    }
}

/// Execute the user-configured post-command, if any
fn run_post_command(image_path: &Path, cf: &Config, monitor: &str) {
    if cf.post_command.is_empty() || !cf.use_post_command {
        return;
    }
    let path_escaped = image_path.display().to_string().replace(' ', "\\ ");
    let cmd = cf.post_command
        .replace("$wallpaper", &path_escaped)
        .replace("$monitor", monitor);
    let _ = Command::new("sh").args(["-c", &cmd]).spawn();
    println!("Executed post-command: {cmd}");
}
