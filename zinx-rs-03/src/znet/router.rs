
use crate::ziface::IRouter;
use crate::znet::request::Request;
use crate::znet::connection::ConnectionSync;

pub struct PingRouter;

impl <'a>IRouter<'a,Request<'a>,ConnectionSync> for PingRouter{
    fn pre_handler(&self ,request:Request<'a>){}
    fn handler(&self,request:Request<'a>){}
    fn post_hander(&self,request:Request<'a>){}
}