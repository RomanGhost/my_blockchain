use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use crate::coin::message::r#type::Message;
use crate::coin::server::Server;

pub fn server_thread(address: String) -> (Server, Receiver<Message>, JoinHandle<()>) {
    let (mut server, rx_server) = Server::new();
    let server_clone = server.clone();

    // Input server address and run it in a separate thread.
    let server_thread = thread::spawn(move || {
        server.run(address);
    });

    (server_clone, rx_server, server_thread)
}
