# PlaySoundRust

A macOS menu bar application that plays a continuous 40Hz tone. Built with Rust.

## Features

- System tray icon that changes color based on playback state
  - Blue: Stopped
  - Green: Playing
- Simple menu interface to control playback
- Plays a continuous 40Hz sine wave tone
- Lightweight and efficient

## Requirements

- Rust (2024 edition)
- macOS (tested on macOS)
- cargo-bundle (for packaging)

## Dependencies

- `rodio` - Audio playback
- `tray-icon` - System tray functionality
- `image` - Icon generation
- `objc2` - macOS integration

## Building

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Running

```bash
cargo run --release
```

## Packaging as macOS App

### Install cargo-bundle

```bash
cargo install cargo-bundle
```

### Build the .app bundle

```bash
cargo bundle --release
```

The app will be created at:
```
target/release/bundle/osx/PlaySoundRust.app
```

### Install to Applications

```bash
cp -r target/release/bundle/osx/PlaySoundRust.app /Applications/
```

### Test the app

```bash
open target/release/bundle/osx/PlaySoundRust.app
```

### Distribute

To create a distributable zip file:

```bash
cd target/release/bundle/osx
zip -r PlaySoundRust.zip PlaySoundRust.app
```

## Usage

1. Launch the app (you'll see a circular icon in your menu bar)
2. Click the menu bar icon
3. Select "Play 40Hz Tone" to start playback
4. Select "Stop" to pause playback
5. Select "Quit" to exit the application

## Configuration

To change the frequency, modify the `FREQUENCY_HZ` constant in `src/main.rs`:

```rust
const FREQUENCY_HZ: f32 = 40.0;
```

## License

This project is provided as-is for personal use.
