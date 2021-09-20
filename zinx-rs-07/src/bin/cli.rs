#![allow(non_snake_case)]
// use crossbeam::channel;
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use std::io::Cursor;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use zinx_rs::util::Error;
use zinx_rs::util::Message;
use zinx_rs::ziface::IDataPack;
use zinx_rs::DataPack;

fn parse_message(buffer: &mut BytesMut) -> std::result::Result<Option<Message>, Error> {
    use zinx_rs::util::Error::Incomplete;
    let mut buf = Cursor::new(&buffer[..]);
    match DataPack::check(&mut buf) {
        Ok(_) => {
            buf.set_position(0);
            let msg = DataPack::Unpack(&mut buf)?;
            let len = buf.position() as usize;
            buffer.advance(len);
            Ok(Some(msg))
        }
        Err(Incomplete) => Ok(None),
        Err(err) => Err(err),
    }
}

fn read_message(
    stream: &mut TcpStream,
    buffer: &mut BytesMut,
) -> std::result::Result<Option<Message>, Error> {
    loop {
        if let Some(msg) = parse_message(buffer)? {
            return Ok(Some(msg));
        }

        let mut buf = vec![0; 256];
        match stream.read(&mut buf[..]) {
            Err(err) => {
                println!("{}", err);
                return Err(Error::Other(Box::new(err)));
            }
            Ok(0) => {
                if buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
            Ok(n) => {
                buffer.put_slice(&buf[..n]);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9090")?;
    let data: Vec<u8> = "hello king eastern sun".bytes().collect();
    let mut msg_id = 1;
    let mut msg = Message::new(msg_id, data);
    loop {
        msg.SetMsgId(msg_id % 3);
        let buf = DataPack::Pack(&msg)?;
        println!("send {}", msg);
        match stream.write(&buf) {
            Ok(_n) => {
                let mut buffer = BytesMut::with_capacity(1024);
                match read_message(&mut stream, &mut buffer) {
                    Ok(None) => {
                        println!("remote close");
                        break;
                    }
                    Ok(Some(msg)) => {
                        println!("recv {}", msg);
                    }
                    Err(err) => {
                        println!("{}", err);
                        break;
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
                break;
            }
        }

        msg_id += 1;
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
