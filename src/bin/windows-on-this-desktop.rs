use wm_tools::{wmctrl_d, wmctrl_l, xprop, Desktop, Window, WindowDesktop, WindowId};

fn with_active_window(window_id: u64, desktop_id: u32) -> Vec<Window> {
    let output = wmctrl_l();
    let mut windows: Vec<Window> = Vec::new();
    for line in output.lines() {
        let window = Window::from_string(line).unwrap();
        if window.desktop == WindowDesktop::Id(desktop_id) {
            let w = Window {
                is_focused: window.identity.0 == window_id,
                ..window
            };
            windows.push(w);
        }
    }
    windows
}

fn current_desktop() -> Desktop {
    let output = wmctrl_d();

    for line in output.lines() {
        let desktop = Desktop::from_string(line).unwrap();
        if desktop.is_current {
            return desktop;
        }
    }

    panic!("No current desktop found");
}

fn main() {
    let xprop = xprop();
    let window_id = WindowId::from_xprop_string(&xprop).unwrap();

    let current_desktop = current_desktop();

    println!("Windows on the current desktop:");
    let windows = with_active_window(window_id.0, current_desktop.number);
    for w in windows {
        println!("  {:?}", w);
    }
}
