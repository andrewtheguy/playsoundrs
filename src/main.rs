use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use rodio::source::SineWave;
use std::sync::{Arc, Mutex};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem, CheckMenuItem, MenuEvent},
};
use image::{Rgba, RgbaImage};

// Constant for the tone frequency in Hz
const FREQUENCY_HZ: f32 = 40.0;

struct AudioState {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    is_playing: bool,
}

impl AudioState {
    fn new() -> Self {
        AudioState {
            sink: None,
            _stream: None,
            is_playing: false,
        }
    }

    fn initialize_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self._stream.is_none() {
            let stream = OutputStreamBuilder::open_default_stream()?;
            self._stream = Some(stream);
        }
        Ok(())
    }

    fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize_audio()?;

        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                self.is_playing = true;
                println!("Resumed {}Hz tone", FREQUENCY_HZ as i32);
                return Ok(());
            }
        }

        if let Some(stream) = &self._stream {
            let sink = Sink::connect_new(stream.mixer());

            let source = SineWave::new(FREQUENCY_HZ)
                .amplify(0.5)
                .repeat_infinite();

            sink.append(source);
            sink.play();

            self.sink = Some(sink);
            self.is_playing = true;
            println!("Started playing {}Hz tone", FREQUENCY_HZ as i32);
        }

        Ok(())
    }

    fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.is_playing = false;
            println!("Stopped {}Hz tone", FREQUENCY_HZ as i32);
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
    let play_item = CheckMenuItem::new(&format!("Play {}Hz Tone", FREQUENCY_HZ as i32), true, false, None);
    let stop_item = MenuItem::new("Stop", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    // Initially, disable "Stop" since we're not playing
    stop_item.set_enabled(false);

    menu.append(&play_item)?;
    menu.append(&stop_item)?;
    menu.append(&quit_item)?;

    let icon = create_stopped_icon();

    // Now it's safe to create the tray icon after NSApplication is initialized
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(&format!("Audio Player - {}Hz Tone", FREQUENCY_HZ as i32))
        .with_icon(icon)
        .build()?;

    println!("Tray icon created. Look for it in your menu bar!");
    println!("Use the menu to Play or Stop the {}Hz tone.", FREQUENCY_HZ as i32);

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

                if event_id == play_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    if let Err(e) = state.play() {
                        eprintln!("Error playing audio: {}", e);
                    } else {
                        // Update menu items: disable Play, enable Stop, show checkmark
                        play_item.set_enabled(false);
                        play_item.set_checked(true);
                        stop_item.set_enabled(true);
                        // Change tray icon to green (playing)
                        tray.set_icon(Some(create_playing_icon())).ok();
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
                    // Update menu items: enable Play, disable Stop, remove checkmark
                    play_item.set_enabled(true);
                    play_item.set_checked(false);
                    stop_item.set_enabled(false);
                    // Change tray icon to blue (stopped)
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

                if event_id == play_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    if let Err(e) = state.play() {
                        eprintln!("Error playing audio: {}", e);
                    } else {
                        // Update menu items: disable Play, enable Stop, show checkmark
                        play_item.set_enabled(false);
                        play_item.set_checked(true);
                        stop_item.set_enabled(true);
                        // Change tray icon to green (playing)
                        tray.set_icon(Some(create_playing_icon())).ok();
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
                    // Update menu items: enable Play, disable Stop, remove checkmark
                    play_item.set_enabled(true);
                    play_item.set_checked(false);
                    stop_item.set_enabled(false);
                    // Change tray icon to blue (stopped)
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
