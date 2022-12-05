use crate::comm::{Communicator, ChannelStream};
use anyhow::Result;
use std::net::TcpStream;
use std::io::{Read, Write};
use crossbeam_channel::{Sender, Receiver};

pub struct Client<S>
where
    S: Read + Write,
{
    stream: S,
}

impl<S> Communicator for Client<S>
where
    S: Read + Write,
{
    type Stream = S;

    fn get_stream(&mut self) -> &mut S {
        &mut self.stream
    }
}

const PORT: &'static str = "10000";

pub type TcpClient = Client<TcpStream>;

impl TcpClient {
    pub fn new(server_address: &str) -> Result<TcpClient> {
        let stream = TcpStream::connect(format!("{}:{}", server_address, PORT))?;
        println!("Connection to {:?}", server_address);
    
        Ok(Client { stream })
    }
}

pub type ChannelClient = Client<ChannelStream>;

impl ChannelClient {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelClient {
        Client {
            stream: ChannelStream::new(rx, tx),
        }
    }
}