use crate::comm::{Communicator, ChannelStream};
use anyhow::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use crossbeam_channel::{Sender, Receiver};

pub struct Server<S>
where
    S: Read + Write,
{
    stream: S,
}

impl<S> Communicator for Server<S>
where
    S: Read + Write,
{
    type Stream = S;

    fn get_stream(&mut self) -> &mut S {
        &mut self.stream
    }
}

const ADDRESS: &'static str = "127.0.0.1";
const PORT: &'static str = "10000";

pub type TcpServer = Server<TcpStream>;

pub fn new_tcp_server() -> Result<TcpServer> {
    let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;
    let (stream, addr) = listener.accept()?;
    println!("Connection from {:?}", addr);

    Ok(Server { stream })
}

pub type ChannelServer = Server<ChannelStream>;

pub fn new_channel_server(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelServer {
    Server {
        stream: ChannelStream::new(rx, tx),
    }
}