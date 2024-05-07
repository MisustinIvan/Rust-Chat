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
    stream: Arc<Mutex<TcpStream>>,
}

impl Client {
    pub fn new(addr: &str, port: &str, username: &str) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", addr, port);
        let stream = TcpStream::connect(addr.clone());

        println!("listening on {addr} as {username}");

        match stream {
            Ok(mut stream) => {
                match stream.set_nonblocking(true) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                match stream.write_all(format!("{}\n", username).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                Ok(Client {
                    stream: Arc::new(Mutex::new(stream)),
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn run(&mut self) {
        let stdin_channel = self.spawn_stdin_reader();
        let server_reader = self.spawn_server_reader();

        loop {
            match stdin_channel.try_recv() {
                Ok(msg) => match self.stream.lock().unwrap().write_all(msg.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {}", e),
                },
                Err(_) => {}
            }

            match server_reader.try_recv() {
                Ok(msg) => println!("{msg}"),
                Err(_) => {}
            }
        }
    }

    fn spawn_stdin_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            };
            if !buffer.trim().is_empty() {
                match sender.send(buffer) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                };
            }
        });
        return receiver;
    }

    fn spawn_server_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let connection = self.stream.clone();

        thread::spawn(move || loop {
            let mut buffer = String::new();
            let stream = connection.lock().unwrap();
            let mut reader = BufReader::new(&*stream);
            match reader.read_line(&mut buffer) {
                Ok(_) => {
                    if !buffer.trim().is_empty() {
                        match sender.send(buffer.trim().to_string()) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        };
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
    let args = std::env::args().collect::<Vec<String>>();

    let username = match args.get(1) {
        Some(username) => username,
        None => "default",
    };

    match Client::new("localhost", "6969", username) {
        Ok(mut client) => {
            client.run();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    };
}
