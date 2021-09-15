#![allow(non_snake_case)]
use std::sync::Arc;
use zinx_rs::err::Result;
use zinx_rs::znet::PingRouter;
use zinx_rs::znet::Server;
use zinx_rs::Iserver;

pub fn main() -> Result<()> {
    let mut ser = Server::new(
        String::from("kingeasternsun"),
        String::from("ipv4"),
        String::from("127.0.0.1"),
        9090,
    );
    ser.AddRouter(Arc::new(Box::new(PingRouter {})));
    ser.Serve();

    Ok(())
}
