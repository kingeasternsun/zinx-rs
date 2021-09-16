// use crate::ziface::IRquest;

/// 路由接口，这里路由是使用链接自定义的处理业务方法
/// 路由里面的 R:IRquest<'a,T> ，包含当前连接信息和连接传入的请求数据
/// 大部分场景下，相同的tcp 连接中的请求的数据格式是一样的，或者我们在解析的时候可以转为一样的类型，例如使用 enum，所以这里使用静态分发
/// 这里使用了默认实现，就不需要像golag版本那样创建 BaesRouter了
pub trait IRouter {
    // 关联一个 Request
    type R;
    fn pre_handler(&self, _request: Self::R) {}
    fn handler(&self, _request: Self::R) {}
    fn post_hander(&self, _request: Self::R) {}
}
