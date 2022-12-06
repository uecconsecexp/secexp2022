use crate::comm::{new_channel_sr, ChannelInfo, ChannelReceiver, ChannelSender, CommunicatorCore, ChannelCommunicator, TCPCommunicator};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

pub struct Client<S, R>
where
    S: Write,
    R: Read,
{
    sender: S,
    receiver: BufReader<R>,
}

impl<S, R> CommunicatorCore<S, R> for Client<S, R>
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

const PORT: &'static str = "10000";

pub type TcpClient = Client<TcpStream, TcpStream>;

impl TcpClient {
    pub fn new(server_address: &str) -> Result<TcpClient> {
        let stream = TcpStream::connect(format!("{}:{}", server_address, PORT))?;
        println!("Connection to {:?}", server_address);
        let receiver = BufReader::new(stream.try_clone()?);

        Ok(Client {
            sender: stream,
            receiver,
        })
    }
}

impl TCPCommunicator for TcpClient {}

pub type ChannelClient = Client<ChannelSender, ChannelReceiver>;

impl ChannelClient {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelClient {
        let (sender, receiver) = new_channel_sr(rx, tx);
        let receiver = BufReader::new(receiver);
        Client { sender, receiver }
    }
}

impl ChannelInfo for ChannelClient {
    fn mut_tr(&mut self) -> &mut ChannelSender{
        &mut self.sender
    }
    fn mut_cr(&mut self) -> &mut ChannelReceiver {
        self.receiver.get_mut()
    }
}

impl ChannelCommunicator for ChannelClient {}
