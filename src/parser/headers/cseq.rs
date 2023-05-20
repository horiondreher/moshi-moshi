use std::str::FromStr;

use nom::{
    character::complete::{alphanumeric0, digit1, space0},
    error::{Error, ErrorKind},
    sequence::separated_pair,
    IResult,
};

use crate::parser::SipParseError;

pub struct CSeq {
    pub number: u32,
    pub method: String,
}

impl CSeq {
    // TODO: make this into a trait
    fn parse(cseq: &str) -> IResult<&str, (u32, &str)> {
        let (output, (cseq_number, method)) = separated_pair(digit1, space0, alphanumeric0)(cseq)?;

        match cseq_number.parse::<u32>() {
            Ok(num) => Ok((output, (num, method))),
            Err(_e) => Err(nom::Err::Error(Error::new(cseq, ErrorKind::Digit))),
        }
    }
}

impl FromStr for CSeq {
    type Err = SipParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match Self::parse(value) {
            Ok((_, cseq)) => Ok(CSeq {
                number: cseq.0,
                method: cseq.1.to_owned(),
            }),
            Err(_e) => Err(SipParseError::new(
                1,
                Some("Could not parse CSeq".to_owned()),
            )),
        }
    }
}
