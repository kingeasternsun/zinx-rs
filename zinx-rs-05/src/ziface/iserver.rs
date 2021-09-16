#![allow(non_snake_case)]
use crate::IRouter;
use std::sync::Arc;
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
