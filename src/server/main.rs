use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use chat::{Message, User};

#[allow(dead_code)]
#[derive(Debug)]
struct ChatServer {
    users: Vec<User>,
    messages: Vec<Message>,
    connections: HashMap<u32, TcpStream>,
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            users: Vec::new(),
            messages: Vec::new(),
            connections: HashMap::new(),
        }
    }

    fn run(&mut self, addr: &str, port: u16) {
        let listener = TcpListener::bind(format!("{}:{}", addr, port)).unwrap();
        println!("Server listening on {}:{}", addr, port);

        for connection in listener.incoming() {
            let connection = connection.unwrap();

            let reader = BufReader::new(&connection);
            // haha jokes -> lets just agree that the first thing sent is the username
            let username = reader.lines().next().unwrap().unwrap();

            let user = User {
                name: username,
                id: self.users.len() as u32,
            };

            self.connections.insert(user.id, connection);
            self.users.push(user.clone());

            let resp = format!("{}\n\n", user.id);

            self.connections
                .get_mut(&user.id)
                .unwrap()
                .write_all(resp.as_bytes())
                .unwrap();

            println!("User {}->{} connected", user.id, user.name);
        }
    }
}

fn main() {
    let mut server = ChatServer::new();
    server.run("localhost", 8080);
}
