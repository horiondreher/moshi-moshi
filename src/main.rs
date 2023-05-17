#![allow(dead_code)]

mod parser;

use moshi_moshi::Calls;
use std::{
    env,
    net::{ToSocketAddrs, UdpSocket},
    str,
};

fn main() {
    let host_addr = env::args().nth(1).expect("Invalid host IP address");
    let port = env::args().nth(2).expect("Invalid port");
    let addr = format!("{}:{}", host_addr, port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let mut buf = vec![0; 2048];

    println!("Bindind socket on {}...", addr);
    let socket = match UdpSocket::bind(&addr) {
        Ok(s) => s,
        Err(e) => panic!("Could not bind to socket. Reason: {}", e),
    };

    let mut calls = Calls::new();

    loop {
        match socket.recv_from(&mut buf) {
            Ok((_amt, src)) => match str::from_utf8(&buf) {
                Ok(valid) => {
                    let responses: Result<Vec<String>, moshi_moshi::SipError> =
                        calls.handle_sip_message(&src, &valid);

                    match responses {
                        Ok(x) => {
                            for response in x.iter() {
                                socket.send_to(response.as_bytes(), src).unwrap();
                            }
                        }
                        Err(_e) => continue,
                    };
                }
                Err(error) => {
                    println!("Invalid received bytes: {}", error.to_string());
                }
            },
            Err(e) => println!("Coult not receive message: {}", e),
        }
    }
}
