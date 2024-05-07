use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(
    stream: Arc<Mutex<TcpStream>>,
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
) {
    // this is ugly but it works
    let mut binding = stream.lock().unwrap().try_clone().unwrap();
    let mut reader = BufReader::new(&mut binding);

    loop {
        let mut msg = String::new();
        match reader.read_line(&mut msg) {
            Ok(_) => {
                if !msg.trim().is_empty() {
                    println!("Received message: {}", msg.trim());
                    // echo back
                    stream.lock().unwrap().write_all(msg.as_bytes()).unwrap();
                    // send the message to all clients
                    let clients = clients.lock().unwrap();
                    for client in clients.values() {
                        let mut client = client.lock().unwrap();
                        client.write_all(msg.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        };
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:6969").unwrap();
    println!("Server listening on port 6969");

    let clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients = clients.clone();
                let shared_stream = Arc::new(Mutex::new(stream));

                clients.lock().unwrap().insert(
                    shared_stream
                        .lock()
                        .unwrap()
                        .peer_addr()
                        .unwrap()
                        .to_string(),
                    shared_stream.clone(),
                );

                handles.push(thread::spawn(move || {
                    handle_client(shared_stream, clients);
                }))
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
