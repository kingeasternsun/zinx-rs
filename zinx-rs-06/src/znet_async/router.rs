use crate::ziface::IRouter;
use crate::ziface::IRquest;
use crate::znet_async::request::Request;

pub struct PingRouter;

impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("pre {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        request.clone()
    }
    fn post_hander(&self, request: &Request) {
        println!("post {}", request.get_data());
    }
}

pub type RouterSync = Box<dyn IRouter<R = Request> + Send + Sync>;

pub struct OneRouter;

impl IRouter for OneRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("ONE_PRE {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        let mut res = request.clone();
        res.data.reverse();
        res
    }
    fn post_hander(&self, request: &Request) {
        println!("ONE_POST {}", request.get_data());
    }
}

pub struct TwoRouter;

impl IRouter for TwoRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("TWO_PRE {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        let mut res = request.clone();
        res.data.reverse();
        res
    }
    fn post_hander(&self, request: &Request) {
        println!("TWO_POST {}", request.get_data());
    }
}
