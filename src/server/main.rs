use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use chat::{JoinFailureReason, Message};

struct Client {
    username: String,
    id: u32,
    stream: TcpStream,
}

struct Server {
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    socket: TcpListener,
}

impl Server {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let socket = TcpListener::bind(addr)?;

        Ok(Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            socket,
        })
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        println!("[INFO] -> listening on {}", self.socket.local_addr()?);

        for stream in self.socket.incoming() {
            match stream {
                Ok(stream) => {
                    self.add_client(stream);
                }
                Err(e) => {
                    eprintln!("[ERROR] -> {}", e);
                }
            }
        }
        Ok(())
    }

    fn add_client(&self, mut stream: TcpStream) {
        let join_msg: Message =
            serde_json::from_str(read_line_from_stream(&stream).unwrap().as_str()).unwrap();

        let mut username = match join_msg {
            Message::JoinRequest { username } => username,
            _ => {
                eprintln!("[ERROR] -> unexpected message from client");
                return;
            }
        };

        let mut clients = self.clients.lock().unwrap();

        while clients.contains_key(&username) {
            let resp = Message::JoinResponseFailure {
                reason: JoinFailureReason::UsernameInUse,
            };
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .unwrap();

            username = match serde_json::from_str(read_line_from_stream(&stream).unwrap().as_str())
                .unwrap()
            {
                Message::JoinRequest { username } => username,
                _ => {
                    eprintln!("[ERROR] -> unexpected message from client");
                    return;
                }
            };
        }

        let resp = Message::JoinResponseSuccess {
            user_id: clients.len() as u32,
        };
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .unwrap();

        println!("[INFO] -> new client: {}", username);
        stream.write_all(b"Welcome to the chat!\n").unwrap();
        clients.insert(username.clone(), Arc::new(Mutex::new(stream)));
        spawn_client_handler(self.clients.clone(), username);
    }
}

fn spawn_client_handler(
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    username: String,
) {
    thread::spawn(move || {
        let stream = clients.lock().unwrap().get(&username).unwrap().clone();
        let mut binding = stream.lock().unwrap().try_clone().unwrap();
        let mut reader = BufReader::new(&mut binding);
        loop {
            let mut msg = String::new();
            match reader.read_line(&mut msg) {
                Ok(0) => {
                    println!("[INFO] -> {} disconnected", username);
                    clients.lock().unwrap().remove(&username);
                    break;
                }
                Ok(_) => {
                    if !msg.is_empty() {
                        let msg: Message = serde_json::from_str(msg.as_str()).unwrap();
                        match msg {
                            Message::Message {
                                user_id: _,
                                content,
                            } => {
                                println!("[MSG] -> {}: {}", username, content.trim());
                                for (name, client) in clients.lock().unwrap().iter() {
                                    if *name != username {
                                        let msg = format!("{}: {}", username, content);
                                        let mut client = client.lock().unwrap();
                                        client.write_all(msg.as_bytes()).unwrap();
                                    }
                                }
                            }
                            _ => {
                                eprintln!("[ERROR] -> unexpected message from client");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR]: {}", e);
                    break;
                }
            }
        }
    });
}

fn read_line_from_stream(stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn main() {
    match Server::new("localhost:6969") {
        Ok(mut server) => match server.run() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[ERROR] -> {}", e);
            }
        },
        Err(e) => {
            eprintln!("[ERROR] -> {}", e);
        }
    }
}
