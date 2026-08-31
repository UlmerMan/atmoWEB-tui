# AtmoWeb TUI

A terminal UI (built with [ratatui](https://ratatui.rs)) for monitoring and controlling a memmert oven (tested with memmert un30) over the AtmoWeb web interface. It displays current and target values for temperature, flap position, and fan speed, refreshes automatically in the background, and lets you adjust target values directly from the keyboard.

## Features

- **Live overview** of current and target temperature, flap, and fan.
- **Heating curve editor** to visually configure multi-stage temperature profiles.
- **Non-blocking execution engine** to run programmed curves against the oven in real-time.
- **Live temperature history** graph (toggleable with `g`).
- **Keyboard-only controls** for easy and fast operation.
- **Automatic refresh** in the background.

## Operating Modes

- **Manual Mode** (`[ MANUAL MODE ]`): Direct keyboard control over Temperature, Flap, and Fan setpoints with live temperature history graph.
- **Auto Mode** (`[ AUTO MODE - HEATING CURVE ]`): Programmed multi-stage heating curve drives the stove temperature setpoints via non-blocking runner. Manual controls are **locked and read-only** to prevent accidental override.

Press **`m`** at any time to toggle between Manual and Auto modes.

## Requirements

- Rust (current stable version, see [rustup.rs](https://rustup.rs))
- An AtmoWeb-capable oven reachable on the local network (via IP address)

## Installation

```bash
git clone <repo-url>
cd atmoweb-tui
cargo build --release
```

## Usage

```bash
cargo run -- --address <OVEN-IP-ADDRESS>
```

Example:

```bash
cargo run -- --address 192.168.1.50
```

### Global Shortcuts

| Key                 | Action                                              |
|---------------------|-----------------------------------------------------|
| `m`                 | Toggle between **Manual Mode** and **Auto Mode**    |
| `g`                 | Toggle right panel (Heating Curve / Live History)   |
| `e`                 | Enter exact numerical float value                   |
| `q`                 | Quit the application                                |

### Manual Mode Controls

| Key                 | Action                                              |
|---------------------|-----------------------------------------------------|
| `Tab` / `Shift+Tab` | Cycle focus between Temperature, Flap, and Fan       |
| `0` / `1` / `2`     | Direct jump to Temperature, Flap, or Fan tile       |
| `↑` / `→` / `+`     | Increase active target value                        |
| `↓` / `←` / `-`     | Decrease active target value                        |
| `Enter`             | Send active target value to oven                    |

### Auto Mode Controls (Heating Curve)

| Key                 | Action                                              |
|---------------------|-----------------------------------------------------|
| `←` / `→` (or `h`/`l`) | Select previous / next curve point               |
| `↑` / `↓` (or `k`/`j`) | Increase / decrease target temperature (±5 °C)   |
| `+` / `-` (or `]`/`[`) | Increase / decrease segment duration (±5 min)    |
| `a` / `n` / `Insert`| Add new curve point after selected                  |
| `d` / `x` / `Delete`| Delete selected curve point                         |
| `Space` / `r`       | Start / Pause heating curve execution               |
| `s`                 | Stop heating curve execution                        |


