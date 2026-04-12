use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(serde::Serialize)]
struct CursorState {
    x: i32,
    y: i32,
    clicked: bool, // true only on the frame the left button transitions to pressed
}

// Needed to store CheckMenuItem in managed state
struct ToggleItem(CheckMenuItem<tauri::Wry>);
unsafe impl Send for ToggleItem {}
unsafe impl Sync for ToggleItem {}

const TRAY_ICON: &[u8] = include_bytes!("../icons/32x32.png");

static ENABLED: AtomicBool = AtomicBool::new(true);
static PREV_LBUTTON: AtomicBool = AtomicBool::new(false);
static SUPPRESS_CLICKS_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Call after any tray interaction to swallow clicks for `ms` milliseconds.
fn suppress_clicks(ms: u64) {
    let until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64 + ms;
    SUPPRESS_CLICKS_UNTIL.store(until, Ordering::Relaxed);
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Transparent fullscreen overlay — starts hidden
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("overlay.html".into()),
            )
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .resizable(false)
            .build()?;

            // Click-through: mouse events pass to apps underneath
            let _ = window.set_ignore_cursor_events(true);
            resize_to_virtual_screen(&window);

            let icon = tauri::image::Image::from_bytes(TRAY_ICON)
                .expect("bundled tray icon is invalid PNG");

            // Right-click menu: enable/disable checkbox + quit
            let toggle = CheckMenuItem::with_id(app, "toggle", "Enabled\tCtrl+Shift+F1", true, true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            app.manage(ToggleItem(toggle));

            let handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("BadClaude — click to toggle whip")
                .menu(&menu)
                .show_menu_on_left_click(false) // left click = toggle, right click = menu
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "toggle" => {
                        suppress_clicks(500);
                        let enabled = app
                            .state::<ToggleItem>()
                            .0
                            .is_checked()
                            .unwrap_or(false);
                        ENABLED.store(enabled, Ordering::Relaxed);
                        apply_enabled(app, enabled);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        suppress_clicks(500);
                        let enabled = !ENABLED.fetch_xor(true, Ordering::Relaxed);
                        let _ = handle.state::<ToggleItem>().0.set_checked(enabled);
                        apply_enabled(&handle, enabled);
                    }
                })
                .build(app)?;

            // Start enabled — show overlay immediately on launch
            apply_enabled(app.handle(), true);

            // Global shortcut: Ctrl+Shift+F1 toggles the whip
            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::F1,
            );
            let gs = app.handle().global_shortcut();
            let _ = gs.unregister(shortcut.clone());
            let _ = gs.on_shortcut(shortcut, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    suppress_clicks(500);
                    let enabled = !ENABLED.fetch_xor(true, Ordering::Relaxed);
                    let _ = app.state::<ToggleItem>().0.set_checked(enabled);
                    apply_enabled(app, enabled);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![hide_overlay, get_cursor])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Show or hide the whip overlay.
fn apply_enabled(app: &AppHandle, enable: bool) {
    let Some(win) = app.get_webview_window("overlay") else {
        return;
    };
    if enable {
        let _ = win.show();
        let win2 = win.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = win2.emit("spawn-whip", ());
        });
    } else {
        let _ = win.emit("drop-whip", ());
        // Guarantee hide after animation time — JS may not complete the handshake
        let win2 = win.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            let _ = win2.hide();
        });
    }
}

/// Resize the overlay window to cover all connected monitors combined.
fn resize_to_virtual_screen(window: &tauri::WebviewWindow) {
    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(_) => {
            if let Ok(Some(m)) = window.primary_monitor() {
                let s = m.size();
                let p = m.position();
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width: s.width,
                    height: s.height,
                }));
                let _ = window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition { x: p.x, y: p.y },
                ));
            }
            return;
        }
    };

    if monitors.is_empty() {
        return;
    }

    let min_x = monitors.iter().map(|m| m.position().x).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.position().y).min().unwrap_or(0);
    let max_x = monitors
        .iter()
        .map(|m| m.position().x + m.size().width as i32)
        .max()
        .unwrap_or(1920);
    let max_y = monitors
        .iter()
        .map(|m| m.position().y + m.size().height as i32)
        .max()
        .unwrap_or(1080);

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: min_x,
        y: min_y,
    }));
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: (max_x - min_x) as u32,
        height: (max_y - min_y) as u32,
    }));
}

/// Polled by JS every frame. Returns cursor position (window-relative logical px)
/// and whether the left mouse button was just pressed this frame.
#[tauri::command]
fn get_cursor(window: tauri::WebviewWindow) -> Option<CursorState> {
    let (raw_x, raw_y) = get_raw_cursor()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    let win_pos = window
        .outer_position()
        .unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });

    // Always call detect_lclick to keep PREV_LBUTTON in sync,
    // but only report the click when the whip is enabled — prevents
    // the tray-icon click itself from triggering a crack sound.
    let lclick = detect_lclick();
    Some(CursorState {
        x: ((raw_x - win_pos.x) as f64 / scale) as i32,
        y: ((raw_y - win_pos.y) as f64 / scale) as i32,
        clicked: ENABLED.load(Ordering::Relaxed) && lclick,
    })
}

/// Called by JS after the drop animation completes to hide the window and sync state.
#[tauri::command]
fn hide_overlay(app: AppHandle) {
    ENABLED.store(false, Ordering::Relaxed);
    let _ = app.state::<ToggleItem>().0.set_checked(false);
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
}

// ── Platform implementations ─────────────────────────────────────────────────

#[cfg(windows)]
fn get_raw_cursor() -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt) != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn detect_lclick() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    let pressed = unsafe { (GetAsyncKeyState(0x01) as u16) & 0x8000 != 0 };
    let prev = PREV_LBUTTON.swap(pressed, Ordering::Relaxed);
    if !pressed || prev {
        return false; // not a new press
    }
    // Swallow clicks that belong to tray interactions
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    now >= SUPPRESS_CLICKS_UNTIL.load(Ordering::Relaxed)
}

#[cfg(target_os = "macos")]
fn get_raw_cursor() -> Option<(i32, i32)> {
    None // TODO: core-graphics
}

#[cfg(target_os = "macos")]
fn detect_lclick() -> bool {
    false // TODO
}

#[cfg(not(any(windows, target_os = "macos")))]
fn get_raw_cursor() -> Option<(i32, i32)> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
fn detect_lclick() -> bool {
    false
}
