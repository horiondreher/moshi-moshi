use nom::{
    bytes::complete::{is_not, take_while},
    IResult,
};

pub fn is_any_ascii(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii())(input)
}

pub fn ascii(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii())(input)
}

pub fn not_space_or_colon(s: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n:")(s)
}
