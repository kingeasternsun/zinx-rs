参考golang版本的 [三、Zinx框架基础路由模块](https://www.kancloud.cn/aceld/zinx/1960215)


由于 future 的一些特性限制，使用tokio无法完全参考golang中的设计架构进行编写，所以从v0.3开始，async版本的开发会以功能优先，用rust async的思维来设计和开发。

# 同步版本server

## znet
使用golang定义接口，返回slice非常简单，golang会基于逃逸分析进行内存的分配和管理，但是在rust中就不能直接这样搞了，需要使用生命周期参数 `a 如下

## IRquest

注意声明trait的时候，生命周期参数‘a，要在泛型参数T前面。

在设计IRquest trait的时候，最开始把 Iconnection 单独作为一个泛型参数如下：

```rust
pub trait IRquest<'a,T> {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<T> where T:Iconnection;
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8];
}
```

后来在定义其他trait的时候越来越难写，泛型参数也越来越多，于是重新换了思路思考了一下，对于一类Conneciton来说，相应的IRequest应该就是固定的，所以我们可以把Connection作为IRquest的关联类型，这样不仅仅简化了IRquest的trait定义，而且在功能层面上也更加的清晰。

给IRquest添加一个关联类型，用于关联Connection,同时把get_data返回类型改为String，这样就把生命周期参数也去掉了。
```rust
pub trait IRquest {
    // 关联 Connection
    type Conn;
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn>;
    // 获取请求消息的数据
    fn get_data(&self) -> String;
}
```

## Request

定义 Request的时候，需要标注生命周期参数，因为 data是一个引用，不然编译器不知道所引用对象的生命周期的长度
```rust
pub struct Request<'a> {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: &'a [u8],            //客户端请求的数据
}
```

实现trait的时候，要首先在impl后面声明‘a
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

IRquest使用关联类型同时把 &'a [u8] 改为 String后如下所示，Request的实现就简单清晰了许多：
```rust
#[derive(Clone)]
pub struct Request {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: String,              //客户端请求的数据
}

impl Request {
    pub fn new(con: Arc<ConnectionSync>, data: String) -> Self {
        Request {
            conn: Arc::clone(&con),
            data: data,
        }
    }
}
```
实现IRquest trait 的是，将 ConnectionSync 作为trait的关联类型进行实现
```rust

impl IRquest for Request {
    type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
                                // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }
    // 获取请求消息的数据
    fn get_data(&self) -> String {
        self.data.clone()
    }
}
```
使用关联类型，相比trait限制，代码可读性上更强，而且从业务上看，对于一个Request来说，对应的IRquest就应该只有一种实现。


## IRouter

IRouter 这里使用了默认实现，就不需要像golag版本那样创建 BaseRouter了
```rust
pub trait IRouter<'a,R,T>  where R:IRquest<'a,T>{
    fn pre_handler(&self ,request:R){}
    fn handler(&self,request:R){}
    fn post_hander(&self,request:R){}
}
```

给IRouter添加一个关联类型，用于关联Request，这里生命周期参数也可以去掉了，代码简洁了许多。
```rust
pub trait IRouter {
    // 关联一个 Request
    type R;
    fn pre_handler(&self, request: Self::R) {}
    fn handler(&self, request: Self::R) {}
    fn post_hander(&self, request: Self::R) {}
}
```

## Router

定义PingRouter
```rust
pub struct PingRouter;
```

给PingRouter实现 IRouter trait
```rust
impl <'a>IRouter<'a,Request<'a>,ConnectionSync> for PingRouter{
    fn pre_handler(&self ,request:Request<'a>){}
    fn handler(&self,request:Request<'a>){}
    fn post_hander(&self,request:Request<'a>){}
}
```

IRouter使用关联类型后，将Request作为 IRouter trait的关联类型进行实现
```rust
impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: Request) {
        println!("pre {}", request.get_data());
    }
    fn handler(&self, request: Request) {}
    fn post_hander(&self, request: Request) {
        println!("post {}", request.get_data());
    }
}
```


## Iserver 

```rust
pub trait Iserver<'a, R> {
    //  启动服务器的方法
    fn Start(&mut self) -> std::io::Result<()>;
    // 停止服务器的方法
    fn Stop(&mut self);
    // 开启业务服务方法
    fn Serve(&mut self);
    //路由功能：给当前服务注册一个路由业务方法，供客户端链接处理使用
    fn AddRouter(&mut self, router: Arc<Box<dyn IRouter<'a, R>>>)
    where
        R: IRquest<'a>;
}
```

给Iserver设置一个关联类型，用于关联 Request，生命周期参数也可以去掉了。
```rust
pub trait Iserver {
    //关联一个 Request
    type R;
    //  启动服务器的方法
    fn Start(&self) -> std::io::Result<()>;
    // 停止服务器的方法
    fn Stop(&mut self);
    // 开启业务服务方法
    fn Serve(&mut self);
    //路由功能：给当前服务注册一个路由业务方法，供客户端链接处理使用
    fn AddRouter(&mut self, router: Arc<Box<dyn IRouter<R = Self::R> + Send + Sync>>);
}
```
这里之所以要给Iserver关联Request 在于AddRouter 方法中参数router中 dyn IRouter 需要制定IRouter的关联类型，所以这里把 Iserver 的关联类型 R 和 IRouter 中的关联类型 R 连接了起来。

## server

```rust
pub struct Server<'a, R> {
    // 服务器名称
    Name: String,
    // tcp4 or other
    IPVersion: String,
    // 服务绑定的IP地址
    IP: String,
    // 服务绑定的端口
    Port: u32,

    Router: Option<Box<dyn IRouter<'a, R>>>,
}
```

使用关联类型后
```rust
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
```
