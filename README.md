# PlaySoundRust

A macOS menu bar application that plays audio for relaxation and focus. Built with Rust.

## Features

- **Multiple Sound Options**
  - 40Hz sine wave tone
  - White noise
  - Pink noise
- **Volume Control**
  - Four preset levels: Low (25%), Medium (50%), High (75%), Max (100%)
  - Adjustable before playback
- **Smart UI**
  - Sound and volume selection locked during playback to prevent interruptions
  - Settings can only be changed when stopped
- **System Tray Integration**
  - Icon changes color based on playback state
    - Blue: Stopped
    - Green: Playing
- **Lightweight and Efficient**
  - Minimal resource usage
  - Native macOS integration

## Requirements

- Rust (2024 edition)
- macOS (tested on macOS)
- cargo-bundle (for packaging)

## Dependencies

- `rodio` - Audio playback and sound generation
- `tray-icon` - System tray functionality
- `image` - Icon generation
- `objc2` - macOS integration
- `rand` - Random number generation for noise

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

1. Launch the app (you'll see a circular blue icon in your menu bar)
2. Click the menu bar icon to open the menu
3. **Select Sound** - Choose from the submenu:
   - 40Hz Tone (sine wave)
   - White Noise
   - Pink Noise
4. **Volume** - Choose your preferred volume level from the submenu:
   - Low (25%)
   - Medium (50%) - default
   - High (75%)
   - Max (100%)
5. Click **Play** to start playback
   - The icon turns green
   - Sound and volume selections become disabled
6. Click **Stop** to stop playback
   - The icon turns blue
   - Sound and volume selections become available again
7. Select **Quit** to exit the application

### Tips

- Choose your sound type and volume level **before** starting playback
- You must stop playback to change sound type or volume
- The application remembers your sound and volume selection between play/stop cycles

## Sound Types Explained

### 40Hz Tone
A pure sine wave at 40Hz frequency. This low-frequency tone is often associated with relaxation and focus enhancement.

### White Noise
Random noise with equal intensity across all frequencies. Useful for:
- Masking background sounds
- Sleep and concentration
- Creating a consistent ambient environment

### Pink Noise
Random noise with more emphasis on lower frequencies (1/f noise). Compared to white noise:
- Sounds softer and more natural
- Often preferred for sleep
- Similar to the sound of rain or wind

## Configuration

To change the sine wave frequency, modify the `FREQUENCY_HZ` constant in `src/main.rs`:

```rust
const FREQUENCY_HZ: f32 = 40.0;
```

The default volume levels can be adjusted by modifying the volume multipliers in the event handlers (0.25, 0.5, 0.75, 1.0).

## Changelog

### Version 0.1.0 (Current)

**New Features:**
- Added white noise generator
- Added pink noise generator (using Paul Kellett's algorithm)
- Added volume control with 4 preset levels (25%, 50%, 75%, 100%)
- Added sound selection submenu
- Implemented UI locking during playback (prevents sound/volume changes while playing)

**Improvements:**
- Fixed sound switching bug - now properly changes to selected sound after stopping
- Sink is now properly destroyed on stop instead of paused
- Volume is applied correctly when starting new playback

**Technical:**
- Added `rand` dependency for noise generation
- Implemented thread-safe RNG using `StdRng`
- Custom white noise and pink noise source implementations

## License

This project is provided as-is for personal use.
