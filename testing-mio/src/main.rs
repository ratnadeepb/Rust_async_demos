use mio::{net::TcpStream, Events, Interest, Poll, Token};
use std::net::{self, SocketAddr};

fn main() {
    // server socket
    let addr: SocketAddr = "127.0.0.1:0".parse().expect("Failed to parse socket addr");
    let server = net::TcpListener::bind(addr).expect("Failed to create listener");

    // new poll handle and events
    let mut poll = Poll::new().expect("Failed to create a poll handle");
    let mut events = Events::with_capacity(1024);

    // connect the stream
    let mut stream =
        TcpStream::connect(server.local_addr().unwrap()).expect("Failed to create stream");
    // register the stream with poll
    poll.registry()
        .register(
            &mut stream,
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )
        .expect("Failed to register interest");

    // wait for the socket to become ready. this happens in a loop to handle spurious wakeups
    loop {
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            if event.token() == Token(0) && event.is_writable() {
                // the socket connected (but could be a spurious wakeup too)
                println!("Something connected");
            }
        }
    }
}
