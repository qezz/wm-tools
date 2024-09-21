use nom::{
    bytes::complete::{tag, take_while},
    character::complete::space1,
    IResult,
};

pub fn xprop_line(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("_NET_ACTIVE_WINDOW(WINDOW):")(input)?;
    let (input, _) = space1(input)?;

    let (input, _) = tag("window")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("id")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("#")(input)?;
    let (input, _) = space1(input)?;

    let (input, window_identifier) = take_while(|_| true)(input)?;

    Ok((input, window_identifier.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line() {
        let input = "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x1e00003";

        let actual = xprop_line(input).unwrap().1;
        let expected = String::from("0x1e00003");

        assert_eq!(actual, expected)
    }
}
