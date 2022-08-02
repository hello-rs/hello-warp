use std::env;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "tracing=info,warp=info");
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let hello = warp::path("hello").and(warp::get()).map(|| {
        info!("Hello, world!");
        "Hello, World!"
    });
    // 设置此路由的`tracing` span 为 `hello`
    // .with(warp::trace::named("hello"));

    let goodbye = warp::path("goodbye").and(warp::get()).map(|| {
        tracing::info!("saying goodbye...");
        "goodbye"
    });
    // 为路由提供自己的 `tracing` spans
    // .with(warp::trace(
    //     |info| info_span!("goodbye", req.path = ?info.path()),
    // ));
    let routes = hello
        .or(goodbye)
        // 所有的请求 `tracing` span
        .with(warp::trace::request());
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// test:
// GET http://127.0.0.1:3000/hello
// GET http://127.0.0.1:3000/goodbye
