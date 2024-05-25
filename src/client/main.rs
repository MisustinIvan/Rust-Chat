use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver},
    thread,
};

use chat::Message;

#[allow(dead_code)]
struct Client {
    id: u32,
    stream: TcpStream,
}

impl Client {
    pub fn new(addr: &str, username: &str) -> Result<Self, io::Error> {
        // create a new stream
        let stream = TcpStream::connect(addr);
        println!("listening on {addr} as {username}");

        match stream {
            Ok(mut stream) => {
                // create the join message
                let msg = Message::JoinRequest {
                    username: username.to_string(),
                };

                // send the join message
                stream
                    .write_all(format!("{}\n", serde_json::to_string(&msg).unwrap()).as_bytes())?;

                // create a reader and a buffer
                let mut reader = BufReader::new(&stream);
                let mut buffer = String::new();

                // read the response
                reader.read_line(&mut buffer)?;

                // set nonblocking
                stream.set_nonblocking(true)?;

                let id: u32;
                let response: Message = serde_json::from_str(&buffer).unwrap();
                match response {
                    Message::JoinResponseSuccess { user_id } => {
                        id = user_id;
                        println!("[INFO] -> joined as {username}:{user_id}");
                    }
                    Message::JoinResponseFailure { reason } => {
                        return Err(io::Error::new(io::ErrorKind::Other, reason.to_string()));
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unexpected response from server",
                        ));
                    }
                }

                Ok(Client { stream, id })
            }
            Err(e) => Err(e),
        }
    }

    pub fn run(&mut self) {
        let stdin_channel = self.spawn_stdin_reader();
        let server_reader = self.spawn_server_reader();

        loop {
            match stdin_channel.try_recv() {
                Ok(msg) => match msg.trim() {
                    "/exit" => {
                        break;
                    }
                    msg if msg.starts_with("/msg") => {
                        let parts = msg.split_whitespace().collect::<Vec<&str>>();

                        if parts.len() < 3 {
                            println!("[ERROR] -> empty message");
                            continue;
                        }

                        let target = parts[1];
                        let content = parts[2..].join(" ");

                        let message = Message::PrivateMessage {
                            user_id: self.id,
                            target_name: target.to_string(),
                            content,
                        };

                        self.send_message(format!(
                            "{}\n",
                            serde_json::to_string(&message).unwrap()
                        ));
                    }
                    "/list" => {
                        let message = Message::ListRequest { user_id: self.id };
                        self.send_message(format!(
                            "{}\n",
                            serde_json::to_string(&message).unwrap()
                        ));
                    }
                    _ => {
                        let message = Message::Message {
                            user_id: self.id,
                            content: msg.to_string(),
                        };
                        self.send_message(format!(
                            "{}\n",
                            serde_json::to_string(&message).unwrap()
                        ));
                    }
                },
                Err(_) => {}
            }

            match server_reader.try_recv() {
                Ok(msg) => {
                    println!("{msg}");
                }
                Err(_) => {}
            }
        }

        println!("[INFO] -> exitting");
        self.stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    fn send_message(&mut self, msg: String) {
        match self.stream.write_all(msg.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[ERROR] sending message -> {e}");
            }
        }
    }

    fn spawn_stdin_reader(&self) -> Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let stdin = io::stdin();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            match stdin.read_line(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[ERROR] reading from stream -> {}", e);
                }
            };
            if !buffer.trim().is_empty() {
                match sender.send(buffer) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("[ERROR] error sending through channel -> {}", e);
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
                                eprintln!("[ERROR] error sending through channel -> {}", e);
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
                        eprintln!("[ERROR] error reading line -> {}", e);
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

    let addr = match args.get(2) {
        Some(addr) => addr,
        None => "localhost:6969",
    };

    match Client::new(addr, username) {
        Ok(mut client) => client.run(),
        Err(e) => {
            eprintln!("[ERROR] error creating client -> {}", e);
        }
    };
}
