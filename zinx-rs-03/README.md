参考golang版本的 [三、Zinx框架基础路由模块](https://www.kancloud.cn/aceld/zinx/1960215)


由于 future 的一些特性限制，使用tokio无法完全参考golang中的设计架构进行编写，所以从v0.3开始，async版本的开发会以功能优先，用rust async的思维来设计和开发。

# 同步版本 znet

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

后来在定义其他trait的时候越来越难写，泛型参数也越来越多，于是重新换了思路思考了一下，对于一类 `Request` 来说，相应的 `Connection` 应该就是固定的，所以我们可以把 `Connection` 作为 `IRquest` 的关联类型，这样不仅仅简化了 `IRquest` 的trait定义，而且在功能层面上也更加的清晰。

给 `IRquest` 添加一个关联类型，用于关联 `Connection` , 同时把生命周期参数 `‘a ` 的生命放在 `get_data` 中，如下:
```rust
pub trait IRquest {
    // 关联 Connection
    type Conn;
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn>;
    // 获取请求消息的数据
    fn get_data<'a>(&'a self) -> &'a [u8];
}
```

## Request

定义 Request的时候，data 定义为 `Vec<u8>` ，这是因为后续Request要独立进行处理，不能和之前的read缓存buf有关联。
```rust
pub struct Request {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: Vec<u8>,            //客户端请求的数据
}
```

实现trait的时候，我们最开始没有使用管理类型的时候写法如下：
```rust
impl IRquest<ConnectionSync> for Request {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }
    // 获取请求消息的数据
    fn get_data(&self) ->&[u8] {
        &self.data[..]
    }
}
```

使用关联类型之后，Request的实现就简单清晰了许多，实现 `IRquest` trait 时候，将 `ConnectionSync` 作为trait的关联类型进行实现
```rust

impl IRquest for Request {
    type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
                                // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }
    // 获取请求消息的数据
    fn get_data(&self) ->&[u8] {
        &self.data[..]
    }
}
```
使用关联类型，相比trait限制，代码可读性上更强，而且从业务上看，对于一个Request来说，对应的IRquest就应该只有一种实现。如果不使用关联类型的话，那么 `impl IRquest<ConnectionSync> for Request` 这种实现方式意味着对于同一个` Request` 类型可以实现多种 `IRquest<T>` trait，显然在实际中是不合适的。

关联类型大家也可以去了解下rust标准库中的Add trait，或者在书籍《Rust编程之道》中也有讲述。


## IRouter

IRouter 这里使用了默认实现，就不需要像golag版本那样创建 BaseRouter了。

另外上文中提到不使用关联类型之前代码最后越写越复杂，类型参数越来越多，这里我们就可以对比下：

### 不使用关联类型的定义方式

```rust
pub trait IRouter<R,T>  where R:IRquest<T>{
    fn pre_handler(&self ,request:R){}
    fn handler(&self,request:R){}
    fn post_hander(&self,request:R){}
}
```

### 使用关联类型之后的定义方式

这里只需要给IRouter添加一个关联类型，用于关联Request。

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

### 不使用关联类型的实现方式

```rust
impl IRouter<Request<,ConnectionSync> for PingRouter{
    fn pre_handler(&self ,request:Request){}
    fn handler(&self,request:Request){}
    fn post_hander(&self,request:Request){}
}
```

### 使用关联类型之后的实现方式

IRouter使用关联类型后，将Request作为 IRouter trait的关联类型进行实现

```rust
impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: Request) {
        println!("pre {:?}", request.get_data());
    }
    fn handler(&self, request: Request) {}
    fn post_hander(&self, request: Request) {
        println!("post {:?}", request.get_data());
    }
}
```

## Iserver 



### 不使用关联类型的定义方式

```rust
pub trait Iserver< R> {
    //  启动服务器的方法
    fn Start(&mut self) -> std::io::Result<()>;
    // 停止服务器的方法
    fn Stop(&mut self);
    // 开启业务服务方法
    fn Serve(&mut self);
    //路由功能：给当前服务注册一个路由业务方法，供客户端链接处理使用
    fn AddRouter(&mut self, router: Arc<Box<dyn IRouter<R>>>)
    where
        R: IRquest<'a>;
}
```

### 使用关联类型之后的定义方式

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
这里之所以要给 `Iserver`关联`Request` 在于`AddRouter` 方法中的参数 `router`的类型 `dyn IRouter` 需要指定 `IRouter`的关联类型，所以这里把 Iserver 的关联类型 R 和 IRouter 中的关联类型 R 连接了起来。

## server

### 不使用关联类型的定义方式

```rust
pub struct Server<R> {
    // 服务器名称
    Name: String,
    // tcp4 or other
    IPVersion: String,
    // 服务绑定的IP地址
    IP: String,
    // 服务绑定的端口
    Port: u32,

    Router: Option<Box<dyn IRouter<R>>>,
}
```

### 使用关联类型之后的定义方式

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
