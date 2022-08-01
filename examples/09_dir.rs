use std::env;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let routes = warp::fs::dir("examples/dir");
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// test:
// GET http://127.0.0.1:3000
