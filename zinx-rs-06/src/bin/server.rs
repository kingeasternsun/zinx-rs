#![allow(non_snake_case)]
use std::fs::File;
use std::io::Read;


use structopt_toml::StructOptToml;
use zinx_rs::err::Result;
use zinx_rs::util::Opt;
use zinx_rs::util::MsgHandle;
use zinx_rs::znet::router::*;
use zinx_rs::znet::Server;
use zinx_rs::Iserver;

pub fn main() -> Result<()> {
    // read the contect of file
    let mut buf = Vec::new();
    {
        let mut f = File::open("opt.toml").expect("opt.toml");
        f.read_to_end(&mut buf).unwrap();
    }

    // parse the bytes
    let opt = Opt::from_args_with_toml(&String::from_utf8_lossy(&buf)).expect("toml parse failed");
    println!("{:?}", opt);

    let mut msgHandle = MsgHandle::new();
    msgHandle.AddRouter(0, Box::new(PingRouter {}));
    msgHandle.AddRouter(1, Box::new(OneRouter {}));
    msgHandle.AddRouter(2, Box::new(TwoRouter {}));

    let mut ser = Server::new(
        opt.name,
        String::from("ipv4"),
        String::from("127.0.0.1"),
        opt.port,
        msgHandle,
    );
    // ser.AddRouter(Arc::new(Box::new(PingRouter {})));
    ser.Serve();

    Ok(())
}
