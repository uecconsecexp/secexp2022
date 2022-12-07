use crate::comm::{ChannelCommunicator, Communicator, TcpCommunicator};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::io::BufReader;
use std::net::TcpListener;

pub struct Server<C: Communicator>(C);

impl<C> Communicator for Server<C>
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

const ADDRESS: &'static str = "127.0.0.1";
const PORT: &'static str = "10000";

pub type TcpServer = Server<TcpCommunicator>;

impl TcpServer {
    pub fn new() -> Result<TcpServer> {
        let listener = TcpListener::bind(format!("{}:{}", ADDRESS, PORT))?;
        let (stream, addr) = listener.accept()?;
        println!("Connection from {:?}", addr);
        let receiver = BufReader::new(stream.try_clone()?);

        Ok(Self(TcpCommunicator {
            sender: stream,
            receiver,
        }))
    }
}

pub type ChannelServer = Server<ChannelCommunicator>;

impl ChannelServer {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> ChannelServer {
        let cc = ChannelCommunicator::new(rx, tx);

        Self(cc)
    }
}
