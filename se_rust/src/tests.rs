use crate::client::{ChannelClient, TcpClient};
use crate::comm::Communicator;
use crate::server::{ChannelServer, TcpServer};
use crossbeam_channel::unbounded;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::{thread, time};

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

    assert_eq!(
        String::from_utf8(rx.recv().unwrap()),
        String::from_utf8(message.to_vec()),
    );
}

fn ping_multiple_raw<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>, messages: Vec<Vec<u8>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    let ms = messages.clone();
    thread::spawn(move || {
        let mut c1 = c1.lock().unwrap();
        for m in ms {
            c1.send(&m).unwrap();
        }
    });

    let ms = messages.clone();
    let t = thread::spawn(move || {
        let mut c2 = c2.lock().unwrap();
        for m in ms {
            let data = c2.receive().unwrap();
            // println!("{:?}", String::from_utf8(data.clone()));
            assert_eq!(String::from_utf8(data), String::from_utf8(m));
        }
    });

    t.join().unwrap();
}

fn ping<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_raw(c1, c2, b"ping");
}

fn ping_multiple<C1, C2>(times: usize, c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_multiple_raw(
        c1,
        c2,
        (0..times)
            .map(|i| format!("ping {}", i).as_bytes().to_vec())
            .collect(),
    );
}

fn ping_with_new_line<C1, C2>(c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_raw(
        c1,
        c2,
        b"Hello,
ping!",
    );
}

fn ping_with_new_line_multiple<C1, C2>(times: usize, c1: Arc<Mutex<C1>>, c2: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    ping_multiple_raw(
        c1,
        c2,
        (0..times)
            .map(|i| {
                format!(
                    "ping
times: {}",
                    i
                )
                .as_bytes()
                .to_vec()
            })
            .collect(),
    );
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

fn ping_matrix_multiple<C1, C2>(
    times: usize,
    c1: Arc<Mutex<C1>>,
    c2: Arc<Mutex<C2>>,
    matrix: Vec<Vec<f64>>,
) where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    let m = matrix.clone();
    thread::spawn(move || {
        let mut c1 = c1.lock().unwrap();
        for _ in 0..times {
            c1.send_table(m.clone()).unwrap();
        }
    });

    let m = matrix.clone();
    let t = thread::spawn(move || {
        let mut c2 = c2.lock().unwrap();
        for _ in 0..times {
            let data = c2.receive_table().unwrap();
            assert_eq!(data, m);
        }
    });

    t.join().unwrap();
}

fn tests_base<C1, C2>(server: Arc<Mutex<C1>>, client: Arc<Mutex<C2>>)
where
    C1: Communicator + Sync + Send + 'static,
    C2: Communicator + Sync + Send + 'static,
{
    println!("ping single");

    ping(Arc::clone(&server), Arc::clone(&client));
    ping(Arc::clone(&client), Arc::clone(&server));

    println!("ping double");

    ping_multiple(2, Arc::clone(&server), Arc::clone(&client));
    ping_multiple(2, Arc::clone(&client), Arc::clone(&server));

    println!("ping new line single");

    ping_with_new_line(Arc::clone(&server), Arc::clone(&client));
    ping_with_new_line(Arc::clone(&client), Arc::clone(&server));

    println!("ping new line double");

    ping_with_new_line_multiple(2, Arc::clone(&server), Arc::clone(&client));
    ping_with_new_line_multiple(2, Arc::clone(&client), Arc::clone(&server));

    let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];

    println!("ping matrix single 1");

    ping_matrix(Arc::clone(&server), Arc::clone(&client), matrix.clone());
    ping_matrix(Arc::clone(&client), Arc::clone(&server), matrix.clone());

    println!("ping matrix double 1");

    ping_matrix_multiple(2, Arc::clone(&server), Arc::clone(&client), matrix.clone());
    ping_matrix_multiple(2, Arc::clone(&client), Arc::clone(&server), matrix.clone());

    let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];

    println!("ping matrix single 2");

    ping_matrix(Arc::clone(&server), Arc::clone(&client), matrix.clone());
    ping_matrix(Arc::clone(&client), Arc::clone(&server), matrix.clone());

    println!("ping matrix double 2");

    ping_matrix_multiple(2, Arc::clone(&server), Arc::clone(&client), matrix.clone());
    ping_matrix_multiple(2, Arc::clone(&client), Arc::clone(&server), matrix.clone());
}

#[test]
fn channel_tests() {
    let (s_tx, s_rx) = unbounded();
    let (c_tx, c_rx) = unbounded();

    let (channel_server, channel_client) = (
        ChannelServer::new(s_rx, c_tx),
        ChannelClient::new(c_rx, s_tx),
    );
    let channel_server = Arc::new(Mutex::new(channel_server));
    let channel_client = Arc::new(Mutex::new(channel_client));

    tests_base(Arc::clone(&channel_server), Arc::clone(&channel_client));
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

    tests_base(Arc::clone(&tcp_server), Arc::clone(&tcp_client));
}
