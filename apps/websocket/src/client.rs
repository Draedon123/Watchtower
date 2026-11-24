use tokio::net::TcpStream;

pub struct Client {
  connection: TcpStream,
}

impl Client {
  pub fn new(connection: TcpStream) -> Self {
    Self { connection }
  }
}
