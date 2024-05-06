use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use chat::User;

const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

struct Client {
    connection: TcpStream,
    user: User,
}

impl Client {
    pub fn new(addr: &str, port: &str, name: &str) -> Self {
        let mut connection = TcpStream::connect(format!("{}:{}", addr, port)).unwrap();
        connection
            .write_all(format!("{}{}{}\n", RED, name, RESET).as_bytes())
            .unwrap();
        let reader = BufReader::new(&connection);
        let id: i32 = reader.lines().next().unwrap().unwrap().parse().unwrap();
        let user = User {
            name: name.to_string(),
            id: id as u32,
        };

        println!("Connected to the server as {}->{}", user.name, user.id);
        Client { connection, user }
    }

    pub fn run(&mut self) {
        loop {
            // read from stdin
            let mut msg = String::new();
            std::io::stdin().read_line(&mut msg).unwrap();
            let msg = msg.trim();

            // send message to server
            self.connection.write_all(msg.as_bytes()).unwrap();
        }
    }
}

fn main() {
    let mut client = Client::new("localhost", "8080", "Yyvan");
    client.run();
}
