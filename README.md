# AtmoWeb TUI

A terminal UI (built with [ratatui](https://ratatui.rs)) for monitoring and controlling a memmert oven (tested with memmert un30) over the AtmoWeb web interface. It displays current and target values for temperature, flap position, and fan speed, refreshes automatically in the background, and lets you adjust target values directly from the keyboard.

## Features

- **Live overview** of current and target temperature, flap, and fan.
- **Temperature history** shown as a graph (current vs. target over time)
- **Keyboard controls** for switching between tiles and adjusting target values
- **Automatic refresh** without blocking the UI

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

### Keyboard Shortcuts

| Key            | Action                                      |
|----------------|----------------------------------------------|
| `Tab`          | Switch to the next tile                      |
| `Shift+Tab`    | Switch to the previous tile                  |
| `↑/+`            | Increase the target value of the active tile |
| `↓/-`            | Decrease the target value of the active tile |
| `Enter`        | Send the target value to the oven            |
| `q`            | Quit the application                         |

The app automatically refreshes current values and online status every 3 seconds in the background — no manual reload is needed.
