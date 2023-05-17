use nom::{
    bytes::complete::take_until,
    character::complete::{alphanumeric0, digit1, space0},
    error::{Error, ErrorKind, ParseError},
    sequence::separated_pair,
    IResult,
};

use std::str;
use std::{collections::HashMap, str::FromStr};
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
    // TODO: change this to work with SipParseError FROM
    pub fn parse(message: &str) -> Result<ReqMessage, SipParseError> {
        // TODO: maybe change this to parse with nom too
        let mut lines: Vec<&str> = message.split("\r\n").collect();

        let uri_line = lines.remove(0);
        let (method, uri, version) = match ReqMessage::parse_request_uri(uri_line) {
            Ok((_, (method, uri, version))) => (method, uri, version),
            Err(e) => {
                println!("Error: {}", e);
                return Err(SipParseError::new(1, Some("Invalid URI".to_owned())));
            }
        };

        let enum_method = method.parse::<ReqMethod>()?;

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
        // TODO: fix take_until
        let (version, (method, raw_uri)) =
            separated_pair(alphanumeric0, space0, take_until(" "))(line)?;
        let (output, uri) = Uri::parse(raw_uri)?;

        Ok((output, (method, uri, version)))
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
    // TODO: change this in structs that derive a trait from header, parsing individually
    pub fn get_cseq_number(&self) -> Result<u32, SipParseError> {
        // TODO: remove this nested match
        match self.get_single_header("CSeq") {
            Some(cseq) => match Self::parse_cseq(cseq.value) {
                Ok((_, cseq_number)) => Ok(cseq_number),
                Err(e) => Err(SipParseError::new(
                    1,
                    Some("Could not parse CSeq".to_owned()),
                )),
            },
            None => Err(SipParseError::new(
                1,
                Some("CSeq header not found".to_owned()),
            )),
        }
    }

    fn parse_cseq(cseq: &str) -> IResult<&str, u32> {
        let (output, (cseq_number, _)) = separated_pair(digit1, space0, alphanumeric0)(cseq)?;

        match cseq_number.parse::<u32>() {
            Ok(x) => Ok((output, x)),
            Err(e) => Err(nom::Err::Error(Error::new(cseq, ErrorKind::Digit))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SipParseError {
    pub code: u32,
    pub message: Option<String>,
}

impl fmt::Display for SipParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.message {
            Some(ref x) => write!(f, "Code: {}, Message: {}", self.code, x),
            None => write!(f, "Code: {}, Message: None", self.code),
        }
    }
}

impl ParseError<&str> for SipParseError {
    // on one line, we show the error code and the input that caused it
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        let message = format!("{:?}:\t{:?}\n", kind, input);
        SipParseError {
            code: 1,
            message: Some(message),
        }
    }

    // if combining multiple errors, we show them one after the other
    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        let message = format!(
            "{}{:?}:\t{:?}\n",
            other.message.unwrap_or_default(),
            kind,
            input
        );
        SipParseError {
            code: 1,
            message: Some(message),
        }
    }
}

impl SipParseError {
    pub fn new(code: u32, message: Option<String>) -> SipParseError {
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

impl FromStr for ReqMethod {
    type Err = SipParseError;

    fn from_str(value: &str) -> Result<ReqMethod, Self::Err> {
        match value {
            "VIA" => Ok(ReqMethod::Register),
            "INVITE" => Ok(ReqMethod::Invite),
            "ACK" => Ok(ReqMethod::Ack),
            "BYE" => Ok(ReqMethod::Bye),
            "CANCEL" => Ok(ReqMethod::Cancel),
            "UPDATE" => Ok(ReqMethod::Update),
            "REFER" => Ok(ReqMethod::Refer),
            "PRACK" => Ok(ReqMethod::Prack),
            "SUBSCRIBE" => Ok(ReqMethod::Subscribe),
            "NOTIFY" => Ok(ReqMethod::Notify),
            "PUBLISH" => Ok(ReqMethod::Publish),
            "MESSAGE" => Ok(ReqMethod::Message),
            "INFO" => Ok(ReqMethod::Info),
            "OPTIONS" => Ok(ReqMethod::Options),
            &_ => Err(SipParseError {
                code: 2,
                message: Some("Invalid Request Method".to_owned()),
            }),
        }
    }
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

impl fmt::Display for ResType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResType::Trying => write!(f, "100 Trying"),
            ResType::Ringing => write!(f, "180 Ringing"),
            ResType::SessionProgress => write!(f, "183 Session Progress"),
            ResType::Ok => write!(f, "200 OK"),
            ResType::BadRequest => write!(f, "400 Bad Request"),
            ResType::Unauthorized => write!(f, "401 Unauthorized"),
            ResType::Forbidden => write!(f, "403 Forbidden"),
            ResType::NotFound => write!(f, "404 Not Found"),
            ResType::RequestTimeout => write!(f, "408 Request Timeout"),
            ResType::TemporarilyUnavailable => write!(f, "480 Temporarily Unavailable"),
            ResType::BusyHere => write!(f, "486 Busy Here"),
            ResType::RequestTerminated => write!(f, "487 Request Terminated"),
            ResType::InternalServerError => write!(f, "500 Internal Server Error"),
            ResType::BadGateway => write!(f, "502 Bad Gateway"),
            ResType::ServiceUnavailable => write!(f, "503 Service Unavailable"),
        }
    }
}

#[derive(Debug)]
pub enum ReqDirection {
    In,
    Out,
}
