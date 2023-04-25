#![allow(dead_code)]

// TODO: resolve this in lib and main
mod parser;

use parser::ReqMessage;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::hash::Hash;
use std::net::SocketAddr;

#[derive(Debug)]
pub enum SipError {
    ResponseMessage,
}

impl fmt::Display for SipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SipError::ResponseMessage => {
                write!(f, "Could not create response for provided request")
            }
        }
    }
}

enum CallState {
    Initializing,
    Trying,
    InProgress,
    Ending,
}

pub struct Call {
    state: CallState,
    call_id: String,
    source: SocketAddr,
}

impl Call {
    pub fn create_response(&self, response_type: &str) -> String {
        let mut response = response_type.to_string();

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
            "To: service <sip:service@{}:{}>;tag=10273SIPpTag011\r\n",
            self.source.ip(),
            self.source.port()
        )
        .unwrap();
        write!(response, "Call-ID: {}\r\n", self.call_id).unwrap();
        write!(response, "CSeq: 1 INVITE\r\n").unwrap();
        write!(
            response,
            "Contact: <sip:{}:{};transport=UDP>\r\n",
            self.source.ip(),
            self.source.port()
        )
        .unwrap();
        write!(response, "Content-Length: 0\r\n\r\n").unwrap();

        return response;
    }
}

pub struct Calls {
    sip_calls: HashMap<String, Call>,
}

impl<'a> Calls {
    pub fn new() -> Self {
        Calls {
            sip_calls: HashMap::new(),
        }
    }

    pub fn handle_sip_message(
        &mut self,
        src_socket: &SocketAddr,
        message: &str,
    ) -> Result<Vec<String>, SipError> {
        let message = match ReqMessage::parse(message) {
            Ok(x) => x,
            Err(_e) => return Err(SipError::ResponseMessage),
        };

        let response = self.handle_request(&src_socket, &message);

        Ok(response)
    }

    pub fn handle_request(&mut self, src: &SocketAddr, message: &'a ReqMessage) -> Vec<String> {
        //TODO: create classes for registers and options
        let call_opt = self.find(message);

        // TODO: there is gotta be a way to improve this
        let call: &mut Call = match call_opt {
            Some(x) => x,
            None => {
                let call_id = message.get_single_header("Call-ID").unwrap().value;
                self.sip_calls.insert(
                    call_id.to_owned(),
                    Call {
                        state: CallState::Initializing,
                        call_id: call_id.to_owned(),
                        source: src.clone(),
                    },
                );
                self.sip_calls.get_mut(call_id).unwrap()
            }
        };
        // let call = self.find(message).unwrap_or(self.create_call(message));

        // TODO: create states
        let mut responses: Vec<String> = Vec::new();
        match call.state {
            CallState::Initializing => {
                responses.push(call.create_response("SIP/2.0 180 Ringing\r\n"));
                responses.push(call.create_response("SIP/2.0 200 OK\r\n"));
                call.state = CallState::InProgress;
            }
            CallState::Trying => {
                responses.push(call.create_response("SIP/2.0 180 Ringing\r\n"));
            }
            CallState::InProgress => {
                responses.push(call.create_response("SIP/2.0 200 OK\r\n"));
            }
            CallState::Ending => {
                responses.push(call.create_response("SIP/2.0 180 Ringing\r\n"));
            }
        };

        responses
    }

    fn create_call(&mut self, _message: &ReqMessage) -> &mut Call {
        todo!()
    }

    fn push(_message: &ReqMessage) {
        todo!()
    }

    fn find(&mut self, message: &ReqMessage) -> Option<&mut Call> {
        let call_id = message.get_single_header("Call-ID");

        if let Some(x) = call_id {
            return self.sip_calls.get_mut(x.value);
        }

        return None;
    }
}
