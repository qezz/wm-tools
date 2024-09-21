use nom::{
    bytes::complete::{take_until1, take_while},
    character::complete::{digit1, space1},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct WindowToken {
    pub identity: String,
    pub desktop: String,
    pub machine_name: String,
    pub title: String,
}

pub fn window_line(input: &str) -> IResult<&str, WindowToken> {
    let (input, id) = take_until1(" ")(input)?;
    let (input, _) = space1(input)?;
    let (input, desktop) = digit1(input)?;
    let (input, _) = space1(input)?;
    let (input, machine_name) = take_until1(" ")(input)?;
    let (input, _) = space1(input)?;
    let (input, window_title) = take_while(|_| true)(input)?;

    Ok((
        input,
        WindowToken {
            identity: id.into(),
            desktop: desktop.into(),
            machine_name: machine_name.into(),
            title: window_title.into(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{Error, Window, WindowDesktop, WindowId};

    use super::*;

    #[test]
    fn parse_line_1() {
        let input =
            "0x010000ba  1 machine-name nom - crates.io: Rust Package Registry — Mozilla Firefox";

        let actual = window_line(input).unwrap().1;
        let expected = WindowToken {
            identity: "0x010000ba".into(),
            desktop: "1".into(),
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn parse_line_2() {
        let input =
            "0x010000ba  1    user@machine-name    Pattern Syntax - The Rust Programming Language — Mozilla Firefox";

        let parsed = window_line(input).unwrap();
        assert!(parsed.0.is_empty());

        let actual = parsed.1;
        let expected = WindowToken {
            identity: "0x010000ba".into(),
            desktop: "1".into(),
            machine_name: "user@machine-name".into(),
            title: "Pattern Syntax - The Rust Programming Language — Mozilla Firefox".into(),
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn parse_token_positive_desktop() {
        let input = WindowToken {
            identity: "0x010000ba".into(),
            desktop: "1".into(),
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
        };

        let actual = Window::from_token(input).unwrap();

        let id = u64::from_str_radix("010000ba", 16).unwrap();
        let expected = Window {
            identity: WindowId(id),
            desktop: WindowDesktop::Id(1),
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
            is_focused: false,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_token_desktop_is_minus_1() {
        let input = WindowToken {
            identity: "0x010000ba".into(),
            desktop: "-1".into(),
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
        };

        let actual = Window::from_token(input).unwrap();

        let id = u64::from_str_radix("010000ba", 16).unwrap();
        let expected = Window {
            identity: WindowId(id),
            desktop: WindowDesktop::StickyWindow,
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
            is_focused: false,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_token_desktop_less_than_minus_1() {
        let input = WindowToken {
            identity: "0x010000ba".into(),
            desktop: "-9".into(),
            machine_name: "machine-name".into(),
            title: "nom - crates.io: Rust Package Registry — Mozilla Firefox".into(),
        };

        let actual = Window::from_token(input).err().unwrap();
        let expected = Error::InvalidDesktopId(-9);

        assert_eq!(actual, expected);
    }
}
