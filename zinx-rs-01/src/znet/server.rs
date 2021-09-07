#![allow(non_snake_case)]
use crate::ziface::iserver::Iserver;
use std::io::{Read, Write};
use std::time;
use std::{net::TcpListener, thread};

pub struct Server {
    // 服务器名称
    Name: String,
    // tcp4 or other
    IPVersion: String,
    // 服务绑定的IP地址
    IP: String,
    // 服务绑定的端口
    Port: u32,
}

impl Server {
    pub fn new(name: String, ip_version: String, ip: String, port: u32) -> Self {
        Server {
            Name: name,
            IPVersion: ip_version,
            IP: ip,
            Port: port,
        }
    }
}

impl Iserver for Server {
    fn Start(&mut self) -> std::io::Result<()> {
        println!(
            "server {} start listenner {} {} {:?} ",
            self.Name, self.IPVersion, self.IP, self.Port
        );
        let listener = TcpListener::bind(format!("{}:{}", self.IP, self.Port))?;
        // 已经监听成功
        // has listen suc
        thread::spawn(move || {
            // 启动server网络连接业务
            // start server to accept connection
            loop {
                // block to wait the client to connect
                match listener.accept() {
                    Ok((mut stream, socket_addr)) => {
                        println!("remote {:?}", socket_addr);
                        // todo: set the max connection ,if exceed the threashold, close this connection
                        // todo: there should be one handler binded to this conn
                        // just make a echo server
                        thread::spawn(move || {
                            loop {
                                let mut buf = vec![0; 256];
                                match stream.read(&mut buf) {
                                    // !!! note if rev Ok(0) ,should break
                                    Ok(n) if n == 0 => return,
                                    Ok(n) => {
                                        println!(
                                            "recv {}",
                                            String::from_utf8(buf.clone()).unwrap()
                                        );
                                        match stream.write(&buf[..n]) {
                                            Ok(n) => println!("write back{}", n),
                                            Err(_) => return,
                                        }
                                    }
                                    Err(err) => {
                                        println!("{}", err);
                                        return;
                                    }
                                }
                            }
                        });
                    }
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                }
            }
        });
        Ok(())
    }

    fn Stop(&mut self) {
        println!("[STOP] {}", self.Name);
        //TODO other clear job
    }

    fn Serve(&mut self) {
        self.Start().unwrap();
        //TODO other job
        loop {
            thread::sleep(time::Duration::from_secs(10));
        }
    }
}
