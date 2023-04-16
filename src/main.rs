#![allow(dead_code)]

mod parser;

use parser::SipRequest;
use std::fmt::Write;
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
            Ok((_amt, src)) => match str::from_utf8(&buf) {
                Ok(valid) => {
                    let message = match SipRequest::parse(valid) {
                        Ok(x) => x,
                        Err(_e) => continue,
                    };

                    println!("Source: {:#?}", message);
                    println!("{}", message.get_single_header("Call-ID").unwrap().value);

                    let call_id = message.get_single_header("Call-ID").unwrap().value;

                    let mut response = "SIP/2.0 180 Ringing\r\n".to_string();
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

                    socket.send_to(response.as_bytes(), src).unwrap();
                }
                Err(error) => {
                    println!("Invalid received bytes: {}", error.to_string());
                }
            },
            Err(e) => println!("Coult not receive message: {}", e),
        }
    }
}
