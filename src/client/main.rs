use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver},
    thread,
};

#[allow(dead_code)]
struct Client {
    stream: TcpStream,
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

                Ok(Client { stream })
            }
            Err(e) => Err(e),
        }
    }

    pub fn run(&mut self) {
        let stdin_channel = self.spawn_stdin_reader();
        let server_reader = self.spawn_server_reader();
        let mut connection = self.stream.try_clone().unwrap();

        loop {
            match stdin_channel.try_recv() {
                Ok(msg) => match msg.trim() {
                    "/exit" => {
                        break;
                    }
                    _ => match connection.write_all(msg.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => eprintln!("[ERROR] -> {}", e),
                    },
                },
                Err(_) => {}
            }

            match server_reader.try_recv() {
                Ok(msg) => println!("{msg}"),
                Err(_) => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        println!("[INFO] -> exitting");
        self.stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    fn spawn_stdin_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let stdin = io::stdin();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            match stdin.read_line(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[ERROR] -> {}", e);
                }
            };
            if !buffer.trim().is_empty() {
                match sender.send(buffer) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("[ERROR] -> {}", e);
                    }
                };
            }
        });
        return receiver;
    }

    fn spawn_server_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let connection = self.stream.try_clone().unwrap();

        thread::spawn(move || loop {
            let mut buffer = String::new();
            let mut reader = BufReader::new(&connection);
            match reader.read_line(&mut buffer) {
                Ok(_) => {
                    if !buffer.trim().is_empty() {
                        match sender.send(buffer.trim().to_string()) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("[ERROR] -> {}", e);
                            }
                        };
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    } else {
                        thread::sleep(std::time::Duration::from_millis(100));
                        eprintln!("[ERROR] -> {}", e);
                        break;
                    }
                }
            }
            thread::sleep(std::time::Duration::from_millis(100));
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
            eprintln!("[ERROR] -> {}", e);
        }
    };
}
