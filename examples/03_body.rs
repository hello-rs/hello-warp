use std::env;

use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Serialize, Deserialize)]
struct Student {
    id: u32,
    name: String,
}

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // 获取 post 请求的 body 数据
    // POST /students/:id {"name":"sun","id":0}
    let routes = warp::post()
        .and(warp::path("students"))
        .and(warp::path::param::<u32>())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|id, mut student: Student| {
            student.id = id;
            warp::reply::json(&student)
        });
    warp::serve(routes).bind(([127, 0, 0, 1], 3000)).await;
}
// test:
// POST http://127.0.0.1:3000/students/1
// body 选择json = {"name":"sun","id":0}
