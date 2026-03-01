/// All constant options and backend definitions

pub const BACKEND_OPTIONS: &[&str] = &[
    "none", "swaybg", "swww", "feh", "xwallpaper", "wallutils", "hyprpaper", "mpvpaper", "awww",
];

pub const FILL_OPTIONS: &[&str] = &["fill", "stretch", "fit", "center", "tile"];

pub const SORT_OPTIONS: &[&str] = &["name", "namerev", "date", "daterev", "random"];

pub const SORT_DISPLAYS: &[(&str, &str)] = &[
    ("name", "Name ↓"),
    ("namerev", "Name ↑"),
    ("date", "Date ↓"),
    ("daterev", "Date ↑"),
    ("random", "Random"),
];

pub const SWWW_TRANSITION_TYPES: &[&str] = &[
    "any", "none", "simple", "fade", "wipe", "left", "right", "top", "bottom", "wave", "grow",
    "center", "outer", "random",
];

pub const IMAGE_EXTENSIONS: &[&str] = &[
    ".gif", ".jpg", ".jpeg", ".png", ".webp", ".bmp", ".pnm", ".tiff", ".avif", ".jxl",
];

pub const VIDEO_EXTENSIONS: &[&str] = &[
    ".webm", ".mkv", ".flv", ".vob", ".ogv", ".ogg", ".gifv", ".mng", ".mov", ".avi", ".qt",
    ".wmv", ".mp4", ".m4p", ".m4v", ".mpg", ".mp2", ".mpeg", ".mpe", ".mpv", ".3gp", ".3g2",
];

/// Returns all valid extensions for the given backend (images + optionally videos)
pub fn get_valid_extensions(backend: &str) -> Vec<&'static str> {
    let video_backends = ["mpvpaper", "awww"];
    let mut exts: Vec<&str> = IMAGE_EXTENSIONS.to_vec();
    if video_backends.contains(&backend) {
        exts.extend_from_slice(VIDEO_EXTENSIONS);
    }
    exts
}

/// Get monitor list from the appropriate backend
pub fn get_monitor_options(backend: &str) -> Vec<String> {
    let mut monitors = vec!["All".to_string()];
    let extra = match backend {
        "hyprpaper" => get_monitors_hyprctl(),
        "swww" => get_monitors_swww(),
        "awww" => get_monitors_awww(),
        _ => get_monitors_generic(),
    };
    monitors.extend(extra);
    monitors
}

fn get_monitors_hyprctl() -> Vec<String> {
    let Ok(output) = std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
    else {
        return vec![];
    };
    let Ok(text) = std::str::from_utf8(&output.stdout) else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return vec![];
    };
    json.as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect()
}

fn get_monitors_swww() -> Vec<String> {
    // Ensure daemon is running
    let _ = std::process::Command::new("swww-daemon").spawn();
    let Ok(output) = std::process::Command::new("swww").arg("query").output() else {
        return vec![];
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            // format: "monitor: ..." for newer swww
            if let Some(idx) = l.find(':') {
                l[idx + 1..].trim().to_string()
            } else {
                l.to_string()
            }
        })
        .collect()
}

fn get_monitors_awww() -> Vec<String> {
    let _ = std::process::Command::new("awww-daemon").spawn();
    let Ok(output) = std::process::Command::new("awww").arg("query").output() else {
        return vec![];
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            if let Some(idx) = l.find(':') {
                l[idx + 1..].trim().to_string()
            } else {
                l.to_string()
            }
        })
        .collect()
}

fn get_monitors_generic() -> Vec<String> {
    // Try reading from /sys/class/drm or wlr-randr as fallback
    if let Ok(output) = std::process::Command::new("wlr-randr").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        return text
            .lines()
            .filter(|l| !l.starts_with(' ') && !l.is_empty())
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    vec![]
}

/// Check which backends are installed on this system
pub fn check_installed_backends() -> Vec<String> {
    let mut installed = vec!["none".to_string()];
    for backend in BACKEND_OPTIONS.iter().skip(1) {
        let binary = match *backend {
            "swww" => "swww",
            "awww" => "awww",
            _ => backend,
        };
        if which(binary) {
            installed.push(backend.to_string());
        }
    }
    installed
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
