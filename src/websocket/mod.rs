use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::http::Method;
use warp::{ws::Message, Filter};
use std::error::Error;

mod ws;
mod handler;

pub use ws::WebSocketHandler;
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
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub async fn serve(ws_handler: Arc<WebSocketHandler>) -> std::result::Result<(), Box<dyn Error>> {  
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
        .and(with_clients(ws_handler.clients.clone()))
        .and_then(handler::register_handler)
        // DELETE /register/{client_id} : ID를 통해 클라이언트를 등록 해제하기 위한 라우트.
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(ws_handler.clients.clone()))
            .and_then(handler::unregister_handler));

    // POST /publish — 클라이언트들에 이벤트를 Broadcasts 하기위한 라우트.
    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(ws_handler.clients.clone()))
        .and_then(handler::publish_handler);

    // GET /ws — WebSocket 엔드포인트
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(ws_handler.clients.clone()))
        .and(with_ws_handler(ws_handler.clone()))
        .and_then(handler::ws_handler);

    // 라우터를 등록하고, CORS를 지원하도록 함
    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .with(cors);

    // 서버 시작
    warp::serve(routes).run(([127, 0, 0, 1], 32332)).await;
    Ok(())
}

// 라우트에 접근했을 때, Client정보를 handler에 전달하기 위한 미들웨어(필터) 용도의 함수
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_ws_handler(
    ws_handler: Arc<WebSocketHandler>,
) -> impl Filter<Extract = (Arc<WebSocketHandler>,), Error = Infallible> + Clone {
    warp::any().map(move || ws_handler.clone())
}