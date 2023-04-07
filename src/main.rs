#![allow(dead_code)]

mod parser;

use parser::SipRequest;
use std::{env, net, str};

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
            Ok((_amt, _src)) => {
                match str::from_utf8(&buf) {
                    Ok(valid) => {
                        let message = SipRequest::parse(valid.clone());
                        println!(
                            "{}",
                            message.as_ref().unwrap().get_single_header("Via").unwrap().value
                        );
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
