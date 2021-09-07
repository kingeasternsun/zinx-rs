use std::string;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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

// async 暂时不支持 trait
impl Server {
    async fn Start(&mut self) -> std::io::Result<()> {
        println!("server start listenner {:?} {:?} ", self.IP, self.Port);
        let listener = TcpListener::bind(format!("{}:{}", self.IP, self.Port)).await?;
        // 已经监听成功
        // has listen suc

        // 启动server网络连接业务
        // start server to accept connection
        loop {
            // block to wait the client to connect
            let (mut stream, _) = listener.accept().await?;

            tokio::spawn(async move {
                let mut buf = BytesMut::with_capacity(256);
                loop {
                    let n = stream.read_buf(&mut buf).await.unwrap();
                    if n == 0 {
                        return;
                    }

                    println!("{}", String::from_utf8_lossy(&buf[..]));

                    stream.write_all_buf(&mut buf).await.unwrap();
                }
            });
        }

    }

    fn Stop(&mut self) {
        println!("[STOP] {}", self.Name);
        //TODO other clear job
    }

    pub async fn Serve(&mut self) {
        self.Start().await.unwrap();
    }
}
