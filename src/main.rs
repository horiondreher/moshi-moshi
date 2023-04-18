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

    loop {
        match socket.recv_from(&mut buf) {
            Ok((_amt, src)) => match str::from_utf8(&buf) {
                Ok(valid) => {
                    let response = Calls::handle_sip_message(&src, &valid);

                    match response {
                        Some(x) => socket.send_to(x.as_bytes(), src).unwrap(),
                        None => continue,
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
