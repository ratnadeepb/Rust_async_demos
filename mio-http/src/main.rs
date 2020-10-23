// Testing: wrk -d 60s -t 8 -c 128 --rate 150k http://127.0.0.1:8080/

use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

// predefined HTTP response
static RESPONSE: &str = "HTTP/1.1 200 OK
Content-Type: text/html
Connection: keep-alive
Content-Length: 6

hello
";

fn is_double_crnl(window: &[u8]) -> bool {
    window.len() >= 4
        && (window[0] == '\r' as u8)
        && (window[1] == '\n' as u8)
        && (window[2] == '\r' as u8)
        && (window[3] == '\n' as u8)
}

fn main() {
    let addr = "0.0.0.0:8080";
    let mut listener = TcpListener::bind(addr.parse().unwrap()).unwrap();

    let mut counter: usize = 0;
    let mut sockets: HashMap<Token, TcpStream> = HashMap::new();

    // let mut response: HashMap<Token, usize> = HashMap::new();

    // Fixed size buffer for reading/writing to/from sockets
    let mut buffer = [0 as u8; 1024];

    // Mocking HTTP
    let mut requests: HashMap<Token, Vec<u8>> = HashMap::new();

    // Then create Poll object and register listener at Token(0) for readable events, activated by edge
    let mut poll = Poll::new().unwrap();
    poll.registry()
        .register(&mut listener, Token(0), Interest::READABLE)
        .unwrap();

    // create events object of a given capacity
    // and a main loop
    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            // accepting connections and dropping them
            // readable events on the listener means incoming connections are waiting to be accepted
            // event on the connected socket:
            // 1. readable - the socket has data available for reading
            // 2. writable - the socket is ready for writing some data into it
            // the listener vs socket event can be distinguished by token, where the listener token is always zero, as it was registered in Poll
            // simple event handling -  accept all connections, drop all packets

            match event.token() {
                Token(0) => {
                    loop {
                        match listener.accept() {
                            Ok((mut socket, _address)) => {
                                // println!("Got connection from {:?}", address);
                                // register sockets to poll
                                counter += 1;
                                let token = Token(counter);

                                // register readable events
                                poll.registry()
                                    .register(
                                        &mut socket,
                                        token,
                                        // marking sockets as both read and write at the same time is problematic
                                        // the "mocking HTTP" protocol will allow us to decide when a socket should be marked as write
                                        // Interest::READABLE | Interest::WRITABLE,
                                        Interest::READABLE,
                                    )
                                    .unwrap();

                                sockets.insert(token, socket);
                                requests.insert(token, Vec::with_capacity(192));
                            } // connection dropped
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // no more connections (the error connection says it's about to block)
                                break;
                            }
                            Err(e) => {
                                panic!("unexpected error: {}", e);
                            }
                        }
                    }
                }
                token if event.is_readable() => {
                    // let mut bytes_read: usize = 0;
                    // Socket associated with token is ready for reading data from it
                    loop {
                        let read = sockets.get_mut(&token).unwrap().read(&mut buffer);
                        match read {
                            Ok(0) => {
                                // successful read of zero bytes means connected is closed
                                sockets.remove(&token);
                                break;
                            }
                            Ok(n) => {
                                // println!("Read {} bytes for token {}", n, token.0);
                                let req = requests.get_mut(&token).unwrap();
                                for b in &buffer[0..n] {
                                    req.push(*b);
                                }
                                // bytes_read += n;
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(e) => panic!("Unexpected error: {}", e),
                        }
                    }
                    // response.insert(token, bytes_read);

                    // once reading is done, mark socket for writing
                    let ready = requests
                        .get(&token)
                        .unwrap()
                        .windows(4)
                        .find(|window| is_double_crnl(*window))
                        .is_some();

                    if ready {
                        let socket = sockets.get_mut(&token).unwrap();
                        poll.registry()
                            .reregister(socket, token, Interest::WRITABLE)
                            .unwrap();
                    }
                }
                token if event.is_writable() => {
                    requests.get_mut(&token).unwrap().clear();
                    sockets
                        .get_mut(&token)
                        .unwrap()
                        .write_all(RESPONSE.as_bytes())
                        .unwrap();
                    // let n_bytes = response[&token];
                    // let message = format!("Received {} bytes!", n_bytes);
                    // sockets.get_mut(&token).unwrap().write_all(message.as_bytes()).unwrap();
                    // response.remove(&token);
                    // sockets.remove(&token);

                    // Re-use existing connection ("keep-alive") - switch back to reading
                    poll.registry()
                        .reregister(sockets.get_mut(&token).unwrap(), token, Interest::READABLE)
                        .unwrap();
                }
                _ => {} // ignore everything else
            }
        }
    }
}
