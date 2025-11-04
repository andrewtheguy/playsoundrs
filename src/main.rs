use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use rodio::source::SineWave;
use std::sync::{Arc, Mutex};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem, MenuEvent},
};
use image::{Rgba, RgbaImage};

struct AudioState {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
}

impl AudioState {
    fn new() -> Self {
        AudioState {
            sink: None,
            _stream: None,
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
                println!("Resumed 200Hz tone");
                return Ok(());
            }
        }

        if let Some(stream) = &self._stream {
            let sink = Sink::connect_new(stream.mixer());

            let source = SineWave::new(200.0)
                .amplify(0.20)
                .repeat_infinite();

            sink.append(source);
            sink.play();

            self.sink = Some(sink);
            println!("Started playing 200Hz tone");
        }

        Ok(())
    }

    fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            println!("Stopped 200Hz tone");
        }
    }
}

fn create_icon() -> tray_icon::Icon {
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
                img.put_pixel(x, y, Rgba([100, 149, 237, 255]));
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

        // Set activation policy to accessory so app doesn't show in Dock
        unsafe {
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }

        // This is CRITICAL - finish launching the app before creating tray icons
        unsafe {
            app.finishLaunching();
        }
    }

    let audio_state = Arc::new(Mutex::new(AudioState::new()));

    let menu = Menu::new();
    let play_item = MenuItem::new("Play 200Hz Tone", true, None);
    let stop_item = MenuItem::new("Stop", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    menu.append(&play_item)?;
    menu.append(&stop_item)?;
    menu.append(&quit_item)?;

    let icon = create_icon();

    // Now it's safe to create the tray icon after NSApplication is initialized
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Audio Player - 200Hz Tone")
        .with_icon(icon)
        .build()?;

    println!("Tray icon created. Look for it in your menu bar!");
    println!("Use the menu to Play or Stop the 200Hz tone.");

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
            unsafe {
                use objc2_app_kit::NSEventMask;
                use objc2_foundation::{NSDate, NSDefaultRunLoopMode};

                // Process all pending events
                while let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                    NSEventMask::Any,
                    Some(&NSDate::distantPast()),
                    NSDefaultRunLoopMode,
                    true,
                ) {
                    app.sendEvent(&event);
                }
            }

            // Check for menu events
            if let Ok(event) = menu_channel.try_recv() {
                let event_id = event.id;

                if event_id == play_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    if let Err(e) = state.play() {
                        eprintln!("Error playing audio: {}", e);
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
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
                    }
                } else if event_id == stop_item.id() {
                    let mut state = audio_state.lock().unwrap();
                    state.stop();
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
