#![allow(non_snake_case)]
use crate::ziface::IRouter;
/// 消息管理抽象层
pub trait IMsgHandle {
    /// 消息管理肯定关联一个特定的类型的request
    type Req;
    /// 马上阻塞方式处理消息
    fn DoMsgHandler(req: &Self::Req);
    /// 为消息添加具体的处理逻辑
    fn AddRouter(msgId: u32, router: Box<dyn IRouter<R = Self::Req> + Send + Sync>);
}
