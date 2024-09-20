use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub type Clients = Arc<Mutex<HashMap<String, ClientData>>>;

#[derive(Clone)]
pub struct ClientData {
    pub stream: Arc<Mutex<TcpStream>>,
}
