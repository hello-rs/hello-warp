use serde::{Deserialize, Serialize};
use std::error::Error;
use std::num::NonZeroU16;
use std::{convert::Infallible, env};
use warp::http::StatusCode;
use warp::{reject, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // 路径上参数的值,除以请求头中获取到的值
    let math = warp::path!("math" / u16);
    let div_with_heder = math
        .and(warp::get())
        .and(div_by())
        .map(|num: u16, denom: NonZeroU16| {
            warp::reply::json(&ResMathDivide {
                op: format!("{} / {} ", num, denom),
                output: num / denom.get(),
            })
        });

    let div_with_body =
        math.and(warp::post())
            .and(warp::body::json())
            .map(|num: u16, req: ReqMathDivide| {
                warp::reply::json(&ResMathDivide {
                    op: format!("{} / {} ", num, req.denom),
                    output: num / req.denom.get(),
                })
            });
    let routes = div_with_heder.or(div_with_body).recover(handle_rejection);
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// entity --------------------------------
// 请求与结果返回值
#[derive(Deserialize)]
struct ReqMathDivide {
    denom: NonZeroU16,
}
#[derive(Serialize)]
struct ResMathDivide {
    op: String,
    output: u16,
}
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}
#[derive(Debug)]
// 自定义除数为零的错误
struct DivideByZero;
impl reject::Reject for DivideByZero {}
// entity --------------------------------

// 从请求头中`div-by`获取分母,若为零则拒绝`DivideByZero`
fn div_by() -> impl Filter<Extract = (NonZeroU16,), Error = Rejection> + Copy {
    // and_then 使用异步检查参数
    warp::header::<u16>("div_by").and_then(|denom| async move {
        if let Some(denom) = NonZeroU16::new(denom) {
            Ok(denom)
        } else {
            Err(reject::custom(DivideByZero))
        }
    })
}

// 此函数处理`Rejection`异常, 并返回一个自定义的值.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = code.as_str();
    } else if let Some(DivideByZero) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "DIVIDE_BY_ZERO";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        message = match e.source() {
            Some(cause) => {
                println!("{}", cause);
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom"
                } else {
                    "BAD_REQUEST"
                }
            }
            None => "BAD_REQUEST",
        };
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

// test:
// GET http://127.0.0.1:3000/math/12  header-> div_by: 2
// POST http://127.0.0.1:3000/math/12  body -> { "denom": 2 }
