# Pegasus

**Pegasus** is a companion app for the PineTime smartwatch built for Linux phones and desktops. It keeps your watch connected, forwards notifications, and provides access to health and device information—while laying the groundwork for a richer cross-device activity tracking experience.

Pegasus began as a fork of the excellent [Watchmate](https://github.com/azymohliad/watchmate) project and is evolving into a broader platform for PineTime integration across the Linux ecosystem.

---

## Features

Pegasus currently supports:

- Bluetooth connection to PineTime
- Notification forwarding from your device
- Battery level monitoring
- Step count display
- Heart-rate display
- Media playback control
- Firmware updates via `fwupd`
- Background service mode

Pegasus is designed to make PineTime feel like a natural extension of your Linux environment.

---

## Roadmap

Pegasus is growing beyond a simple companion utility. Planned directions include:

- Activity tracking across phone and desktop
- Persistent step and heart-rate history
- Exportable activity summaries
- Lightweight personal fitness logging
- Improved reconnect reliability
- Convergent interface support for Linux mobile workflows

The long-term goal is to make PineTime an active partner in your daily movement—not just a passive display.

---

## Installation (Development Build)

### Install Dependencies (Ubuntu / Kubuntu)

```bash
sudo apt install \
  build-essential \
  pkg-config \
  libdbus-1-dev \
  libgtk-4-dev \
  libadwaita-1-dev \
  meson \
  ninja-build
```

### Build Pegasus

```bash
cargo build
```

### Run Pegasus

```bash
cargo run
```

---

## Background Mode

Pegasus can run in the background to maintain the PineTime connection:

```bash
pegasus --background
```

---

## Flatpak Build (Optional)

Build locally using Flatpak:

```bash
flatpak-builder --user target/flatpak flatpak/io.github.graeus.Pegasus.yml
```

Run the Flatpak build:

```bash
flatpak-builder --run target/flatpak flatpak/io.github.graeus.Pegasus.yml pegasus
```

Install locally:

```bash
flatpak-builder --install target/flatpak flatpak/io.github.graeus.Pegasus.yml
```

---

## Project Status

Pegasus is early in its independent development.

Current priorities:

- stabilizing the renamed application identity
- preparing activity-tracking infrastructure
- improving Bluetooth reconnect reliability
- supporting convergent Linux workflows

Expect rapid iteration.

---

## Relationship to Watchmate

Pegasus builds on the foundation created by the original Watchmate project:

https://github.com/azymohliad/watchmate

Watchmate made PineTime integration on Linux practical and enjoyable. Pegasus continues that work while exploring new directions for activity tracking and cross-device workflows.

---

## Contributing

Contributions, testing feedback, and ideas are welcome.

If you are experimenting with PineTime on Linux phones, desktops, or convergent devices, your experience can directly shape Pegasus.

Open an issue or discussion:

https://github.com/GraeUS/Pegasus/issues

---

## License

Pegasus inherits the license of the upstream Watchmate project. See the `LICENSE` file for details.
