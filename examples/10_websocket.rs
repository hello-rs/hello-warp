use std::env;

use futures_util::{FutureExt, StreamExt};
use log::{error, info};
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // warp::ws() 用于 websocket upgrade
    let ws = warp::path("echo").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            // forward 将 rx 收到的消息转发给了 tx 返回了所收到的消息
            rx.forward(tx).map(|result| {
                if let Err(err) = result {
                    error!("ws error: {}", err)
                }
            })
        })
    });
    let routes = ws;
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// test: 可用 vscode 插件 websocket client
// [插件地址](https://marketplace.visualstudio.com/items?itemName=mohamed-nouri.websocket-client)
// ws://localhost:3000/echo
