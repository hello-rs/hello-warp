use std::env;
use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // 假定没有 DNS,所有 host 请求头是一个地址
    let host = warp::header::<SocketAddr>("host");
    // 完全匹配 `accept: */*`
    let accept_stars = warp::header::exact("accept", "*/*");
    let routes = host
        .and(accept_stars)
        .map(|addr| format!("accepting stars on {}", addr));

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test:
// GET http://127.0.0.1:3000
