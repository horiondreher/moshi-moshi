use nom::{
    bytes::complete::take_until,
    character::complete::{alphanumeric0, space0},
    sequence::separated_pair,
    IResult,
};

use std::collections::HashMap;
use std::str;
use std::{collections::VecDeque, fmt};

use super::{header::Header, uri::Uri};

// #[derive(Default, Debug)]
#[derive(Debug)]
pub struct ReqMessage<'a> {
    pub method: ReqMethod,
    pub direction: ReqDirection,
    pub uri: Uri<'a>,
    pub version: &'a str,
    pub headers: HashMap<String, VecDeque<Header<'a>>>,
    pub body: Option<Vec<&'a str>>,
}

impl<'a> ReqMessage<'a> {
    pub fn parse(message: &str) -> Result<ReqMessage, SipParseError> {
        // TODO: maybe change this to parse with nom too
        let mut lines: Vec<&str> = message.split("\r\n").collect();

        let uri_line = lines.remove(0);
        let (method, uri, version) = match ReqMessage::parse_request_uri(uri_line) {
            Ok((_, (method, uri, version))) => (method, uri, version),
            Err(e) => {
                println!("Error: {}", e);
                return Err(SipParseError::new(1, Some("Invalid URI")));
            }
        };

        let enum_method = match parse_sip_method(method) {
            Some(x) => x,
            None => return Err(SipParseError::new(1, Some("Invalid request message"))),
        };

        // Separates body from request checking for whitespace and draining string to
        // body_values. Also check if any position has \0 value
        let ending = lines.iter().position(|&x| x == "");
        let body_values = match ending {
            Some(end_pos) => {
                let body_lines = lines.drain(end_pos..).filter(|x| !x.starts_with("\0"));
                Some(body_lines.collect())
            }
            None => None,
        };

        let mut headers: HashMap<String, VecDeque<Header>> = HashMap::new();

        for line in &lines {
            let header = match Header::parse(line) {
                Ok((_input, header)) => header,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };

            headers
                .entry(header.name.to_owned())
                .or_default()
                .push_back(header);
        }

        Ok(ReqMessage {
            method: enum_method,
            direction: ReqDirection::In,
            uri: uri,
            version: version,
            headers: headers,
            body: body_values,
        })
    }

    /// This parser works on the first line of a SIP message
    /// It separates method (like INVITE), destination URI
    fn parse_request_uri(line: &str) -> IResult<&str, (&str, Uri, &str)> {
        let (version, (method, raw_uri)) =
            separated_pair(alphanumeric0, space0, take_until(" "))(line)?;
        let (output, uri) = Uri::parse(raw_uri)?;

        Ok((output, (method, uri, version)))
    }

    // TODO: impl From
    fn match_header_name(name: &str) -> Option<ReqMethod> {
        match name {
            "VIA" => Some(ReqMethod::Register),
            "INVITE" => Some(ReqMethod::Invite),
            "ACK" => Some(ReqMethod::Ack),
            "BYE" => Some(ReqMethod::Bye),
            "CANCEL" => Some(ReqMethod::Cancel),
            "UPDATE" => Some(ReqMethod::Update),
            "REFER" => Some(ReqMethod::Refer),
            "PRACK" => Some(ReqMethod::Prack),
            "SUBSCRIBE" => Some(ReqMethod::Subscribe),
            "NOTIFY" => Some(ReqMethod::Notify),
            "PUBLISH" => Some(ReqMethod::Publish),
            "MESSAGE" => Some(ReqMethod::Message),
            "INFO" => Some(ReqMethod::Info),
            "OPTIONS" => Some(ReqMethod::Options),
            &_ => None,
        }
    }

    pub fn get_single_header(&self, name: &str) -> Option<&Header> {
        match self.headers.get(name) {
            Some(s) => {
                if s.len() == 1 {
                    Some(&s[0])
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SipParseError<'a> {
    pub code: u32,
    pub message: Option<&'a str>,
}

impl<'a> fmt::Display for SipParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid SIP message format")
    }
}

impl<'a> SipParseError<'a> {
    pub fn new(code: u32, message: Option<&'a str>) -> SipParseError {
        SipParseError {
            code: code,
            message: message,
        }
    }
}

#[derive(Debug)]
pub enum ReqMethod {
    Register,
    Invite,
    Ack,
    Bye,
    Cancel,
    Update,
    Refer,
    Prack,
    Subscribe,
    Notify,
    Publish,
    Message,
    Info,
    Options,
}

pub enum ResType {
    Trying,
    Ringing,
    SessionProgress,
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    RequestTimeout,
    TemporarilyUnavailable,
    BusyHere,
    RequestTerminated,
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
}

#[derive(Debug)]
pub enum ReqDirection {
    In,
    Out,
}

fn parse_sip_method(method: &str) -> Option<ReqMethod> {
    match method {
        "REGISTER" => Some(ReqMethod::Register),
        "INVITE" => Some(ReqMethod::Invite),
        "ACK" => Some(ReqMethod::Ack),
        "BYE" => Some(ReqMethod::Bye),
        "CANCEL" => Some(ReqMethod::Cancel),
        "UPATE" => Some(ReqMethod::Update),
        "REFER" => Some(ReqMethod::Refer),
        "PRACK" => Some(ReqMethod::Prack),
        "SUBSCRIBE" => Some(ReqMethod::Subscribe),
        "NOTIFY" => Some(ReqMethod::Notify),
        "PUBLISH" => Some(ReqMethod::Publish),
        "MESSAGE" => Some(ReqMethod::Message),
        "INFO" => Some(ReqMethod::Info),
        "OPTIONS" => Some(ReqMethod::Options),
        &_ => None,
    }
}
