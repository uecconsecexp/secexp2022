use crate::matrix;
use crossbeam_channel::{Receiver, Sender};
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::net::TcpStream;

pub trait CommunicatorCore {
    type Sender: Write;
    type Receiver: Read;

    fn get_sender(&mut self) -> &mut Self::Sender;
    fn get_receiver(&mut self) -> &mut BufReader<Self::Receiver>;

    fn write(&mut self, data: &[u8]) -> Result<()> {
        let data = data
            .iter()
            .map(|&b| match b {
                b'\r' => vec![b'\\', b'r'],
                b'\n' => vec![b'\\', b'n'],
                b => vec![b],
            })
            .flatten()
            .collect::<Vec<_>>();

        let sender = self.get_sender();
        sender.write_all(&data)?;
        sender.flush()?;

        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let len = self.get_receiver().read_until(b'\n', &mut buf)?;
        buf = buf[..len].to_vec();

        let mut data = Vec::new();

        buf.reverse();
        while let Some(b) = buf.pop() {
            match b {
                b'\\' => match buf.pop() {
                    Some(b'r') => data.push(b'\r'),
                    Some(b'n') => data.push(b'\n'),
                    Some(b) => {
                        data.push(b'\\');
                        data.push(b);
                    }
                    None => {
                        data.push(b'\\');
                        break;
                    }
                },
                b'\n' => (),
                b => data.push(b),
            }
        }

        Ok(data)
    }
}

pub trait Communicator {
    fn send(&mut self, data: &[u8]) -> Result<()>;

    // channelでrecvを挟むために用意した
    fn receive(&mut self) -> Result<Vec<u8>>;

    fn send_table(&mut self, table: Vec<Vec<f64>>) -> anyhow::Result<()> {
        matrix::send_table(self, table)
    }

    fn receive_table(&mut self) -> anyhow::Result<Vec<Vec<f64>>> {
        matrix::receive_table(self)
    }
}

pub struct TcpCommunicator {
    pub sender: TcpStream,
    pub receiver: BufReader<TcpStream>,
}

impl CommunicatorCore for TcpCommunicator {
    type Sender = TcpStream;
    type Receiver = TcpStream;

    fn get_sender(&mut self) -> &mut Self::Sender {
        &mut self.sender
    }

    fn get_receiver(&mut self) -> &mut BufReader<Self::Receiver> {
        &mut self.receiver
    }
}

impl Communicator for TcpCommunicator {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.write(data)?;
        let sender = self.get_sender();
        sender.write_all(b"\n")?;
        sender.flush()?;

        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>> {
        self.read()
    }
}

pub struct ChannelReceiver {
    pub rx: Receiver<Vec<u8>>,
    received_buf: Vec<u8>,
}

impl ChannelReceiver {
    pub fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            received_buf: Vec::new(),
        }
    }
}

impl Read for ChannelReceiver {
    // &mut [u8]がリサイズできれば簡単に書けるのだけどリサイズ不可能のはずなので...
    // TcpStreamの実装を見てみたところ、そっちはunsafeでうまいことやっているみたいだった

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // let data = self.rx.recv().map_err(|_| std::io::ErrorKind::BrokenPipe)?;
        let (write_vec, len): (Vec<u8>, usize) = if self.received_buf.len() > buf.len() {
            let v = self.received_buf.drain(..buf.len());
            (v.collect(), buf.len())
        } else {
            let len = self.received_buf.len();
            self.received_buf.resize_with(buf.len(), Default::default);
            (self.received_buf.drain(..).collect(), len)
        };
        buf.copy_from_slice(&write_vec);
        Ok(len)
    }
}

pub struct ChannelSender {
    pub tx: Sender<Vec<u8>>,
    send_buf: Vec<u8>,
}

impl ChannelSender {
    pub fn new(tx: Sender<Vec<u8>>) -> Self {
        Self {
            tx,
            send_buf: Vec::new(),
        }
    }
}

impl Write for ChannelSender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.send_buf.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct ChannelCommunicator {
    sender: ChannelSender,
    receiver: BufReader<ChannelReceiver>,
}

impl ChannelCommunicator {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> Self {
        Self {
            sender: ChannelSender::new(tx),
            receiver: BufReader::new(ChannelReceiver::new(rx)),
        }
    }

    fn mut_cr_inner(&mut self) -> &mut ChannelReceiver {
        self.receiver.get_mut()
    }
}

impl CommunicatorCore for ChannelCommunicator {
    type Sender = ChannelSender;
    type Receiver = ChannelReceiver;

    fn get_sender(&mut self) -> &mut Self::Sender {
        &mut self.sender
    }

    fn get_receiver(&mut self) -> &mut BufReader<Self::Receiver> {
        &mut self.receiver
    }
}

impl Communicator for ChannelCommunicator {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.write(data)?;

        let sender = &mut self.sender;
        let mut data: Vec<u8> = sender.send_buf.drain(..).collect();
        data.extend(b"\n");

        sender
            .tx
            .send(data)
            .map_err(|_| std::io::ErrorKind::BrokenPipe)?;

        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>> {
        let cr = self.mut_cr_inner();
        let data = cr.rx.recv().map_err(|_| std::io::ErrorKind::BrokenPipe)?;
        cr.received_buf.extend(data);

        self.read()
    }
}
