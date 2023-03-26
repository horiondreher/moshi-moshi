use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{char, space0},
    combinator::{map, opt},
    sequence::{delimited, separated_pair},
    IResult,
};

use std::collections::{BTreeMap, VecDeque};
use std::str;
use std::{env, net};

// use nom::bytes::complete::{take_while, take_while1};

#[macro_export]
macro_rules! sip_parse_error {
    // error with message
    ($error_code:expr) => {
        Err(nom::Err::Error(SipLineParseError::new($error_code, None)))
    };

    // error without message
    ($error_code:expr, $message:expr) => {
        Err(nom::Err::Error(SipLineParseError::new(
            $error_code,
            Some($message),
        )))
    };
}

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

struct Uri<'a> {
    raw: &'a str,
    display_name: &'a str,
    protocol: &'a str,
    user: &'a str,
    address: &'a str,
    tag: &'a str,
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
    orig_addr: &'a str,
    uri: Uri<'a>,
    headers: Option<Headers<'a>>,
    body: Option<&'a str>,
}

// pub fn is_field_char(c: char) -> bool {
//     c.is_ascii() && c != ':'
// }

pub fn is_field_char(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii() && c != ':')(input)
}

impl<'a> ReqMessage<'a> {
    fn new(message: &str) -> Self {
        let lines: Vec<&str> = message.split("\r\n").collect();

        for line in lines {
            println!("{}", line);
            if let Ok((_input, header)) = ReqMessage::parse_line(line) {
                println!("Header - {}", header.name);
                println!("Value - {}", header.value);
            }
            // let (_input, header) = ReqMessage::parse_line(line).unwrap();
            // println!("Header - {}", header.name);
            // println!("Value - {}", header.value);
        }

        ReqMessage {
            method: ReqMethod::Invite,
            direction: ReqDirection::In,
            orig_addr: "",
            uri: Uri {
                raw: "",
                display_name: "",
                protocol: "",
                user: "",
                address: "",
                tag: "",
            },
            headers: None,
            body: None,
        }
    }

    fn parse_line(line: &str) -> IResult<&str, Header> {
        // let parse_header_value = separated_pair(is_field_char, char(':'), parse_field_char);

        // map(parse_header_value, |(header, value)| Header {
        //     name: header,
        //     value: value,
        //     params: None,
        // })(line)

        let (input, (name, value)) = separated_pair(
            take_until(":"),
            delimited(space0, char(':'), space0),
            alt((
                delimited(space0, take_until("\r\n"), opt(tag("\r\n"))),
                take_until("\r\n"),
            )),
        )(line)?;

        // let header = SipHeader {
        //     name: name.to_string(),
        //     value: value.to_string(),
        // };

        // let (out_line, header_name) = take_while1(is_field_char)(line)?;
        // let (out_line, value) = take_while(is_whitespace)(out_line)?;

        // println!("Value - {}", out_line);
        // println!("Value - {}", value);

        Ok((
            input,
            Header {
                name: name,
                value: value,
                params: None,
            },
        ))
    }
}
fn main() {
    let host_addr = env::args().nth(1).expect("Invalid host IP address");
    let host_port = env::args().nth(2).expect("Invalid host port");
    let host = format!("{}:{}", host_addr, host_port);

    let mut buf = vec![0; 2048];

    println!("Bindind socket on {}...", host);
    let socket = match net::UdpSocket::bind(&host) {
        Ok(s) => s,
        Err(e) => panic!("Could not bind to socket. Reason: {}", e),
    };

    let global_message: ReqMessage;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("amt {}", amt);
                println!("src: {}", src);
                match str::from_utf8(&buf) {
                    Ok(valid) => {
                        // println!("{}", valid);
                        let message = ReqMessage::new(valid);
                        global_message = message;
                        break;
                    }
                    Err(error) => {
                        println!("Invalid received bytes: {}", error.to_string());
                    }
                }
            }
            Err(e) => println!("Coult not receive message: {}", e),
        }
    }

    println!("{:#?}", global_message.method);
}
