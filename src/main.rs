#![allow(dead_code)]

use moshi_moshi::Calls;
use std::{
    env,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    str,
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<UdpBuffer>;

struct UdpBuffer {
    socket: Arc<UdpSocket>,
    message: Vec<u8>,
    src: SocketAddr,
}

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
    let socket = match UdpSocket::bind(addr) {
        Ok(s) => s,

        Err(e) => panic!("Could not bind to socket. Reason: {}", e),
    };

    let socket = Arc::new(socket);
    let calls = Arc::new(Mutex::new(Calls::new()));
    let (sender, receiver) = mpsc::channel();

    receiver_thread(receiver, calls);

    loop {
        if let Ok((_, src)) = socket.recv_from(&mut buf) {
            let message = buf.clone();

            sender
                .send(Box::new(UdpBuffer {
                    socket: Arc::clone(&socket),
                    message,
                    src,
                }))
                .unwrap();
        }
    }
}

fn receiver_thread(receiver: mpsc::Receiver<Job>, calls: Arc<Mutex<Calls>>) {
    thread::spawn(move || loop {
        let udp_message = receiver.recv().unwrap();

        let message = str::from_utf8(&udp_message.message).unwrap_or_default();

        let responses = calls
            .lock()
            .unwrap()
            .handle_sip_message(&udp_message.src, message);

        if let Ok(x) = responses {
            for response in x.iter() {
                udp_message
                    .socket
                    .send_to(response.as_bytes(), udp_message.src)
                    .unwrap();
            }
        }
    });
}
