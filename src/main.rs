use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use rodio::source::SineWave;
use std::sync::{Arc, Mutex};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem, CheckMenuItem, Submenu, MenuEvent},
};
use image::{Rgba, RgbaImage};

// Constant for the tone frequency in Hz
const FREQUENCY_HZ: f32 = 40.0;
const SAMPLE_RATE: u32 = 44_100;
const PINK_NOISE_ROWS: usize = 16;

struct WhiteNoise {
    rng: StdRng,
}

impl WhiteNoise {
    fn new() -> Self {
        Self { rng: StdRng::from_entropy() }
    }
}

impl Iterator for WhiteNoise {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.rng.gen_range(-1.0..=1.0))
    }
}

impl Source for WhiteNoise {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

struct PinkNoise {
    rng: StdRng,
    rows: [f32; PINK_NOISE_ROWS],
    running_sum: f32,
    counter: u32,
}

impl PinkNoise {
    fn new() -> Self {
        let mut rng = StdRng::from_entropy();
        let mut rows = [0.0; PINK_NOISE_ROWS];
        let mut running_sum = 0.0;
        for row in rows.iter_mut() {
            *row = rng.gen_range(-1.0..=1.0);
            running_sum += *row;
        }

        Self {
            rows,
            running_sum,
            counter: 0,
            rng,
        }
    }
}

impl Iterator for PinkNoise {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.counter = self.counter.wrapping_add(1);
        let zeros = self.counter.trailing_zeros() as usize;

        if zeros < PINK_NOISE_ROWS {
            self.running_sum -= self.rows[zeros];
            self.rows[zeros] = self.rng.gen_range(-1.0..=1.0);
            self.running_sum += self.rows[zeros];
        }

        let white = self.rng.gen_range(-1.0..=1.0);
        let sample = (self.running_sum + white) / (PINK_NOISE_ROWS as f32 + 1.0);

        Some(sample)
    }
}

impl Source for PinkNoise {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

struct BrownNoise {
    rng: StdRng,
    integrator: f32,
    output: f32,
}

impl BrownNoise {
    fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            integrator: 0.0,
            output: 0.0,
        }
    }
}

impl Iterator for BrownNoise {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {

    }
}

impl Source for BrownNoise {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SoundType {
    SineWave,
    WhiteNoise,
    PinkNoise,
    BrownNoise,
}

struct AudioState {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    is_playing: bool,
    sound_type: SoundType,
    volume: f32,
}


impl AudioState {
    fn new() -> Self {
        AudioState {
            sink: None,
            _stream: None,
            is_playing: false,
            sound_type: SoundType::SineWave,
            volume: 0.5, // Default to 50% volume
        }
    }

    fn initialize_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self._stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            self._stream = Some(stream);
        }
        Ok(())
    }

    fn set_sound_type(&mut self, sound_type: SoundType) {
        self.sound_type = sound_type;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
            println!("Volume set to {}%", (self.volume * 100.0) as i32);
        }
    }

    fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize_audio()?;

        // If already playing, do nothing
        if self.is_playing {
            return Ok(());
        }

        if let Some(stream) = &self._stream {
            let sink = Sink::connect_new(stream.mixer());
            sink.set_volume(self.volume);

            match self.sound_type {
                SoundType::SineWave => {
                    let source = SineWave::new(FREQUENCY_HZ)
                        .repeat_infinite();
                    sink.append(source);
                    println!("Started playing {}Hz tone at {}% volume", FREQUENCY_HZ as i32, (self.volume * 100.0) as i32);
                }
                SoundType::WhiteNoise => {
                    let source = WhiteNoise::new()
                        .amplify(0.3) // Base amplify for white noise to prevent it being too loud
                        .repeat_infinite();
                    sink.append(source);
                    println!("Started playing white noise at {}% volume", (self.volume * 100.0) as i32);
                }
                SoundType::PinkNoise => {
                    let source = PinkNoise::new()
                        .repeat_infinite();
                    sink.append(source);
                    println!("Started playing pink noise at {}% volume", (self.volume * 100.0) as i32);
                }
                SoundType::BrownNoise => {
                    let source = BrownNoise::new()
                        .repeat_infinite();
                    sink.append(source);
                    println!("Started playing brown noise at {}% volume", (self.volume * 100.0) as i32);
                }
            }

            sink.play();
            self.sink = Some(sink);
            self.is_playing = true;
        }

        Ok(())
    }

    fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
            self.is_playing = false;
            let name = match self.sound_type {
                SoundType::SineWave => format!("{}Hz tone", FREQUENCY_HZ as i32),
                SoundType::WhiteNoise => "white noise".to_string(),
                SoundType::PinkNoise => "pink noise".to_string(),
                SoundType::BrownNoise => "brown noise".to_string(),
            };
            println!("Stopped {}", name);
        }
    }
}

