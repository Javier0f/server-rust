use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::Extension;
use axum::response::Response;
use axum::{Router, routing::get};
use tower_http::services::ServeDir;
use std::sync::{Arc};

use web::{State, aceptar, denegar};

async fn connect(ws: WebSocketUpgrade, Extension(estado): Extension<Arc<State>>)->Response{
    let estado_actual = estado.state.lock().unwrap();

    if *estado_actual{
        ws.on_upgrade(denegar)
    }else {
        let estado_clone = estado.clone();
        ws.on_upgrade(move |socket: WebSocket|aceptar(socket, Extension(estado_clone)))
    }
}

#[tokio::main]
async fn main() {

    let estado = Arc::new(State::new());

    let public = ServeDir::new("./data");

    let app = Router::new()
        .route("/connect", get(connect))
        .layer(Extension(estado))
        .fallback_service(public);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}