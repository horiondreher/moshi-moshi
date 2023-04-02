use std::collections::BTreeMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric0, char, digit0},
    sequence::separated_pair,
    IResult,
};

use super::{common::sip_chars::is_any_ascii, param::Param};

pub struct Uri<'a> {
    display_name: Option<&'a str>,
    scheme: &'a str,
    user: &'a str,
    host: &'a str,
    port: u16,
    params: Option<BTreeMap<&'a str, Param<'a>>>,
}

impl<'a> Uri<'a> {
    /// First this function split the scheme part of URI (sip or sips) using alt, if fails for sip it tries for sips
    /// Second it uses separated_pair to break apart the user from host and port, using '@' for it
    /// Finally split the host and port to store in variables
    /// TODO: Make it work for ipv6 too
    pub fn parse(input: &'a str) -> IResult<&str, Self> {
        let (input, scheme) = alt((tag("sip:"), tag("sips:")))(input)?;
        let (_params, (user, host_port)) =
            separated_pair(alphanumeric0, char('@'), is_any_ascii)(input)?;
        let (output, (host, port)) = separated_pair(take_until(":"), char(':'), digit0)(host_port)?;

        Ok((
            output,
            Uri {
                display_name: None,
                user,
                scheme,
                host: host,
                port: port.parse::<u16>().unwrap(),
                params: None,
            },
        ))
    }
}
