use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(
    stream: Arc<Mutex<TcpStream>>,
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    username: String,
) {
    // this is ugly but it works
    let mut binding = stream.lock().unwrap().try_clone().unwrap();
    let mut reader = BufReader::new(&mut binding);

    loop {
        let mut msg = String::new();
        match reader.read_line(&mut msg) {
            Ok(_) => {
                if !msg.trim().is_empty() {
                    println!("[{username}]: {}", msg.trim());
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
                eprintln!("Error: {}", e);
                break;
            }
        };
    }
}

fn read_line_from_stream(stream: &TcpStream) -> String {
    let reader = BufReader::new(stream);
    reader.lines().next().unwrap().unwrap()
}

fn main() {
    let listener = TcpListener::bind("localhost:6969").unwrap();
    println!("[STARTED] localhost:6969");

    let clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let clients = clients.clone();
                let mut username = read_line_from_stream(&stream);

                while clients.lock().unwrap().contains_key(&username) {
                    stream
                        .write_all("Name taken, enter a new name.\n".as_bytes())
                        .unwrap();
                    username = read_line_from_stream(&stream);
                }
                stream
                    .write_all("Successfully connected.\n".as_bytes())
                    .unwrap();

                println!("[{username}] -> connected");

                let shared_stream = Arc::new(Mutex::new(stream));
                clients
                    .lock()
                    .unwrap()
                    .insert(username.clone(), shared_stream.clone());

                handles.push(thread::spawn(move || {
                    handle_client(shared_stream, clients, username);
                }))
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
