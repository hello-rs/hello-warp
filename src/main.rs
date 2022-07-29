use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    // 1.初始化日志
    env_logger::init();
    // GET /
    let hello_warp = warp::any().map(|| "hello warp!");
    // GET /hi
    // GET /hello/from/warp 通过 `path!` 宏实现多个路径段

    // GET /sum/:u32/:u32 获取路径上的参数

    // GET /:u16/times/:u16
    // 任何实现FromStr的类型都可以使用(如u16,u32...),顺序可以随意

    // 分组,使用相同的父路径
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16

    // and 可以组合过滤器,内部实现了 `path!`
    // GET /bye/:string

    // 除了 and 还有 or 可以匹配其中一个
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16

    // warp::path::end() 表示路由路径匹配结束

    // 告诉人们 sum` and `times` 移动到了 `math` 下

    // `or` 可以将路由组合在一起,最后生成 `routes`
    // warp::get() 并且强制使用 GET

    // warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test:
// GET http://127.0.0.1:3000/
// GET http://127.0.0.1:3000/hi
// GET http://127.0.0.1:3000/hello/from/warp
// GET http://127.0.0.1:3000/bye/:string
// GET http://127.0.0.1:3000/math/sum/:u32/:u32
// GET http://127.0.0.1:3000/math/:u16/times/:u16
