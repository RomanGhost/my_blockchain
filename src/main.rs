mod coin;

use coin::server::Server;

fn main() {
    let mut server = Server::new();
    server.run("localhost:7878");
    println!("Server shutting down...");
}