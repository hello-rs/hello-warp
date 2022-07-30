use std::str::FromStr;
use std::time::Duration;
use std::{convert::Infallible, env};
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // /:Seconds  睡眠几秒后返回数据
    let routes = warp::path::param().and_then(sleepy);
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// 创建一个新类型,用于限制睡眠最大的秒数,实现FromStr即可作为参数使用
struct Seconds(u64);
impl FromStr for Seconds {
    type Err = ();
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.parse::<u64>().map_err(|_| ()).and_then(|num| {
            if num <= 5 {
                Ok(Seconds(num))
            } else {
                Err(())
            }
        })
    }
}

async fn sleepy(Seconds(seconds): Seconds) -> Result<impl warp::Reply, Infallible> {
    tokio::time::sleep(Duration::from_secs(seconds)).await;
    Ok(format!("睡眠了 {} 秒后返回", seconds))
}

// test:
// GET http://127.0.0.1:3000/2
