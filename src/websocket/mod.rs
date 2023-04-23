use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::http::Method;
use warp::{ws::Message, Filter, Rejection};

use crate::models::{AppEvent, WsEvent};
use std::thread;

mod handler;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;
/*
 * Client를 Hash맵에 저장해 track하여 연결 유지
 * 단, 비동기 작업을 할 것이므로(클라이언트 등록, 메세지 전송 등..)
 * 스레드 간 안전하게 전달되고 문제 없이 동시에 작업이 가능하도록 해야 함.
 * 따라서 Hash맵을 Mutex 뒤에 두어 코드의 한 부분에서만, 특정 시간에 접근할 수 있도록 한다.
 * 이를 다른 스레드로 안전하게 전달하기 위해, 스레드에 안전한 방식으로 공유 소유권을 제공하는
 * Smart pointer 타입의 Arc로 래핑한다.
 */
type Clients = Arc<RwLock<HashMap<String, Client>>>; // Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    //    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[tokio::main]
pub async fn serve(sender: Sender<AppEvent>, receiver: Receiver<WsEvent>) {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    // GET /health : 서비스가 활성 상태인지 확인하기 위한 Health check 라우트.
    let health_route = warp::path!("health").and_then(handler::health_handler);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(&[Method::GET, Method::POST, Method::OPTIONS])
        .max_age(30);

    // POST /reghster : ws 서비스에 client를 등록하기 위한 라우트.
    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        // DELETE /register/{client_id} : ID를 통해 클라이언트를 등록 해제하기 위한 라우트.
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    // POST /publish — 클라이언트들에 이벤트를 Broadcasts 하기위한 라우트.
    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

    // GET /ws — WebSocket 엔드포인트
    let ws_route = warp::path("ws")
        .and(warp::ws()) // HTTP연결을 WebSocket연결로 Upgrade하기 위한 필터
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and(with_sender(sender.clone()))
        .and_then(handler::ws_handler);

    // 라우터를 등록하고, CORS를 지원하도록 함
    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .with(cors);

    // https://docs.rs/crate/tokio-tungstenite/0.10.1/source/examples/server.rs
    // https://users.rust-lang.org/t/broadcast-server-what-is-the-best-way/88277/3
    // https://stackoverflow.com/questions/60025114/how-to-run-futures-containing-borrowed-tcpstream-concurrently

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        loop {
            let r = receiver.clone();
            match r.recv() {
                Ok(WsEvent::Config(x, id)) => {
                    log::debug!("Received: {:?}", x);
                    let _ = runtime
                        .handle()
                        .block_on(handler::send_config(x, clients.clone(), id));
                }
                // 다른 모듈로부터 받은 전송 데이터를 뿌려준다.
                Ok(WsEvent::UpdateInfo(x, id)) => {
                    log::debug!("Received: {:?}", x);
                    let _ = runtime.handle().block_on(handler::send_update_info(
                        x,
                        clients.clone(),
                        id,
                    ));
                }
                Ok(WsEvent::Track(x)) => {
                    let _ = runtime
                        .handle()
                        .block_on(handler::broadcast_track(x, clients.clone()));
                }
                Ok(_) => {}
                Err(_) => {} //panic!("panic happened"),
            }
        }
    });

    // 서버 시작
    warp::serve(routes).run(([127, 0, 0, 1], 32332)).await;
}

// 라우트에 접근했을 때, Client정보를 handler에 전달하기 위한 미들웨어(필터) 용도의 함수
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_sender(
    sender: Sender<AppEvent>,
) -> impl Filter<Extract = (Sender<AppEvent>,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}
