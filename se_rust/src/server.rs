use crate::comm::{
    new_channel_sr, ChannelInfo, ChannelReceiver, ChannelSender, CommunicatorCore,
    TCPCommunicator, ChannelCommunicator
};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct Server<S, R>
where
    S: Write,
    R: Read,
{
    sender: S,
    receiver: BufReader<R>,
}

impl<S, R> CommunicatorCore<S, R> for Server<S, R>
where
    S: Write,
    R: Read,
{
    fn get_sender(&mut self) -> &mut S {
        &mut self.sender
    }

    fn get_receiver(&mut self) -> &mut BufReader<R> {
        &mut self.receiver
    }
}

const ADDRESS: &'static str = "127.0.0.1";
const PORT: &'static str = "10000";

pub type TcpServer = Server<TcpStream, TcpStream>;

impl TcpServer {
    pub fn new() -> Result<TcpServer> {
        let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;
        let (stream, addr) = listener.accept()?;
        println!("Connection from {:?}", addr);
        let receiver = BufReader::new(stream.try_clone()?);

        Ok(Server {
            sender: stream,
            receiver,
        })
    }
}

impl TCPCommunicator for TcpServer {}

pub type ChannelServer = Server<ChannelSender, ChannelReceiver>;

impl ChannelServer {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelServer {
        let (sender, receiver) = new_channel_sr(rx, tx);
        let receiver = BufReader::new(receiver);
        Server { sender, receiver }
    }
}

impl ChannelInfo for ChannelServer {
    fn mut_tr(&mut self) -> &mut ChannelSender{
        &mut self.sender
    }
    fn mut_cr(&mut self) -> &mut ChannelReceiver {
        self.receiver.get_mut()
    }
}

impl ChannelCommunicator for ChannelServer {}
