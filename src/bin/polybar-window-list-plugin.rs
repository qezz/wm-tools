use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

use crossbeam::channel::{unbounded, Sender};

use wm_tools::{wmctrl_d, wmctrl_l, xprop, Desktop, Window, WindowDesktop, WindowId};

fn tail_process_tick(
    sender: Sender<()>,
    command: &str,
    args: &[&str],
) -> std::thread::JoinHandle<()> {
    let cmd = command.to_string();
    let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();

    let h = thread::spawn(move || {
        let process = Command::new(&cmd)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");

        let stdout = process.stdout.expect("Failed to capture stdout");

        let reader = BufReader::new(stdout);
        for _ in reader.lines().map_while(Result::ok) {
            sender.send(()).expect("Failed to send through channel");
        }
    });

    h
}

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

    // windows.sort_by(|a, b| a.identity.0.cmp(&b.identity.0));

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

fn figure_out_windows() -> Vec<Window> {
    let xprop = xprop();
    let window_id = WindowId::from_xprop_string(&xprop).unwrap();

    let current_desktop = current_desktop();

    with_active_window(window_id.0, current_desktop.number)
}

fn cut_to_width(s: &str, max_width: usize) -> String {
    s.chars().take(max_width).collect::<String>()
}

fn windows_into_line(windows: &[Window], width_limit: usize) -> String {
    let mut line = String::new();
    let total = windows.len();
    if total == 0 {
        return String::new();
    }
    let char_limit = (width_limit / total) - 2;

    for w in windows {
        let internal = format!("{:char_limit$}", w.title);
        let internal = cut_to_width(&internal, char_limit);

        let s = if w.is_focused {
            format!("%{{F#FFFFFF}}%{{B#4C5056}} {} %{{F-}}%{{B-}}", internal)
        } else {
            format!("%{{F#777777}}%{{B#282A2E}} {} %{{F-}}%{{B-}}", internal)
        };
        line.push_str(&s);
    }
    line
}

fn update(max_width: usize) {
    let windows = figure_out_windows();
    let line = windows_into_line(&windows, max_width);

    println!("{}", line);
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let max_width: usize = args[1].parse::<usize>().unwrap();

    update(max_width);

    let (sender, receiver) = unbounded();

    let thread_handler = tail_process_tick(
        sender,
        "xev",
        &["-root", "-event", "focus", "-event", "property", "-1"],
    );

    let mut latest_update = Instant::now();

    for _ in receiver.iter() {
        // prevent bursts from triggering too many updates.
        // 16ms is 60 fps.
        if latest_update.elapsed().as_millis() < 16 {
            continue;
        }
        latest_update = Instant::now();

        update(max_width);
    }

    thread_handler.join().unwrap();
}
