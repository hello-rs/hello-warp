#[tokio::main]
async fn main() {
    use std::env;
    use warp::Filter;

    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let routes = warp::any().map(|| "hello warp!");
    // 启动服务
    warp::serve(routes)
        .tls()
        .cert_path("examples/tls/cert.pem")
        .key_path("examples/tls/key.rsa")
        .run(([127, 0, 0, 1], 3000))
        .await;
}

// test:
// https://127.0.0.1:3000
