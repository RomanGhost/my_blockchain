use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use crate::coin::server::protocol::message::r#type::Message;
use crate::coin::server::server::Server;

pub fn server_thread(address: String) -> (Server, Receiver<Message>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let mut server = Server::new(tx);
    let server_clone = server.clone();

    // Input server address and run it in a separate thread.
    let server_thread = thread::spawn(move || {
        server.run(address.as_ref());
    });

    let rx_server = rx;

    (server_clone, rx_server, server_thread)
}
