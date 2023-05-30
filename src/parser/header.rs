use super::{
    common::sip_chars::{ascii, not_space_or_colon},
    param::Param,
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    sequence::{separated_pair, tuple},
    IResult,
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub params: Option<BTreeMap<&'a str, Param<'a>>>,
}

impl<'a> Header<'a> {
    /// This parser break apart Header from Values using separated_pair
    /// Example - Via: SIP/2.0/UDP 192.168.1.146:5070;branch=z9hG4bK-10149-1-0
    ///
    /// The first parameter is a parser that gets the values until its not space or color (as the name implies)
    /// but the reason of it is because: "The formal grammar for a message-header specified in Section 25
    /// allows for an arbitrary amount of whitespace on either side of the colon;" (RFC 3261)
    ///
    /// The second parameter is a parser that gets the caracters that will be discarded,
    /// so it can be zero or more spaces, a colon, and zero or more spaces again
    ///
    /// The third paramter gets any ascii caracter until the end of the line
    pub fn parse(line: &'a str) -> IResult<&str, Self> {
        let header_value =
            separated_pair(not_space_or_colon, tuple((space0, tag(":"), space0)), ascii);

        // Transforms the parser into a Header struct
        map(header_value, |(header, value)| Header {
            name: header,
            value,
            params: None,
        })(line)
    }
}
