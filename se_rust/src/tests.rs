use std::{time, thread};
use crate::server::{TcpServer, ChannelServer};
use crate::client::{TcpClient, ChannelClient};
use crate::comm::Communicator;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use crossbeam_channel::bounded;

fn ping_raw<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>, message: &[u8])
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    let m = message.to_vec();
    thread::spawn(move || {
        let mut c1 = c1.lock().unwrap();
        c1.send(&m).unwrap();
    });
    
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut c2 = c2.lock().unwrap();
        let data = c2.receive().unwrap();
        tx.send(data).unwrap();
    });

    assert_eq!(rx.recv().unwrap(), message);
}

fn ping<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_raw(c1, c2, b"ping");
}

fn ping_with_new_line<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_raw(c1, c2, b"Hello,
ping!");
}

fn ping_matrix<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>, matrix: Vec<Vec<f64>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    let m = matrix.clone();
    thread::spawn(move || {
        let mut c1 = c1.lock().unwrap();
        c1.send_table(m).unwrap();
    });
    
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut c2 = c2.lock().unwrap();
        let data = c2.receive_table().unwrap();
        tx.send(data).unwrap();
    });

    assert_eq!(rx.recv().unwrap(), matrix);
}

#[test]
fn channel_tests() {
    let (s_tx, s_rx) = bounded(0);
    let (c_tx, c_rx) = bounded(0);

    let (channel_server, channel_client) = (ChannelServer::new(s_rx, c_tx), ChannelClient::new(c_rx, s_tx));
    let channel_server = Arc::new(Mutex::new(channel_server));
    let channel_client = Arc::new(Mutex::new(channel_client));

    ping(Arc::clone(&channel_server), Arc::clone(&channel_client));
    ping(Arc::clone(&channel_client), Arc::clone(&channel_server));
    ping_with_new_line(Arc::clone(&channel_server), Arc::clone(&channel_client));
    ping_with_new_line(Arc::clone(&channel_client), Arc::clone(&channel_server));
    let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    ping_matrix(Arc::clone(&channel_server), Arc::clone(&channel_client), matrix.clone());
    ping_matrix(Arc::clone(&channel_client), Arc::clone(&channel_server), matrix);
    let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    ping_matrix(Arc::clone(&channel_server), Arc::clone(&channel_client), matrix.clone());
    ping_matrix(Arc::clone(&channel_client), Arc::clone(&channel_server), matrix);
}

fn prepare_tcp_members() -> (TcpServer, TcpClient) {
    let (s_tx, s_rx) = channel();

    thread::spawn(move || {
        let tcp_server = TcpServer::new().unwrap();
        s_tx.send(tcp_server).unwrap();
    });

    thread::sleep(time::Duration::from_millis(100));

    let (c_tx, c_rx) = channel();

    thread::spawn(move || {
        let tcp_client = TcpClient::new("0.0.0.0").unwrap();
        c_tx.send(tcp_client).unwrap();
    });

    (s_rx.recv().unwrap(), c_rx.recv().unwrap())
}

#[test]
fn tcp_tests() {
    let (tcp_server, tcp_client) = prepare_tcp_members();
    let tcp_server = Arc::new(Mutex::new(tcp_server));
    let tcp_client = Arc::new(Mutex::new(tcp_client));

    ping(Arc::clone(&tcp_server), Arc::clone(&tcp_client));
    ping(Arc::clone(&tcp_client), Arc::clone(&tcp_server));
    ping_with_new_line(Arc::clone(&tcp_server), Arc::clone(&tcp_client));
    ping_with_new_line(Arc::clone(&tcp_client), Arc::clone(&tcp_server));
    let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    ping_matrix(Arc::clone(&tcp_server), Arc::clone(&tcp_client), matrix.clone());
    ping_matrix(Arc::clone(&tcp_client), Arc::clone(&tcp_server), matrix);
    let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    ping_matrix(Arc::clone(&tcp_server), Arc::clone(&tcp_client), matrix.clone());
    ping_matrix(Arc::clone(&tcp_client), Arc::clone(&tcp_server), matrix);
}