use crate::matrix;
use std::io::{BufRead, BufReader, Result, Write, Read};
use crossbeam_channel::{Sender, Receiver};

pub trait Communicator {
    type Stream: Write + Read;

    fn get_stream(&mut self) -> &mut Self::Stream;

    fn send(&mut self, data: &[u8]) -> Result<()> {
        let data = data
            .iter()
            .map(|&b| match b {
                b'\r' => vec![b'\\', b'r'],
                b'\n' => vec![b'\\', b'n'],
                b => vec![b],
            })
            .flatten()
            .collect::<Vec<_>>();

        let stream = self.get_stream();
        stream.write_all(&data)?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        Ok(())
    }

    fn send_table(&mut self, table: Vec<Vec<f64>>) -> anyhow::Result<()> {
        matrix::send_table(self, table)
    }

    fn receive(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut reader = BufReader::new(self.get_stream());
        let len = reader.read_until(b'\n', &mut buf)?;
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

    fn receive_table(&mut self) -> anyhow::Result<Vec<Vec<f64>>> {
        matrix::receive_table(self)
    }
}

pub struct ChannelStream {
    pub rx: Receiver<Vec<u8>>,
    received_buf: Vec<u8>,
    pub tx: Sender<Vec<u8>>,
}

impl ChannelStream {
    pub fn new(rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) -> Self {
        Self {
            rx,
            received_buf: Vec::new(),
            tx
        }
    }
}

impl Read for ChannelStream {

    // &mut [u8]がリサイズできれば簡単に書けるのだけどリサイズ不可能のはずなので...
    // TcpStreamの実装を見てみたところ、そっちはunsafeでうまいことやっているみたいだった

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while let Ok(data) = self.rx.try_recv() {
            self.received_buf.extend(data);
        }
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

impl Write for ChannelStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx
            .send(buf.to_vec())
            .map_err(|_| std::io::ErrorKind::BrokenPipe)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}