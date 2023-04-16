#![allow(dead_code)]

// TODO: resolve this in lib and main
mod parser;

use parser::SipRequest;
use std::{collections::HashMap, net::SocketAddr};

enum CallState {
    Trying,
    InProgress,
    Ending,
}

struct SipCall {
    state: CallState,
    call_id: String,
    source: SocketAddr,
}

impl SipCall {}

struct Calls {
    sip_calls: HashMap<String, SipCall>,
}

impl Calls {
    fn push(message: &SipRequest) {
        todo!()
    }

    fn find(message: &SipRequest) {
        todo!()
    }

    fn handle_invite(message: &SipRequest) {
        todo!()
    }
}
