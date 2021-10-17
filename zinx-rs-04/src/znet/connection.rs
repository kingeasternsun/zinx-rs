#![allow(non_snake_case, dead_code)]
use crossbeam::channel;
use crossbeam::scope;
use crossbeam_channel::select;
use std::io::Read;
// use std::io::Write;
use crate::util::ConnID;
use crate::ziface::Iconnection;
use crate::znet::Request;
use crate::znet::RouterSync;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
/// connection
///
/// 1. 注意要 use std::io::Read;std::io::Write 这两个trait
/// 2. Connection中的conn 使用 try_clone 进行复制
/// 3. handler_api Box<dyn ..>
/// 4. 调用handler_api 使用的时候要 把self.handler_api 用括号包裹着 (self.handler_api)(&self.conn,&buf[..],n)

pub struct Connection {
    // 当前连接的tcpstream
    conn: TcpStream,
    // 对端地址
    socket_addr: SocketAddr,
    // 当前连接的ID 唯一
    conn_id: ConnID,
    // 当前连接的关闭状态
    is_closed: Arc<Mutex<bool>>,

    // 用来通知当前连接已经退出或停止
    exit_buff_chan: channel::Sender<bool>,
    receiver: channel::Receiver<bool>,
    handler_api: HandlerFn,
}

type HandlerFn = Arc<dyn Fn(&mut TcpStream, &[u8], usize) -> std::io::Result<usize> + Send + Sync>;

impl Connection {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: HandlerFn,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        Connection {
            conn: stream,
            socket_addr: socket_addr,
            conn_id: ConnID::new(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            handler_api: f,
        }
    }

    fn start_read(&mut self) {
        loop {
            let mut buf = vec![0; 256];
            match self.conn.read(&mut buf) {
                // !!! note if rev Ok(0) ,should break
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    println!(
                        "{:?} recv {}",
                        self.conn_id,
                        String::from_utf8(buf.clone()).unwrap()
                    );
                    // match self.conn.write(&buf[..n]) {
                    match (self.handler_api)(&mut self.conn, &buf[..], n) {
                        Ok(n) => println!("{:?} write back{}", self.conn_id, n),
                        Err(_) => break,
                    }
                }
                Err(err) => {
                    println!("{:?}{}", self.conn_id, err);
                    break;
                }
            }
        }

        self.stop();
    }

    // 启动连接，让当前连接开始工作
    pub fn start(&mut self) {
        println!("{:?} start", self.conn_id);
        let r1 = self.receiver.clone();

        thread::spawn(move || {
            select! {
                recv(r1)->msg => {
                    println!("close {}",msg.unwrap());
                    return
                } ,
            }
        });

        self.start_read();
    }

    // 利用crossbeam 来 spawn 调用自身的其他方法
    pub fn start_scope(&mut self) {
        println!("{:?} start", self.conn_id);
        let r1 = self.receiver.clone();

        scope(|scope| {
            scope.spawn(|_| {
                self.start_read();
            });
        })
        .unwrap();

        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());
                return
            } ,
        }
    }

    pub fn stop(&self) {
        let mut close = self.is_closed.lock().unwrap();
        if close.eq(&true) {
            return;
        }

        //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        *close = true;
        match self.conn.shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }

        // 通知从 tcp stream读数据的业务关闭
        self.exit_buff_chan.send(true).unwrap();
        println!("[STOP] {:?}", self.conn_id);
    }

    //从当前连接获取原始的tcp stream
    pub fn get_tcp_stream(&self) -> TcpStream {
        self.conn.try_clone().unwrap()
    }

    // 获取当前连接ID
    pub fn get_conn_id(&self) -> ConnID {
        self.conn_id
    }

    // 获取远程客户端地址信息
    pub fn remote_addr(&self) -> SocketAddr {
        self.socket_addr.clone()
    }
}

pub struct ConnectionSync {
    // 当前连接的tcpstream
    conn: Arc<Mutex<TcpStream>>,
    // 对端地址
    socket_addr: SocketAddr,
    // 当前连接的ID 唯一
    conn_id: ConnID,
    // 当前连接的关闭状态
    is_closed: Arc<Mutex<bool>>,

    // 用来通知当前连接已经退出或停止
    exit_buff_chan: channel::Sender<bool>,
    receiver: channel::Receiver<bool>,
    handler_api: HandlerFnSync,
    Router: RouterSync,
}

type HandlerFnSync =
    Arc<dyn Fn(Arc<Mutex<TcpStream>>, &[u8], usize) -> std::io::Result<usize> + Send + Sync>;

impl ConnectionSync {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: HandlerFnSync,
        router: RouterSync,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        ConnectionSync {
            conn: Arc::new(Mutex::new(stream)),
            socket_addr: socket_addr,
            conn_id: ConnID::new(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            handler_api: f,
            Router: router,
        }
    }

    fn start_read(self: &Arc<Self>) {
        loop {
            let mut buf = vec![0; 256];
            let res;
            {
                let mut conn = self.conn.lock().unwrap();
                res = conn.read(&mut buf);
                // release lock
            }
            match res {
                // !!! note if rev Ok(0) ,should break
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    println!(
                        "{:?} recv {}",
                        self.conn_id,
                        String::from_utf8(buf.clone()).unwrap()
                    );
                    let request =
                        Request::new(Arc::clone(self), &buf[0..n]);

                    self.Router.pre_handler(request.clone());
                    // match self.conn.write(&buf[..n]) {
                    match (self.handler_api)(self.conn.clone(), &buf[..], n) {
                        Ok(n) => {
                            println!("{:?} write back {} bytes", self.conn_id, n);
                            self.Router.post_hander(request);
                        }
                        Err(_) => break,
                    }
                }
                Err(err) => {
                    println!("{:?}{}", self.conn_id, err);
                    break;
                }
            }
        }

        self.stop();
    }

    // 利用crossbeam 来 spawn 调用自身的其他方法
    pub fn start_scope(self: &Arc<Self>) {
        println!("{:?} start", self.conn_id);
        let r1 = self.receiver.clone();

        scope(|s| {
            s.spawn(|_| {
                self.start_read();
            });
        })
        .unwrap();

        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());
                return
            } ,
        }
    }
}

impl Iconnection for ConnectionSync {
    // 启动连接，让当前连接开始工作
    fn start(self: &Arc<Self>) {
        println!("{:?} start", self.conn_id);

        let c = Arc::clone(&self);
        thread::spawn(move || {
            c.start_read();
        });

        let r1 = self.receiver.clone();

        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());
                return
            } ,
        }
    }

    fn stop(&self) {
        let mut close = self.is_closed.lock().unwrap();
        if close.eq(&true) {
            return;
        }

        //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        *close = true;
        match self.conn.lock().unwrap().shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }

        // 通知从 tcp stream读数据的业务关闭
        self.exit_buff_chan.send(true).unwrap();
        println!("[STOP] {:?} {}", self.get_conn_id(), self.remote_addr());
    }

    //从当前连接获取原始的tcp stream
    fn get_tcp_stream(&self) -> Arc<Mutex<TcpStream>> {
        self.conn.clone()
    }

    // 获取当前连接ID
    fn get_conn_id(&self) -> ConnID {
        self.conn_id
    }

    // 获取远程客户端地址信息
    fn remote_addr(&self) -> SocketAddr {
        self.socket_addr.clone()
    }
}
