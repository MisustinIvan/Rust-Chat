use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

#[allow(dead_code)]
struct Client {
    connection: Arc<Mutex<TcpStream>>,
}

impl Client {
    pub fn new(addr: &str, port: &str) -> Self {
        let connection = Arc::new(Mutex::new(
            TcpStream::connect(format!("{}:{}", addr, port)).unwrap(),
        ));
        connection.lock().unwrap().set_nonblocking(true).unwrap();
        println!("Connected to server at {}:{}", addr, port);
        Self { connection }
    }

    pub fn run(&mut self) {
        let stdin_channel = self.spawn_stdin_reader();
        let server_reader = self.spawn_server_reader();

        loop {
            match stdin_channel.try_recv() {
                Ok(msg) => self
                    .connection
                    .lock()
                    .unwrap()
                    .write_all(msg.as_bytes())
                    .unwrap(),
                Err(_) => {}
            }

            match server_reader.try_recv() {
                Ok(msg) => println!("Received message: {}", msg),
                Err(_) => {}
            }
        }
    }

    fn spawn_stdin_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            if !buffer.trim().is_empty() {
                sender.send(buffer).unwrap();
            }
        });
        return receiver;
    }

    fn spawn_server_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let connection = self.connection.clone();

        thread::spawn(move || loop {
            let mut buffer = String::new();
            let stream = connection.lock().unwrap();
            let mut reader = BufReader::new(&*stream);
            match reader.read_line(&mut buffer) {
                Ok(_) => {
                    if !buffer.trim().is_empty() {
                        sender.send(buffer.trim().to_string()).unwrap();
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        continue;
                    } else {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
        });
        return receiver;
    }
}

fn main() {
    let mut client = Client::new("localhost", "6969");
    client.run();
}
