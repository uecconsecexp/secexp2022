use crate::comm::{ChannelCommunicator, Communicator, TcpCommunicator};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::io::BufReader;
use std::net::TcpStream;

pub struct Client<C: Communicator>(C);

impl<C> Communicator for Client<C>
where
    C: Communicator,
{
    fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.0.send(data)
    }

    fn receive(&mut self) -> std::io::Result<Vec<u8>> {
        self.0.receive()
    }
}

const PORT: &'static str = "10000";

pub type TcpClient = Client<TcpCommunicator>;

impl TcpClient {
    pub fn new(server_address: &str) -> Result<TcpClient> {
        let stream = TcpStream::connect(format!("{}:{}", server_address, PORT))?;
        println!("Connection to {:?}", server_address);
        let receiver = BufReader::new(stream.try_clone()?);

        Ok(Client(TcpCommunicator {
            sender: stream,
            receiver,
        }))
    }
}

pub type ChannelClient = Client<ChannelCommunicator>;

impl ChannelClient {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelClient {
        let cc = ChannelCommunicator::new(rx, tx);
        Self(cc)
    }
}
