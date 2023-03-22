use std::str;
use std::{env, net};

enum ReqType {
    Register,
    Invite,
    Ack,
    Bye,
    Cancel,
    Update,
    Refer,
    Prack,
    Subscribe,
    Notify,
    Publish,
    Message,
    Info,
    Options,
}

enum ResType {
    Trying,
    Ringing,
    SessionProgress,
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    RequestTimeout,
    TemporarilyUnavailable,
    BusyHere,
    RequestTerminated,
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
}

fn main() {
    let host_addr = env::args().nth(1).expect("Invalid host IP address");
    let host_port = env::args().nth(2).expect("Invalid host port");
    let host = format!("{}:{}", host_addr, host_port);

    let mut buf = vec![0; 2048];

    println!("Bindind socket on {}...", host);
    let socket = match net::UdpSocket::bind(&host) {
        Ok(s) => s,
        Err(e) => panic!("Could not bind to socket. Reason: {}", e),
    };

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("amt {}", amt);
                println!("src: {}", src);
                match str::from_utf8(&buf) {
                    Ok(valid) => {
                        println!("{}", valid);
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
