use zinx_rs::err::Result;
use zinx_rs::znet::Server;
use zinx_rs::Iserver;

pub fn main() -> Result<()> {
    let mut ser = Server::new(
        String::from("name"),
        String::from("ipv4"),
        String::from("127.0.0.1"),
        9090,
    );
    ser.Serve();

    Ok(())
}
