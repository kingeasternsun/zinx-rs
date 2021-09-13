# znet
使用golang定义接口，返回slice非常简单，golang会给予逃逸分析进行内存的分配和管理，但是在rust中就不能直接这样搞了，需要使用声明周期如下

## IRquest

声明trait的时候，生命周期‘a，要在T前面
```rust
pub trait IRquest<'a,T> {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<T> where T:Iconnection;
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8];
}

```

改为关联类型
```rust
pub trait IRquest<'a> {
    type Conn;
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn> ;
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8];
}
```

## Request

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

使用关联类型变为
```rust
impl <'a> IRquest<'a> for Request<'a> {
    type Conn = ConnectionSync;
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
使用关联类型，相比trait限制，代码可读性上更强，而且从业务上看，对于一个Request来说，对应的IRquest就应该只有一种实现。


## IRouter

IRouter 这里使用了默认实现，就不需要像golag版本那样创建 BaesRouter了
```rust
pub trait IRouter<'a,R,T>  where R:IRquest<'a,T>{
    fn pre_handler(&self ,request:R){}
    fn handler(&self,request:R){}
    fn post_hander(&self,request:R){}
}
```

改为 关联类型
```rust
pub trait IRouter<'a,R>  where R:IRquest<'a>{
    fn pre_handler(&self ,request:R){}
    fn handler(&self,request:R){}
    fn post_hander(&self,request:R){}
}
```

## Router

```rust
impl <'a>IRouter<'a,Request<'a>,ConnectionSync> for PingRouter{
    fn pre_handler(&self ,request:Request<'a>){}
    fn handler(&self,request:Request<'a>){}
    fn post_hander(&self,request:Request<'a>){}
}
```

使用关联类型后
```rust
impl <'a>IRouter<'a,Request<'a>> for PingRouter{
    fn pre_handler(&self ,request:Request<'a>){}
    fn handler(&self,request:Request<'a>){}
    fn post_hander(&self,request:Request<'a>){}
}
```