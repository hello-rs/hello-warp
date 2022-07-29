use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    // 1.初始化日志
    env_logger::init();
    // GET /
    let hello_warp = warp::path::end().map(|| "hello warp!");
    // GET /hi
    let hi = warp::path("hi").map(|| "hello world!");
    // GET /hello/from/warp 通过 `path!` 宏实现多个路径段
    let hello_from_warp = warp::path!("hello" / "from" / "warp").map(|| "hello from warp!");
    // GET /sum/:u32/:u32 获取路径上的参数
    let sum = warp::path!("sum" / u32 / u32).map(|a, b| format!("{} + {} = {}", a, b, a + b));
    // GET /:u16/times/:u16
    // 任何实现FromStr的类型都可以使用(如u16,u32...),顺序可以随意
    let times = warp::path!(u16 / "times" / u16).map(|a, b| format!("{} * {} = {}", a, b, a * b));
    // 分组,使用相同的父路径
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16
    let math = warp::path("math");
    let _sum = math.and(sum);
    let _times = math.and(times);
    // and 可以组合过滤器,内部实现了 `path!`
    // GET /bye/:string
    let bye = warp::path("bye")
        .and(warp::path::param())
        .map(|name: String| format!("Good Bye {}!", name));
    // 除了 and, 还有 or 函数可以匹配其中一个
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16
    let math = warp::path("math").and(sum.or(times));
    // GET /math warp::path::end() 表示路由路径匹配结束
    let help = warp::path("math")
        .and(warp::path::end())
        .map(|| "This is the Math API. Try calling /math/sum/:u32/:u32 or /math/:u16/times/:u16");
    let math = help.or(math);
    // 告诉人们 sum` and `times` 移动到了 `math` 下
    // GET /sum/:u32/:u32
    // GET /:u16/times/:u16
    let sum = sum.map(|output| format!("(路由已经移动到 /math/sum/:u32/:u32) {}", output));
    let times = times.map(|output| format!("(路由已经移动到 /math/times/:u16/:u16) {}", output));
    // `or` 可以将路由组合在一起,最后生成 `routes` warp::get() 并且强制使用 GET
    let routes = warp::get().and(
        hello_warp
            .or(hi)
            .or(hello_from_warp)
            .or(bye)
            .or(math)
            .or(sum)
            .or(times),
    );

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test:
// GET http://127.0.0.1:3000/
// GET http://127.0.0.1:3000/hi
// GET http://127.0.0.1:3000/hello/from/warp
// GET http://127.0.0.1:3000/bye/:string
// GET http://127.0.0.1:3000/math/sum/2/3
// GET http://127.0.0.1:3000/math/2/times/2
// GET http://127.0.0.1:3000/sum/2/3
// GET http://127.0.0.1:3000/2/times/2
// GET http://127.0.0.1:3000/math
