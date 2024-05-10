use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

mod old;

struct Server {
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self, addr: &str, port: &str) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(format!("{}:{}", addr, port))?;
        println!("[INFO] -> listening on {addr}:{port}");

        for stream in listener.incoming() {
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

    fn add_client(&self, stream: TcpStream) {
        let stream = Arc::new(Mutex::new(stream));
        let mut username = read_line_from_stream(&stream.lock().unwrap()).unwrap();
        let mut clients = self.clients.lock().unwrap();

        while clients.contains_key(&username) {
            stream
                .lock()
                .unwrap()
                .write_all(b"Username already taken, please try again\n")
                .unwrap();
            username = read_line_from_stream(&stream.lock().unwrap()).unwrap();
        }

        stream
            .lock()
            .unwrap()
            .write_all(b"Welcome to the chat!\n")
            .unwrap();

        println!("[INFO] -> new client: {}", username);
        clients.insert(username.clone(), stream.clone());
        spawn_client_handler(stream.clone(), self.clients.clone(), username);
    }
}

fn spawn_client_handler(
    stream: Arc<Mutex<TcpStream>>,
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    username: String,
) {
    thread::spawn(move || {
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
                        println!("[MSG] -> {}: {}", username, msg.trim());
                        let clients = clients.lock().unwrap();
                        for (name, client) in clients.iter() {
                            if *name != username {
                                msg = format!("{}: {}", username, msg);
                                let mut client = client.lock().unwrap();
                                client.write_all(msg.as_bytes()).unwrap();
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
    let mut server = Server::new();
    match server.run("localhost", "6969") {
        Ok(_) => {}
        Err(e) => {
            println!("[ERROR]: {}", e);
        }
    };
}
