use nom::{
    bytes::complete::{is_a, tag, take_until1, take_while},
    character::complete::space1,
    IResult,
};

/// See `man wmctrl` for details.
#[derive(Debug, PartialEq)]
pub struct DesktopToken {
    /// Desktop number
    pub number: String,
    /// Desktop marker: `*` for current, `-` for everything else
    pub marker: String,
    /// DG - Desktop Geometry
    pub desktop_geometry: String,
    /// VP - Viewport Position
    pub viewport_position: String,
    /// WA - Workarea geometry
    pub workarea: String,
    /// Name of a desktop
    pub name: String,
}

pub fn desktop_line(input: &str) -> IResult<&str, DesktopToken> {
    let (input, number) = take_until1(" ")(input)?;
    let (input, _) = space1(input)?;
    let (input, marker) = is_a("*-")(input)?;
    let (input, _) = space1(input)?;

    let (input, _) = tag("DG:")(input)?;
    let (input, _) = space1(input)?;
    let (input, dg) = take_until1(" ")(input)?;
    let (input, _) = space1(input)?;

    let (input, _) = tag("VP:")(input)?;
    let (input, _) = space1(input)?;
    let (input, vp) = take_until1(" ")(input)?;
    let (input, _) = space1(input)?;

    let (input, _) = tag("WA:")(input)?;
    let (input, _) = space1(input)?;
    let (input, wa) = take_until1(" ")(input)?;

    let (input, _) = space1(input)?;
    let (input, name) = take_while(|_| true)(input)?;

    Ok((
        input,
        DesktopToken {
            number: number.into(),
            marker: marker.into(),
            desktop_geometry: dg.into(),
            viewport_position: vp.into(),
            workarea: wa.into(),
            name: name.into(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_1() {
        let input = "0  - DG: N/A  VP: N/A  WA: N/A  1";

        let actual = desktop_line(input).unwrap().1;
        let expected = DesktopToken {
            number: "0".into(),
            marker: "-".into(),
            desktop_geometry: "N/A".into(),
            viewport_position: "N/A".into(),
            workarea: "N/A".into(),
            name: "1".into(),
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn parse_line_2() {
        let input = "2  * DG: N/A  VP: N/A  WA: N/A  3";

        let actual = desktop_line(input).unwrap().1;
        let expected = DesktopToken {
            number: "2".into(),
            marker: "*".into(),
            desktop_geometry: "N/A".into(),
            viewport_position: "N/A".into(),
            workarea: "N/A".into(),
            name: "3".into(),
        };

        assert_eq!(actual, expected)
    }
}
