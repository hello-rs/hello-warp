use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // 路由文件 README.md
    let readme = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./README.md"));
    // 路由目录 examples
    let examples = warp::path("ex").and(warp::fs::dir("./examples/"));

    let routes = readme.or(examples);
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// test:
// GET http://127.0.0.1:3000
// GET http://127.0.0.1:3000/ex/01_hello.rs
