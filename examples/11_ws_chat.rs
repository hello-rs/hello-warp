use std::{
    collections::HashMap,
    env,
    sync::{atomic::AtomicUsize, Arc},
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use log::{error, info};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    // 1.设置日志级别并初始化日志
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let users = Users::default();
    // 将用户数据封装到 warp::Filter 中
    let join_users = warp::any().map(move || users.clone());
    // warp::ws() 用于 websocket upgrade
    let ws = warp::path("chat")
        .and(warp::ws())
        .and(join_users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(|socket| user_connected(socket, users)));
    // 客户端 html
    let index_html = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    let routes = ws.or(index_html);
    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

// test:
// http://127.0.0.1:3000

async fn user_connected(socket: WebSocket, users: Users) {
    // 新的用户连接: 自增用户ID
    let user_id = NEXT_USER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    eprintln!("new chat user: {}", user_id);
    // 拆分 socket 为发送方和接受方
    let (mut user_tx, mut user_rx) = socket.split();
    // 创建一个没有缓冲区的 channel
    let (chan_tx, chan_rx) = mpsc::unbounded_channel();
    let mut chan_rx = UnboundedReceiverStream::new(chan_rx);
    // tokio 异步,将channel中收到的消息通过 ws 发送出去
    tokio::spawn(async move {
        while let Some(msg) = chan_rx.next().await {
            user_tx
                .send(msg)
                .unwrap_or_else(|err| error!("发送消息失败: {}", err))
                .await;
        }
    });
    // 保存用户信息
    users.write().await.insert(user_id, chan_tx);
    // 处理用户逻辑: 用户收到消息将广播给其他用户
    println!("??????");
    while let Some(msg) = user_rx.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                error!("接收消息失败 uid= {},err={}", user_id, err);
                break;
            }
        };
        user_message(user_id, msg, &users).await;
    }
    // 连接断开
    user_disconnected(user_id, &users).await;
}

async fn user_message(user_id: usize, msg: Message, users: &Users) {
    // 跳过非文本的消息
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };
    // 格式化消息
    let msg = format!("用户:{}, 发送: {}", user_id, msg);
    for (&receice_user_id, user_tx) in users.read().await.iter() {
        if receice_user_id == user_id {
            continue;
        }
        let res = user_tx.send(Message::text(msg.clone()));
        if let Err(err) = res {
            error!("发送消息失败 uid={},err={}", receice_user_id, err);
        }
    }
}

async fn user_disconnected(user_id: usize, users: &Users) {
    info!("disconnected user_id : {}", user_id);
    // 删除用户
    users.write().await.remove(&user_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
