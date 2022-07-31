use std::str::FromStr;
use std::time::Duration;
use std::{convert::Infallible, env};
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "todos=debug");
    env_logger::init();
    // 初始化数据库
    // 初始化api
    // 设置日志为 target = todos
    // warp::log(name)
    let routes = warp::path::param().and_then(sleepy);
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

mod filters {
    use std::convert::Infallible;

    use crate::models::TodoListOptions;

    use super::models::{Todo, DB};
    use warp::{Filter, Rejection, Reply};

    fn todos_list(db: DB) -> impl Filter<Extract = Reply, Error = Rejection> + Clone {
        warp::path("todos")
            .and(warp::get())
            .and(warp::query::<TodoListOptions>())
            .and(join_db(db).and_then(other))
    }

    // 将 db 加入参数中 Infallible 不会发生错误
    fn join_db(db: DB) -> impl Filter<Extract = (DB,), Error = Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    // body 加入参数
    fn join_body() -> impl Filter<Extract = (Todo,), Error = Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}
mod handles {
    use super::models::{Todo, TodoListOptions, DB};
    use log::debug;
    use std::convert::Infallible;
    use warp::http::StatusCode;

    async fn todos_list(opts: TodoListOptions, db: DB) -> Result<impl warp::Reply, Infallible> {
        let db = db.lock().await;
        let res: Vec<Todo> = db
            .clone()
            .into_iter()
            .skip(opts.offset.unwrap_or(0))
            .take(opts.limit.unwrap_or(std::usize::MAX))
            .collect();
        Ok(warp::reply::json(&res))
    }

    async fn todos_create(req: Todo, db: DB) -> Result<impl warp::Reply, Infallible> {
        debug!("create todo : {:?}", req);
        let mut code = StatusCode::BAD_REQUEST;
        let mut db = db.lock().await;
        for todo in db.iter() {
            if todo.id == req.id {
                debug!("todo is exist : {}", req.id);
                return Ok(code);
            }
        }
        db.push(req);
        code = StatusCode::CREATED;
        Ok(code)
    }
}

mod models {
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;

    // 声明一个简单的内存数据库类型
    pub type DB = Arc<Mutex<Vec<Todo>>>;
    // 初始化 db
    fn blank_db() -> DB {
        Arc::new(Mutex::new(Vec::new()))
    }
    // 表结构
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Todo {
        pub id: u64,
        pub text: String,
        pub completed: bool,
    }

    #[derive(Debug, Deserialize)]
    // 查询参数
    pub struct TodoListOptions {
        pub offset: Option<usize>,
        pub limit: Option<usize>,
    }
}
