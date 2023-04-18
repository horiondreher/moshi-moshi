#![allow(dead_code)]

// TODO: resolve this in lib and main
mod parser;

use parser::{ReqMessage, ReqMethod};
use std::collections::HashMap;
use std::fmt::Write;
use std::net::SocketAddr;

enum CallState {
    Trying,
    InProgress,
    Ending,
}

pub struct SipCall {
    state: CallState,
    call_id: String,
    source: SocketAddr,
}

impl SipCall {}

pub struct Calls {
    sip_calls: HashMap<String, SipCall>,
}

impl<'a> Calls {
    fn push(message: &ReqMessage) {
        todo!()
    }
    fn find(message: &ReqMessage) -> Option<CallState> {
        todo!()
    }

    pub fn handle_sip_message(src_socket: &SocketAddr, message: &str) -> Option<String> {
        let message = match ReqMessage::parse(message) {
            Ok(x) => x,
            Err(_e) => return None,
        };

        let response: String = Self::handle_request(&src_socket, &message);

        Some(response)
    }

    pub fn handle_request(src: &SocketAddr, message: &'a ReqMessage) -> String {
        let state = match message.method {
            ReqMethod::Invite => Self::handle_invite(message),
            _ => Self::find(message),
        };

        let call_id = message.get_single_header("Call-ID").unwrap().value;

        let mut response = "SIP/2.0 180 Ringing\r\n".to_string();
        let host_addr = src.ip();
        let port = src.port();

        write!(
            response,
            "Via: SIP/2.0/UDP 192.168.1.146:5070;branch=z9hG4bK-10274-1-0\r\n"
        )
        .unwrap();
        write!(
            response,
            "From: sipp <sip:sipp@192.168.1.146:5070>;tag=10274SIPpTag001\r\n"
        )
        .unwrap();
        write!(
            response,
            "To: service <sip:service@{host_addr}:{port}>;tag=10273SIPpTag011\r\n"
        )
        .unwrap();
        write!(response, "Call-ID: {call_id}\r\n").unwrap();
        write!(response, "CSeq: 1 INVITE\r\n").unwrap();
        write!(
            response,
            "Contact: <sip:{host_addr}:{port};transport=UDP>\r\n"
        )
        .unwrap();
        write!(response, "Content-Length: 0\r\n\r\n").unwrap();

        response
    }

    fn handle_invite(message: &ReqMessage) -> Option<CallState> {
        todo!()
    }
}
