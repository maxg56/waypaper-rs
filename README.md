# waypaper-rs

GUI wallpaper setter for Wayland and Xorg — a Rust rewrite of [waypaper](https://github.com/anufrievroman/waypaper).

Works as a frontend for popular wallpaper backends: `swaybg`, `swww`, `awww`, `hyprpaper`, `mpvpaper`, `feh`, `xwallpaper`, and `wallutils`.

## Features

- Image grid with cached thumbnails
- Search / filter by filename
- Sort by name, date, or random
- Vim-style keyboard navigation (`h` `j` `k` `l`)
- Multi-monitor support
- Fill modes: fill, stretch, fit, center, tile
- Background color picker
- swww/awww transition options (type, step, angle, duration, fps)
- mpvpaper controls (pause, stop, sound toggle)
- Include subfolders and hidden files
- Zen mode (hide UI, navigate with keyboard only)
- Thumbnail cache (XDG cache dir)
- Config compatible with the original waypaper (`~/.config/waypaper/config.ini`)
- Post-command execution (`$wallpaper`, `$monitor` variables)
- CLI for scripting and WM startup

## Installation

### Prerequisites

Install at least one wallpaper backend:

```bash
# Arch
pacman -S swaybg        # simple Wayland background
pacman -S swww          # animated transitions
pacman -S hyprpaper     # Hyprland native
pacman -S feh           # X11
yay -S awww             # swww fork with GIF support
yay -S mpvpaper         # video wallpapers
```

### Build from source

Requires Rust 1.70+ (`rustup` recommended).

```bash
git clone https://github.com/yourname/waypaper-rs
cd waypaper-rs
cargo build --release
```

The binary is at `target/release/waypaper-rs`. Copy it to your `$PATH`:

```bash
install -m755 target/release/waypaper-rs ~/.local/bin/
```

## Usage

```bash
# Open the GUI
waypaper-rs

# Restore previous wallpaper(s) — add to WM startup
waypaper-rs --restore

# Set a random wallpaper
waypaper-rs --random

# Set a specific wallpaper
waypaper-rs --wallpaper ~/Pictures/my-wallpaper.jpg

# Set on a specific monitor
waypaper-rs --monitor DP-1 --wallpaper ~/Pictures/my-wallpaper.jpg

# Print current state as JSON
waypaper-rs --list

# Override backend and folder
waypaper-rs --backend swww --folder ~/Pictures/landscapes
```

### All CLI options

| Option | Description |
|---|---|
| `--restore` | Restore the previously saved wallpaper(s) |
| `--random` | Set a random wallpaper from the configured folder |
| `--wallpaper <PATH>` | Set a specific image as wallpaper |
| `--fill <MODE>` | Fill mode: `fill`, `stretch`, `fit`, `center`, `tile` |
| `--folder <PATH>...` | Override image folder(s) |
| `--backend <NAME>` | Override backend |
| `--monitor <NAME>` | Target a specific monitor |
| `--config-file <PATH>` | Use an alternative config file |
| `--state-file <PATH>` | Use an alternative state file |
| `--no-post-command` | Skip the post-command execution |
| `--list` | Print monitor/wallpaper state as JSON and exit |

## Keyboard shortcuts

| Key | Action |
|---|---|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `k` / `↑` | Move up |
| `j` / `↓` | Move down |
| `g` | Go to first image |
| `Enter` | Set selected image as wallpaper |
| `r` | Clear cache and reload |
| `f` | Open folder picker |
| `s` | Toggle subfolders |
| `h` | Toggle hidden files |
| `z` | Toggle zen mode |
| `q` | Quit |

## Configuration

Config is stored at `~/.config/waypaper/config.ini` — the same location and format as the original waypaper, so existing configs are fully compatible.

```ini
[settings]
backend = swww
folder = ~/Pictures
fill = fill
color = #ffffff
sort = name
number_of_columns = 3
subfolders = False
show_hidden = False
show_gifs_only = False
zen_mode = False
show_path_in_tooltip = True
post_command =
swww_transition_type = any
swww_transition_step = 63
swww_transition_angle = 0
swww_transition_duration = 2.0
swww_transition_fps = 60
mpvpaper_sound = False
mpvpaper_options =
monitors = All
wallpaper = ~/Pictures/my-wallpaper.jpg
```

### Restore wallpaper on startup

Add to your WM config (Hyprland, sway, i3, etc.):

```bash
exec waypaper-rs --restore
```

## Supported backends

| Backend | Wayland | X11 | GIF | Video |
|---|:---:|:---:|:---:|:---:|
| swaybg | ✅ | | | |
| swww | ✅ | | ✅ | |
| awww | ✅ | | ✅ | |
| hyprpaper | ✅ | | | |
| mpvpaper | ✅ | | ✅ | ✅ |
| feh | | ✅ | | |
| xwallpaper | | ✅ | | |
| wallutils | ✅ | ✅ | | |

## Project structure

```
src/
├── main.rs      Entry point and CLI argument handling
├── app.rs       egui GUI application
├── config.rs    Config read/write (INI format)
├── changer.rs   Wallpaper backend implementations
├── common.rs    Image discovery and thumbnail caching
└── options.rs   Constants: backends, fill modes, sort options
```

## Differences from the original waypaper

- Written in Rust — single static binary, no Python runtime required
- GUI uses [egui](https://github.com/emilk/egui) instead of GTK3
- Thumbnail cache uses a hash-based filename scheme
- No translation system (English only for now)
- No daemon mode (`waypaperd`)

## License

GPL-3.0 — same as the original waypaper.
