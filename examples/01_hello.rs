use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /
    let routes = warp::any().map(|| "hello warp!");
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test: 
// http://127.0.0.1:3000
