pub mod parser;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    StdIo(String),
    Utf8(std::string::FromUtf8Error),
    ParseDesktopId(String),
    InvalidDesktopId(i32),
    Nom(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StdIo(e.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseDesktopId(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl<E: std::fmt::Debug> From<nom::Err<E>> for Error {
    fn from(e: nom::Err<E>) -> Self {
        Error::Nom(format!("{:?}", e))
    }
}

pub fn cmd(s: &str) -> Result<String, Error> {
    let output = std::process::Command::new("sh").arg("-c").arg(s).output()?;

    let s = String::from_utf8(output.stdout)?;
    let s = s.trim().to_string();

    Ok(s)
}

pub fn wmctrl_l() -> String {
    cmd("wmctrl -l").unwrap()
}

pub fn wmctrl_d() -> String {
    cmd("wmctrl -d").unwrap()
}

pub fn xprop() -> String {
    cmd("xprop -root | grep '_NET_ACTIVE_WINDOW(WINDOW)'").unwrap()
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WindowId(pub u64);

impl WindowId {
    fn from_hex(s: &str) -> Result<Self, Error> {
        let no_0x = s.trim_start_matches("0x");
        let parsed = u64::from_str_radix(no_0x, 16)?;

        Ok(WindowId(parsed))
    }

    pub fn from_xprop_string(input: &str) -> Result<Self, Error> {
        let token = parser::xprop_line(input)?.1;

        Self::from_hex(&token)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WindowDesktop {
    Id(u32), // 0 and higher
    StickyWindow,  // -1
}

#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub identity: WindowId,
    pub desktop: WindowDesktop,
    pub machine_name: String,
    pub title: String,

    pub is_focused: bool,
}

impl Window {
    pub fn from_string(input: &str) -> Result<Self, Error> {
        let token = parser::window_line(input)?.1;

        Self::from_token(token)
    }

    pub fn from_token(token: parser::WindowToken) -> Result<Self, Error> {
        let desktop_dec: i32 = token.desktop.parse()?;
        let desktop = match desktop_dec {
            0.. => WindowDesktop::Id(desktop_dec as u32),
            -1 => WindowDesktop::StickyWindow,
            _ => return Err(Error::InvalidDesktopId(desktop_dec)),
        };

        Ok(Window {
            identity: WindowId::from_hex(&token.identity)?,
            desktop,
            machine_name: token.machine_name,
            title: token.title,
            is_focused: false,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Desktop {
    pub number: u32,
    pub is_current: bool,
    pub name: String,
}

impl Desktop {
    pub fn from_string(input: &str) -> Result<Self, Error> {
        let token = parser::desktop_line(input)?.1;

        Self::from_token(token)
    }

    pub fn from_token(token: parser::DesktopToken) -> Result<Self, Error> {
        let number: u32 = token.number.parse()?;
        let is_current = token.marker == "*";

        Ok(Desktop {
            number,
            is_current,
            name: token.name,
        })
    }
}

pub struct State {
    pub current_desktop: i32,
    pub all_windows: Vec<Window>,
    pub this_desktop_windows: Vec<Window>,
    pub sticky_windows: Vec<Window>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_window_end_to_end() {
        let input =
            "0x010000ba  1    user@machine-name    Pattern Syntax - The Rust Programming Language — Mozilla Firefox";

        let actual = Window::from_string(input).unwrap();

        let expected = Window {
            identity: WindowId::from_hex("0x010000ba").unwrap(),
            desktop: WindowDesktop::Id(1),
            machine_name: "user@machine-name".into(),
            title: "Pattern Syntax - The Rust Programming Language — Mozilla Firefox".into(),
            is_focused: false,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_desktop_end_to_end() {
        let input = "2  * DG: N/A  VP: N/A  WA: N/A  3";

        let actual = Desktop::from_string(input).unwrap();

        let expected = Desktop {
            number: 2,
            is_current: true,
            name: "3".into(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_xprop_end_to_end() {
        let input = "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x1e00003";

        let actual = WindowId::from_xprop_string(input).unwrap();
        println!("{:?}", actual);
        let expected = WindowId::from_hex("0x1e00003").unwrap();

        assert_eq!(actual, expected);
    }
}
