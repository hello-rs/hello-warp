use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // GET /
    let hello_warp = warp::path::end().map(|| "hello warp!");
    // 通过 `path!` 宏实现多个路径段
    // GET /hello/from/warp
    let hello_from_warp = warp::path!("hello" / "from" / "warp").map(|| "hello from warp!");
    // 任何实现 FromStr 的类型都可以作为参数(如u32,u16...),顺序可以随意
    // GET /sum/:u32/:u32 获取 url 路径上的参数
    let sum = warp::path!("sum" / u32 / u32).map(|a, b| format!("{} + {} = {}", a, b, a + b));
    // 分组,使用相同的父路径
    // GET /math/sum/:u32/:u32
    let math = warp::path("math");
    // `and`组合两个过滤器 `or` 两者中匹配其中一个
    let _sum = math.and(sum);
    // GET /bye/:string  param(): 从路径中提取参数
    let bye = warp::path("bye")
        .and(warp::path::param())
        .map(|name: String| format!("Good Bye {}!", name));
    // GET /math/sum/:u32/:u32
    let math = warp::path("math").and(sum);
    // GET /math warp::path::end() 表示路由路径匹配结束
    let help = warp::path("math")
        .and(warp::path::end())
        .map(|| "math 中有一个API. 请尝试调用 /math/sum/:u32/:u32");
    let math = help.or(math);
    // 告诉人们 sum` and `times` 移动到了 `math` 下,output 为上方sum map中计算的结果
    // GET /sum/:u32/:u32
    let sum = sum.map(|output| format!("(路由已经移动到 /math/sum/:u32/:u32) {}", output));
    // `or` 可以将路由组合在一起,最后生成 `routes` warp::get() 并且强制使用 GET
    let routes = warp::get().and(hello_warp.or(hello_from_warp).or(bye).or(math).or(sum));

    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test:
// GET http://127.0.0.1:3000/
// GET http://12`7.0.0.1:3000/hello/from/warp
// GET http://127.0.0.1:3000/bye/:string
// GET http://127.0.0.1:3000/math/sum/2/3
// GET http://127.0.0.1:3000/sum/2/3
// GET http://127.0.0.1:3000/math
