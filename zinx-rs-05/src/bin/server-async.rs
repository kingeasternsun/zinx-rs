#![allow(non_snake_case)]
use std::sync::Arc;
use structopt_toml::StructOptToml;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use zinx_rs::util::Opt;
use zinx_rs::znet_async::PingRouter;
use zinx_rs::znet_async::Server;
#[tokio::main]
async fn main() {
    // read the contect of file
    let mut buf = Vec::new();
    {
        let mut f = File::open("opt.toml").await.expect("opt.toml");
        f.read_to_end(&mut buf).await.unwrap();
    }

    // parse the bytes
    let opt = Opt::from_args_with_toml(&String::from_utf8_lossy(&buf)).expect("toml parse failed");

    println!("{:?}", opt);
    let mut ser = Server::new(
        opt.name,
        String::from("ipv4"),
        String::from("127.0.0.1"),
        opt.port,
    );
    ser.AddRouter(Arc::new(Box::new(PingRouter {})));
    ser.Serve().await
}
