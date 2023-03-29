use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while},
    character::{
        complete::{alphanumeric0, char, digit0, space0},
    },
    combinator::map,
    error::{Error, ErrorKind, ParseError},
    sequence::{delimited, separated_pair, tuple},
    IResult, Parser,
};

#[derive(Debug, Clone)]
struct SipParseError;

impl fmt::Display for SipParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid SIP message format")
    }
}

use std::net::{SocketAddr, ToSocketAddrs};
use std::str;
use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
};
use std::{env, net};

#[derive(Debug)]
enum ReqMethod {
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

enum ResType {
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

enum ReqDirection {
    In,
    Out,
}

pub fn is_any_ascii(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii())(input)
}

pub fn ascii(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii())(input)
}

fn not_space_or_colon(s: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n:")(s)
}

struct Uri<'a> {
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
    fn parse(input: &'a str) -> IResult<&str, Self> {
        let (input, scheme) = alt((tag("sip:"), tag("sips:")))(input)?;
        let (_params, (user, host_port)) = separated_pair(alphanumeric0, char('@'), is_any_ascii)(input)?;
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
        &_ => None
    }
}

struct Param<'a> {
    name: &'a str,
    value: &'a str,
}

struct Header<'a> {
    name: &'a str,
    value: &'a str,
    params: Option<BTreeMap<&'a str, Param<'a>>>,
}

struct Headers<'a> {
    via: VecDeque<Header<'a>>,
    max_forwards: Header<'a>,
    from: Header<'a>,
    to: Header<'a>,
    call_id: Header<'a>,
    cseq: Header<'a>,
    contact: Header<'a>,
    content_type: Header<'a>,
    content_length: Header<'a>,
}

// #[derive(Default, Debug)]
// #[derive(Debug)]
struct ReqMessage<'a> {
    method: ReqMethod,
    direction: ReqDirection,
    uri: Uri<'a>,
    version: &'a str,
    headers: Option<Headers<'a>>,
    body: Option<Vec<&'a str>>,
}

impl<'a> ReqMessage<'a> {
    fn parse(message: &str) -> Result<ReqMessage, SipParseError> {
        let mut lines: Vec<&str> = message.split("\r\n").collect();
        let mut headers: Headers;

        let uri_line = lines.remove(0);
        let (method, uri, version) = match ReqMessage::parse_request_uri(uri_line) {
            Ok((_, (method, uri, version))) => (method, uri, version),
            Err(e) => {
                println!("Error: {}", e);
                return Err(SipParseError);
            }
        };

        let enum_method = match parse_sip_method(method) {
            Some(x) => x,
            None => return Err(SipParseError)
        };

        let ending =  lines.iter().position(|&x| x == "");
        let body_values = match ending {
            Some(end_pos) => Some(lines.drain(end_pos..).collect()),
            None => None
        };

        for line in &lines {
            let (_input, header) = match ReqMessage::parse_header(line) {
                Ok((_input, header)) => {
                    (_input, header)
                }
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };
        }

        Ok(ReqMessage {
            method: enum_method,
            direction: ReqDirection::In,
            uri: uri,
            version: version,
            headers: None,
            body: body_values,
        })
    }

    /// This parser works on the first line of a SIP message 
    /// It separates method (like INVITE), destination URI 
    fn parse_request_uri(line: &str) -> IResult<&str, (&str, Uri, &str)> {
        let (version, (method, raw_uri)) = separated_pair(alphanumeric0, space0, take_until(" "))(line)?;
        let (output, uri) = Uri::parse(raw_uri)?;

        Ok((output, (method, uri, version)))
    }

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
    fn parse_header(line: &str) -> IResult<&str, Header> {
        let header_value = separated_pair(not_space_or_colon, tuple((space0, tag(":"), space0)), ascii);

        // Transforms the parser into a Header struct
        map(header_value, |(header, value)| Header {
            name: header,
            value: value,
            params: None,
        })(line)
    }
}
fn main() {
    let host_addr = env::args().nth(1).expect("Invalid host IP address");
    let port = env::args().nth(2).expect("Invalid port");
    let addrs = format!("{}:{}", host_addr, port);

    let mut buf = vec![0; 2048];

    println!("Bindind socket on {}...", addrs);
    let socket = match net::UdpSocket::bind(&addrs) {
        Ok(s) => s,
        Err(e) => panic!("Could not bind to socket. Reason: {}", e),
    };

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("amt {}", amt);
                println!("src: {}", src);
                match str::from_utf8(&buf) {
                    Ok(valid) => {
                        let message = ReqMessage::parse(valid);
                    }
                    Err(error) => {
                        println!("Invalid received bytes: {}", error.to_string());
                    }
                }
            }
            Err(e) => println!("Coult not receive message: {}", e),
        }
    }
}
