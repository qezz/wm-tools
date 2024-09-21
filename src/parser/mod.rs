pub mod desktop;
pub mod window;
pub mod xprop_current_window;

pub use desktop::{desktop_line, DesktopToken};
pub use window::{window_line, WindowToken};
pub use xprop_current_window::xprop_line;
