#![allow(non_snake_case)]
// use crossbeam::channel;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9090")?;
    loop {
        // let req  = String::from("hello kes").into_bytes();
        match stream.write(b"hello king eastern sun") {
            Ok(_n) => {
                let mut buf = vec![0; 256];
                match stream.read(&mut buf) {
                    Ok(n) => {
                        println!("recv {} {}", n, String::from_utf8(buf).unwrap());
                    }
                    Err(err) => {
                        println!("{}", err);
                        return Err(err);
                    }
                }
            }
            Err(err) => println!("{}", err),
        }

        thread::sleep(Duration::from_secs(1));
    }
}
