由于 tikio.net.TcpStream 执行写入也是 async，只能在async中调用，所以写入操作无法作为回调函数的一部分

所以async实现中，回调函数只包含数据的逻辑处理，数据写回仍然在Connection中完成


https://users.rust-lang.org/t/how-to-share-an-arc-dyn-fn-u8-between-threads/49329/3

Connection 结构体定义中的 handler_api 必须要加 + Send + Sync 限定
```rust
handler_api: Arc<dyn Fn(& mut TcpStream, &mut[u8], usize) -> std::io::Result<usize> + Send + Sync>,
```

另外下面这个 也是 非 Send的需要定位原因
```rust
    // 当前连接的关闭状态
    // is_closed: Arc<Mutex<bool>>,
```