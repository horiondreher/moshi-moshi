#![allow(dead_code)]

// TODO: resolve this in lib and main
mod parser;

use parser::{ReqMessage, ReqMethod, ResType};
use std::collections::HashMap;
use std::fmt::{self, Write};
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

impl From<std::fmt::Error> for SipError {
    fn from(_error: std::fmt::Error) -> Self {
        SipError::ResponseMessage
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
    cseq: u32,
    call_id: String,
    source: SocketAddr,
}

impl Call {
    pub fn create_response(&self, response_type: ResType, request: &str) -> Result<String, SipError> {
        let mut response = String::from("SIP/2.0 ");

        write!(response, "{}\r\n", response_type)?;
        write!(
            response,
            "Via: SIP/2.0/UDP 192.168.1.146:5070;branch=z9hG4bK-10274-1-0\r\n"
        )?;

        write!(
            response,
            "From: sipp <sip:sipp@192.168.1.146:5070>;tag=10274SIPpTag001\r\n"
        )?;

        write!(
            response,
            "To: service <sip:service@{}:{}>;tag=10273SIPpTag011\r\n",
            self.source.ip(),
            self.source.port()
        )?;

        write!(response, "Call-ID: {}\r\n", self.call_id)?;
        write!(response, "CSeq: {} {}\r\n", self.cseq, request)?;
        write!(
            response,
            "Contact: <sip:{}:{};transport=UDP>\r\n",
            self.source.ip(),
            self.source.port()
        )?;
        write!(response, "Content-Length: 0\r\n\r\n")?;

        return Ok(response);
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
                        cseq: 0,
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

        call.cseq = message.get_cseq_number().unwrap();

        // TODO: create fmt::Display for Requests
        match message.method {
            ReqMethod::Invite => {
                if let Ok(session_progress) = call.create_response(ResType::SessionProgress, "INVITE") {
                    responses.push(session_progress);
                };
                if let Ok(ok_res) = call.create_response(ResType::Ok, "INVITE") {
                    responses.push(ok_res);
                };
                call.state = CallState::InProgress;
            }
            ReqMethod::Bye => {
                if let Ok(ok_res) = call.create_response(ResType::Ok, "BYE") {
                    responses.push(ok_res);
                };
                call.state = CallState::Ending;
            }
            _ => (),
        }
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
