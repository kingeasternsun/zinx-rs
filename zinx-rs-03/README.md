# znet
使用golang定义接口，返回slice非常简单，golang会给予逃逸分析进行内存的分配和管理，但是在rust中就不能直接这样搞了，需要使用声明周期如下

声明trait的时候，生命周期‘a，要在T前面
```rust
pub trait IRquest<'a,T> {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<T> where T:Iconnection;
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8];
}

```

定义 Request的时候，需要标准生命周期参数，因为 data是一个引用，不然编译器不知道所引用对象的生命周期的长度
```rust
pub struct Request<'a> {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: &'a [u8],            //客户端请求的数据
}
```

实现trait的时候，要极限impl后面声明‘a
```rust
impl <'a> IRquest<'a,ConnectionSync> for Request<'a> {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }

    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8] {
        self.data
    }
}
```

IRouter 这里使用了默认实现，就不需要像golag版本那样创建 BaesRouter了
```rust
pub trait IRouter<'a,R,T>  where R:IRquest<'a,T>{
    fn pre_handler(&self ,request:R){}
    fn handler(&self,request:R){}
    fn post_hander(&self,request:R){}
}
```