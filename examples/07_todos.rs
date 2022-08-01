use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // 初始化数据库
    let db = models::blank_db();
    // 初始化api
    let apis = filters::todos(db);
    // 设置日志为 target = todos
    let routes = apis.with(warp::log("todos"));
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
// test:
// GET http://127.0.0.1:3000/todos
// POST http://127.0.0.1:3000/todos body = {"id" : 1,"text" : "text","completed" : false}
// PUT http://127.0.0.1:3000/todos/1 body = {"id" : 1,"text" : "123","completed" : true}
// DELETE http://127.0.0.1:3000/todos/1

mod filters {
    use std::convert::Infallible;

    use crate::{handles, models::TodoListOptions};

    use super::models::{Todo, DB};
    use warp::{Filter, Rejection, Reply};

    pub fn todos(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        todos_list(db.clone())
            .or(todos_create(db.clone()))
            .or(todos_update(db.clone()))
            .or(todos_delete(db.clone()))
    }

    pub fn todos_list(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("todos")
            .and(warp::get())
            .and(warp::query::<TodoListOptions>())
            .and(join_db(db))
            .and_then(handles::todos_list)
    }

    pub fn todos_create(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("todos")
            .and(warp::post())
            .and(join_body())
            .and(join_db(db))
            .and_then(handles::todos_create)
    }

    pub fn todos_update(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("todos" / u64)
            .and(warp::put())
            .and(join_body())
            .and(join_db(db))
            .and_then(handles::todos_update)
    }

    pub fn todos_delete(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let admin_only = warp::header::exact("authorization", "Bearer admin");

        warp::path!("todos" / u64)
            .and(admin_only)
            .and(warp::delete())
            .and(join_db(db))
            .and_then(handles::todos_delete)
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
    use log::{debug, warn};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    pub async fn todos_list(opts: TodoListOptions, db: DB) -> Result<impl warp::Reply, Infallible> {
        let db = db.lock().await;
        let res: Vec<Todo> = db
            .clone()
            .into_iter()
            .skip(opts.offset.unwrap_or(0))
            .take(opts.limit.unwrap_or(std::usize::MAX))
            .collect();
        Ok(warp::reply::json(&res))
    }

    pub async fn todos_create(req: Todo, db: DB) -> Result<impl warp::Reply, Infallible> {
        debug!("create todo : {:?}", req);
        let mut db = db.lock().await;
        let mut code = StatusCode::BAD_REQUEST;
        for todo in db.iter() {
            if todo.id == req.id {
                warn!("todo is exist : {}", req.id);
                return Ok(code);
            }
        }
        db.push(req);
        code = StatusCode::CREATED;
        Ok(code)
    }

    pub async fn todos_update(
        id: u64,
        update: Todo,
        db: DB,
    ) -> Result<impl warp::Reply, Infallible> {
        debug!("update todo : {:?}", id);
        let mut db = db.lock().await;
        for todo in db.iter_mut() {
            if todo.id == id {
                *todo = update;
                return Ok(StatusCode::OK);
            }
        }
        warn!("找不到此todo: {}", id);
        Ok(StatusCode::BAD_REQUEST)
    }

    pub async fn todos_delete(id: u64, db: DB) -> Result<impl warp::Reply, Infallible> {
        debug!("delete todo : {:?}", id);
        let mut db = db.lock().await;
        let len = db.len();
        db.retain(|todo| todo.id != id);
        if db.len() != len {
            Ok(StatusCode::NO_CONTENT)
        } else {
            warn!("找不到此todo: {}", id);
            Ok(StatusCode::NOT_FOUND)
        }
    }
}

mod models {
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;

    // 声明一个简单的内存数据库类型
    pub type DB = Arc<Mutex<Vec<Todo>>>;
    // 初始化 db
    pub fn blank_db() -> DB {
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
