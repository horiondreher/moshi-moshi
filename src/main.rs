#![allow(dead_code)]

use moshi_moshi::Calls;
use std::{
    env,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    str, thread, sync::mpsc,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

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

    let (sender, receiver) = mpsc::channel();

    receiver_thread(receiver);

    loop {
        if let Ok((_, src)) = socket.recv_from(&mut buf) {
            // sender.as_ref().send(Box::new(|| {
            //     handle_connection(&socket, &mut calls, &mut buf, src);
            // }));
            handle_connection(&socket, &mut calls, &mut buf, src);
        } 
    }
}

fn handle_connection(socket: &UdpSocket, calls: &mut Calls, buf: &mut Vec<u8>, src: SocketAddr) {
    let message = str::from_utf8(&buf).unwrap_or_default();

    let responses = calls.handle_sip_message(&src, &message);

    if let Ok(x) = responses {
        for response in x.iter() {
            socket.send_to(response.as_bytes(), src).unwrap();
        }
    }
}

fn receiver_thread(receiver: mpsc::Receiver<Job>) {
    thread::spawn(move || loop {
        let job = receiver.recv().unwrap();

        job();
    });
   
}