fn create_icon_with_color(r: u8, g: u8, b: u8) -> tray_icon::Icon {
    let size = 32u32;
    let mut img = RgbaImage::new(size, size);

    let center = (size / 2) as i32;
    let radius = (size / 2 - 2) as i32;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - center;
            let dy = y as i32 - center;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();

            if distance <= radius as f32 {
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            } else {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    tray_icon::Icon::from_rgba(rgba, width, height)
        .expect("Failed to create icon")
}

fn create_stopped_icon() -> tray_icon::Icon {
    // Blue circle for stopped state
    create_icon_with_color(100, 149, 237)
}

fn create_playing_icon() -> tray_icon::Icon {
    // Green circle for playing state
    create_icon_with_color(76, 175, 80)
}

#[cfg(target_os = "macos")]
fn default_run_loop_mode() -> &'static objc2_foundation::NSRunLoopMode {
    // SAFETY: `NSDefaultRunLoopMode` is provided by AppKit and lives for the duration of the process.
    unsafe { objc2_foundation::NSDefaultRunLoopMode }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting macOS Audio Tray App...");

    // On macOS, we MUST initialize NSApplication BEFORE creating any tray icons
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

        let mtm = MainThreadMarker::new()
            .expect("Must run on the main thread for macOS GUI");
        let app = NSApplication::sharedApplication(mtm);

        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    
        app.finishLaunching();
      
    }

    let audio_state = Arc::new(Mutex::new(AudioState::new()));

    let menu = Menu::new();

    // Create submenu for sound selection
    let sound_menu = Submenu::new("Select Sound", true);
    let sine_item = CheckMenuItem::new(&format!("{}Hz Tone", FREQUENCY_HZ as i32), true, true, None);
    let white_noise_item = CheckMenuItem::new("White Noise", true, false, None);
    let pink_noise_item = CheckMenuItem::new("Pink Noise", true, false, None);
    let brown_noise_item = CheckMenuItem::new("Brown Noise", true, false, None);

    sound_menu.append(&sine_item)?;
    sound_menu.append(&white_noise_item)?;
    sound_menu.append(&pink_noise_item)?;
    sound_menu.append(&brown_noise_item)?;

    // Create submenu for volume selection
    let volume_menu = Submenu::new("Volume", true);
    let vol_low_item = CheckMenuItem::new("Low (25%)", true, false, None);
    let vol_medium_item = CheckMenuItem::new("Medium (50%)", true, true, None);
    let vol_high_item = CheckMenuItem::new("High (75%)", true, false, None);
    let vol_max_item = CheckMenuItem::new("Max (100%)", true, false, None);

    volume_menu.append(&vol_low_item)?;
    volume_menu.append(&vol_medium_item)?;
    volume_menu.append(&vol_high_item)?;
    volume_menu.append(&vol_max_item)?;

    let play_item = MenuItem::new("Play", true, None);
    let stop_item = MenuItem::new("Stop", false, None);
    let quit_item = MenuItem::new("Quit", true, None);

    menu.append(&sound_menu)?;
    menu.append(&volume_menu)?;
    menu.append(&play_item)?;
    menu.append(&stop_item)?;
    menu.append(&quit_item)?;

    let icon = create_stopped_icon();

    // Now it's safe to create the tray icon after NSApplication is initialized
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Audio Player - Select and play sounds")
        .with_icon(icon)
        .build()?;

    println!("Tray icon created. Look for it in your menu bar!");
    println!("Use the menu to select a sound and play it.");

    let menu_channel = MenuEvent::receiver();

    #[cfg(target_os = "macos")]
    {
        // On macOS, we need to pump the event loop
        use std::time::Duration;
        use objc2_app_kit::NSApplication;
        use objc2_foundation::MainThreadMarker;

        let mtm = MainThreadMarker::new().unwrap();
        let app = NSApplication::sharedApplication(mtm);

        // Process events in a loop
        loop {
            // Process pending macOS events
            
            use objc2_app_kit::NSEventMask;
            use objc2_foundation::NSDate;

            // Process all pending events
            while let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                    Some(&NSDate::distantPast()),
                    default_run_loop_mode(),
                true,
            ) {
                app.sendEvent(&event);
            }
            

            // Check for menu events
            if let Ok(event) = menu_channel.try_recv() {
                let event_id = event.id;

                if event_id == sine_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::SineWave);
                    sine_item.set_checked(true);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(false);
                } else if event_id == white_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::WhiteNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(true);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(false);
                } else if event_id == pink_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::PinkNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(true);
                    brown_noise_item.set_checked(false);
                } else if event_id == brown_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::BrownNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(true);
                } else if event_id == vol_low_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.25);
                    vol_low_item.set_checked(true);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_medium_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.5);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(true);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_high_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.75);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(true);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_max_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(1.0);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(true);
                } else if event_id == play_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    if let Err(e) = state.play() {
                        eprintln!("Error playing audio: {}", e);
                    } else {
                        play_item.set_enabled(false);
                        stop_item.set_enabled(true);
                        // Disable sound selection while playing
                        sine_item.set_enabled(false);
                        white_noise_item.set_enabled(false);
                        pink_noise_item.set_enabled(false);
                        brown_noise_item.set_enabled(false);
                        // Disable volume adjustment while playing
                        vol_low_item.set_enabled(false);
                        vol_medium_item.set_enabled(false);
                        vol_high_item.set_enabled(false);
                        vol_max_item.set_enabled(false);
                        tray.set_icon(Some(create_playing_icon())).ok();
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
                    play_item.set_enabled(true);
                    stop_item.set_enabled(false);
                    // Re-enable sound selection when stopped
                    sine_item.set_enabled(true);
                    white_noise_item.set_enabled(true);
                    pink_noise_item.set_enabled(true);
                    brown_noise_item.set_enabled(true);
                    // Re-enable volume adjustment when stopped
                    vol_low_item.set_enabled(true);
                    vol_medium_item.set_enabled(true);
                    vol_high_item.set_enabled(true);
                    vol_max_item.set_enabled(true);
                    tray.set_icon(Some(create_stopped_icon())).ok();
                } else if event_id == quit_item.id() {
                    println!("Quitting application...");
                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        loop {
            if let Ok(event) = menu_channel.try_recv() {
                let event_id = event.id;

                if event_id == sine_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::SineWave);
                    sine_item.set_checked(true);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(false);
                } else if event_id == white_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::WhiteNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(true);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(false);
                } else if event_id == pink_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::PinkNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(true);
                    brown_noise_item.set_checked(false);
                } else if event_id == brown_noise_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_sound_type(SoundType::BrownNoise);
                    sine_item.set_checked(false);
                    white_noise_item.set_checked(false);
                    pink_noise_item.set_checked(false);
                    brown_noise_item.set_checked(true);
                } else if event_id == vol_low_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.25);
                    vol_low_item.set_checked(true);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_medium_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.5);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(true);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_high_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(0.75);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(true);
                    vol_max_item.set_checked(false);
                } else if event_id == vol_max_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.set_volume(1.0);
                    vol_low_item.set_checked(false);
                    vol_medium_item.set_checked(false);
                    vol_high_item.set_checked(false);
                    vol_max_item.set_checked(true);
                } else if event_id == play_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    if let Err(e) = state.play() {
                        eprintln!("Error playing audio: {}", e);
                    } else {
                        play_item.set_enabled(false);
                        stop_item.set_enabled(true);
                        // Disable sound selection while playing
                        sine_item.set_enabled(false);
                        white_noise_item.set_enabled(false);
                        pink_noise_item.set_enabled(false);
                        brown_noise_item.set_enabled(false);
                        // Disable volume adjustment while playing
                        vol_low_item.set_enabled(false);
                        vol_medium_item.set_enabled(false);
                        vol_high_item.set_enabled(false);
                        vol_max_item.set_enabled(false);
                        tray.set_icon(Some(create_playing_icon())).ok();
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
                    play_item.set_enabled(true);
                    stop_item.set_enabled(false);
                    // Re-enable sound selection when stopped
                    sine_item.set_enabled(true);
                    white_noise_item.set_enabled(true);
                    pink_noise_item.set_enabled(true);
                    brown_noise_item.set_enabled(true);
                    // Re-enable volume adjustment when stopped
                    vol_low_item.set_enabled(true);
                    vol_medium_item.set_enabled(true);
                    vol_high_item.set_enabled(true);
                    vol_max_item.set_enabled(true);
                    tray.set_icon(Some(create_stopped_icon())).ok();
                } else if event_id == quit_item.id() {
                    println!("Quitting application...");
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    Ok(())
}
