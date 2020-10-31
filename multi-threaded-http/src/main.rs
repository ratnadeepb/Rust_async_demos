mod pool;

use log::debug;
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use parser_combinators::{
    http::{as_string, parse_http_request, Header, Request, Response},
    stream::ByteStream,
};
use pool::ThreadPool;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

fn blocks(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::WouldBlock
}

fn get_header<'a>(headers: &'a Vec<Header>, name: &String) -> Option<&'a String> {
    headers.iter().find(|h| &h.name == name).map(|h| &h.value)
}

fn res_sec_websocket_accept(req_sec_websocket_key: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(req_sec_websocket_key.to_owned() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::encode(hasher.finalize())
}

fn handler(req: Request) -> Response {
    let connection = get_header(&req.headers, &"Connection".to_string())
        .map(|h| h.contains("Upgrade"))
        .unwrap_or_default();

    let upgrade = get_header(&req.headers, &"Upgrade".to_string())
        .map(|h| h.contains("websocket"))
        .unwrap_or_default();

    
}

struct Handler {
    token: Token,
    socket: TcpStream,
    is_open: bool,
    recv_stream: ByteStream,
    send_stream: ByteStream,
}

fn main() {
    println!("Hello, world!");
}
