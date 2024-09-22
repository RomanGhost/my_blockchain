mod coin;

use coin::blockchain::block::Block;
use coin::server::Server;
use coin::message::BlockMessage;

fn main() {
    let mut server = Server::new();
    server.run("localhost:7878");
    println!("Server shutting down");
}