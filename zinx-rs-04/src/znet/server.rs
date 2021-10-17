#![allow(non_snake_case)]
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;
use std::{net::TcpListener, net::TcpStream, thread};

// use tokio::sync::futures::Notified;

use crate::ziface::iconnection::Iconnection;
use crate::ziface::iserver::Iserver;
use crate::znet::connection::ConnectionSync;
use crate::znet::Request;
use crate::znet::RouterSync;
// use crate::IRouter;
pub struct Server {
    // 服务器名称
    Name: String,
    // tcp4 or other
    IPVersion: String,
    // 服务绑定的IP地址
    IP: String,
    // 服务绑定的端口
    Port: u32,

    Router: Option<RouterSync>,
}

impl Server {
    pub fn new(name: String, ip_version: String, ip: String, port: u32) -> Self {
        Server {
            Name: name,
            IPVersion: ip_version,
            IP: ip,
            Port: port,
            Router: None,
        }
    }
}

impl Iserver for Server {
    type R = Request;
    fn Start(&self) -> std::io::Result<()> {
        println!(
            "server {} start listenner {} {} {:?} ",
            self.Name, self.IPVersion, self.IP, self.Port
        );
        let listener = TcpListener::bind(format!("{}:{}", self.IP, self.Port))?;
        // 已经监听成功
        // has listen suc
        // TODO 这里如果使用 thread::spawn 就会报声明周期错误
        // thread::spawn(move || {
        // 启动server网络连接业务
        // start server to accept connection
        let mut conn_id = 0;
        loop {
            // block to wait the client to connect
            match listener.accept() {
                Ok((stream, socket_addr)) => {
                    println!("remote {:?}", socket_addr);
                    // todo: set the max connection ,if exceed the threashold, close this connection
                    // todo: there should be one handler binded to this conn
                    // just make a echo server
                    let conn = Arc::new(ConnectionSync::new(
                        stream,
                        socket_addr,
                        conn_id,
                        Arc::new(callbacke_to_client_sync),
                        Arc::clone(self.Router.as_ref().unwrap()),
                    ));
                    thread::spawn(move || {
                        conn.start();
                    });

                    conn_id += 1;
                }
                Err(err) => {
                    println!("{}", err);
                    // return;
                }
            }
        }
        // });
        Ok(())
    }

    fn Stop(&mut self) {
        println!("[STOP] {}", self.Name);
        //TODO other clear job
    }

    fn Serve(&mut self) {
        self.Start().unwrap();
        //TODO other job
        // loop {
        //     thread::sleep(time::Duration::from_secs(10));
        // }
    }

    fn AddRouter(&mut self, router: RouterSync) {
        self.Router = Some(Arc::clone(&router))
    }
}

// fn callbacke_to_client(stream: &mut TcpStream, data: &[u8], n: usize) -> std::io::Result<usize> {
//     stream.write(&data[..n])
// }

fn callbacke_to_client_sync(
    stream: Arc<Mutex<TcpStream>>,
    data: &[u8],
    n: usize,
) -> std::io::Result<usize> {
    stream.lock().unwrap().write(&data[..n])
}
/*

error[E0277]: `(dyn for<'r, 's> Fn(&'r std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be sent between threads safely
   --> src/znet/server.rs:55:25
    |
55  |                         thread::spawn(move || {
    |                         ^^^^^^^^^^^^^ `(dyn for<'r, 's> Fn(&'r std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be sent between threads safely
*/
