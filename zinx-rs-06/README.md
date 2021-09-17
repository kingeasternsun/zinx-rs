参考golang版本 [六、Zinx的多路由模式](https://www.kancloud.cn/aceld/zinx/1960229)

我们之前在已经给Zinx配置了路由模式，之前的Zinx好像只能绑定一个路由的处理业务方法。显然这是无法满足基本的服务器需求的，那么现在我们要在之前的基础上，给Zinx添加多路由的方式。

既然是多路由的模式，我们这里就需要给MsgId和对应的处理逻辑进行捆绑。所以我们需要一个Map。
其中key就是msgId， value就是对应的Router，里面应是使用者重写的Handle等方法。

我们再定义一个消息管理模块来进行维护这个Map。

特定类型的消息管理器肯定和特定类型的消息关联，所以我们定义的消息管理 trait IMsgHandle 包含一个关联类型Req用于关联 Request。

消息管理模块实现 MsgHandle 跟同步或异步无关，所以放入到util中

IRquest 新增一个方法get_msgID 获取里面message的ID
```rust
pub trait IRquest {
    // 关联 Connection
    type Conn;
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn>;
    // 获取请求消息的数据
    fn get_data(&self) -> Message;

    fn get_msgID(&self)->u32;
}
```



IRouter 中 handler 方法改为返回 Self::R

```rust
pub trait IRouter {
    // 关联一个 Request
    type R;
    fn pre_handler(&self, _request: &Self::R) {}
    fn handler(&self, request: &Self::R)->Self::R;
    fn post_hander(&self, _request: &Self::R) {}
}
```



首先`iserver`的`AddRouter()`的接口要稍微改一下，增添MsgId参数



其次，`Server` 类型中 之前有一个`Router`成员，类型 `Option<RouterSync>` ，代表唯一的处理方法，现在应该替换成` Arc<MsgHandle<Request>>`类型



始化Server自然也要改正，增加msgHandler初始化



当Server在处理conn请求业务的时候，创建conn的时候也需要把msgHandler作为参数传递给Connection对象



Connection对象中 之前有一个`Router`成员，类型 `RouterSync` ，代表唯一的处理方法，现在也应该替换成` Arc<MsgHandle<Request>>`类型，来查找消息对应的回调路由方法





最后，在conn已经拆包之后，需要调用路由业务的时候，我们只需要让conn调用MsgHandler中的`DoMsgHander()`方法就好了