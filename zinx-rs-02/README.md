Zinx-V0.2-简单的连接封装与业务绑定 https://www.kancloud.cn/aceld/zinx/1960214

# 同步实现

在 connection 模块中，如果 handler_api 的类型 HandlerFn 的定义如下
```rust
type HandlerFn = Arc<dyn Fn(& mut TcpStream, &[u8], usize) -> std::io::Result<usize>>;
```
上面定义中没有加 Send 和 Sync 的trait 限定，
如果想要用下面的方式调用,也就是把Connection的创建放在spawn外面，如下：

```rust
                        let mut conn = Connection::new(
                            stream,
                            socket_addr,
                            conn_id,
                            Arc::new(callbacke_to_client),
                        );
                        thread::spawn(move || {

                            conn.start();
                        });

```
就会报下面的错误
```
error[E0277]: `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be sent between threads safely
   --> src/znet/server.rs:75:25
    |
75  |                         thread::spawn(move || {
    |                         ^^^^^^^^^^^^^ `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be sent between threads safely

617 |     F: Send + 'static,
    |        ---- required by this bound in `std::thread::spawn`
    |
    = help: the trait `Send` is not implemented for `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)`
    = note: required because of the requirements on the impl of `Send` for `Arc<(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)>`
    = note: required because it appears within the type `znet::connection::Connection`
    = note: required because it appears within the type `[closure@src/znet/server.rs:75:39: 77:26]`

error[E0277]: `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be shared between threads safely
   --> src/znet/server.rs:75:25
    |
75  |                         thread::spawn(move || {
    |                         ^^^^^^^^^^^^^ `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)` cannot be shared between threads safely

617 |     F: Send + 'static,
    |        ---- required by this bound in `std::thread::spawn`
    |
    = help: the trait `Sync` is not implemented for `(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)`
    = note: required because of the requirements on the impl of `Send` for `Arc<(dyn for<'r, 's> Fn(&'r mut std::net::TcpStream, &'s [u8], usize) -> std::result::Result<usize, std::io::Error> + 'static)>`
    = note: required because it appears within the type `znet::connection::Connection`
    = note: required because it appears within the type `[closure@src/znet/server.rs:75:39: 77:26]`
```

因为 HandlerFn 没有 Send  和 Sync trait，所以 conn.handler_api 不能安全的在线程间移动，解决方法两种

一种是在 server 模块中，只能这样来使用Connection
```rust
                        thread::spawn(move || {
                            let mut conn = Connection::new(
                                stream,
                                socket_addr,
                                conn_id,
                                Arc::new(callbacke_to_client),
                            );
                            conn.start();
                        });
```

第二种也是推荐的做法，HandlerFn 加上 Send Sync trait
```rust
type HandlerFn = Arc<dyn Fn(&mut TcpStream, &[u8], usize) -> std::io::Result<usize> + Send + Sync>;
```
## spawn method

如果要在method中spawn 其他的method，两种方案
1. 使用 crossbeam::scope ，参见Connection::start_scope()
2. 当前对象要用Arc<Mutex> 封装，参见 https://users.rust-lang.org/t/how-to-use-self-while-spawning-a-thread-from-method/8282

# 异步实现
由于 tikio.net.TcpStream 执行写入也是 async，只能在async中调用，所以写入操作无法作为回调函数的一部分

所以async实现中，回调函数只包含数据的逻辑处理，数据写回仍然在Connection中完成



Connection 结构体定义中的 handler_api 必须要加 + Send + Sync 限定
```rust
handler_api: Arc<dyn Fn(& mut TcpStream, &mut[u8], usize) -> std::io::Result<usize> + Send + Sync>,
```

另外下面这个 也是 非 Send的需要定位原因
```rust
    // 当前连接的关闭状态
    // is_closed: Arc<Mutex<bool>>,
```

# 参考

https://users.rust-lang.org/t/how-to-share-an-arc-dyn-fn-u8-between-threads/49329/3
