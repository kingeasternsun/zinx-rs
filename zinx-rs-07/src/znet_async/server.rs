#![allow(non_snake_case, dead_code)]

// use bytes::BytesMut;
use crate::util::ConnID;
use crate::util::Message;
use crate::util::MsgHandle;
use crate::znet_async::connection::ConnectionReader;
use crate::znet_async::connection::ConnectionWriter;
use crate::znet_async::Request;
use crossbeam::channel;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
pub struct Server {
    // 服务器名称
    Name: String,
    // tcp4 or other
    IPVersion: String,
    // 服务绑定的IP地址
    IP: String,
    // 服务绑定的端口
    Port: u32,

    // 消息管理 多路由
    Router: Arc<MsgHandle<Request>>,
}

impl Server {
    pub fn new(
        name: String,
        ip_version: String,
        ip: String,
        port: u32,
        router: MsgHandle<Request>,
    ) -> Self {
        Server {
            Name: name,
            IPVersion: ip_version,
            IP: ip,
            Port: port,
            Router: Arc::new(router),
        }
    }
}

// async 暂时不支持 trait
impl Server {
    async fn Start(&mut self) -> std::io::Result<()> {
        println!(
            "server {} start listenner {} {} {:?} ",
            self.Name, self.IPVersion, self.IP, self.Port
        );
        let listener = TcpListener::bind(format!("{}:{}", self.IP, self.Port)).await?;
        // 已经监听成功
        // has listen suc

        let mut conn_id: u32 = 0;
        // 启动server网络连接业务
        // start server to accept connection
        loop {
            // block to wait the client to connect
            let (stream, socket_addr) = listener.accept().await?;

            let (rd, wr) = stream.into_split();
            let (close_sx, close_rx) = channel::bounded(1);
            let (msg_sx, msg_rx) = channel::bounded(0);
            let is_closed = Arc::new(Mutex::new(false));

            //  创建 reader
            let mut reader = ConnectionReader::new(
                rd,
                ConnID::new(conn_id),
                is_closed.clone(),
                close_rx.clone(),
                msg_sx,
                Arc::clone(&self.Router),
            );
            tokio::spawn(async move { reader.start_reader().await });

            // 创建 writer
            tokio::spawn(async move {
                let mut writer = ConnectionWriter::new(
                    wr,
                    socket_addr,
                    ConnID::new(conn_id),
                    is_closed.clone(),
                    close_sx,
                    close_rx,
                    msg_rx,
                );
                writer.start_writer().await
            });

            conn_id += 1;
        }

        // Ok(())
    }

    fn Stop(&mut self) {
        println!("[STOP] {}", self.Name);
        //TODO other clear job
    }

    pub async fn Serve(&mut self) {
        self.Start().await.unwrap();
    }

    // pub fn AddRouter(&mut self,msgID:u32, router: RouterSync) {
    //     self.Router.AddRouter(msgID, router)
    // }
}

fn callbacke_to_client(
    _stream: &mut TcpStream,
    data: &mut [u8],
    _n: usize,
) -> std::io::Result<usize> {
    // stream.write(&data[..n]).await
    data.reverse();
    Ok(0)
}

fn callbacke_to_client_sync(
    _stream: Arc<Mutex<TcpStream>>,
    data: &mut [u8],
    _n: usize,
) -> std::io::Result<usize> {
    data.reverse();
    // tokio::spawn(async {
    //     let mut s = stream.lock().await;
    //     s.write(&data[..n]).await
    // });

    Ok(0)
}

fn callbacke_to_client_async(
    stream: Arc<Mutex<TcpStream>>,
    data: &'static [u8],
    n: usize,
) -> tokio::task::JoinHandle<std::io::Result<usize>> {
    tokio::spawn(async move {
        let mut s = stream.lock().await;
        s.write(&data[..n]).await
    })
}

// async fn process(socket: Arc<Mutex<TcpStream>>) {
//     let buf = BytesMut::with_capacity(256);
//     callbacke_to_client_async(socket,&buf[..],0).await;
// }

fn reverse_msg_handler(_stream: Arc<Mutex<TcpStream>>, data: &Message) -> std::io::Result<Message> {
    let mut msg = data.clone();
    msg.reverse();

    Ok(msg)
}
