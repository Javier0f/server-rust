use std::sync::{Mutex, Arc};
use axum::Extension;
use axum::extract::ws::Message;
use axum::extract::ws::{WebSocket};

// mod peer;
// use peer::{crear_api, create_peer_connection, connect_peers};

mod peers_conector;
use peers_conector::{create_api, rtc_peer_connection, connect_peers};

//Acepta la conexión por web socket y actualiza el estado del servidor
//Cuando ese cliente salga, lo devuelve FALSE para aceptar el proximo cliente
pub async fn aceptar(socket: WebSocket, Extension(estado): Extension<Arc<State>>){
    estado.in_use();

    let api = create_api().await;
    let pc = rtc_peer_connection(&api).await;

    println!("cliente conectado por web socket");

    connect_peers(pc, socket).await;

    estado.free();
}

//Acepta la conexión, envia un mensaje al cliente diciendo que
//el servidor ya está en uso por otro cliente y luego cierra la conexión
pub async fn denegar(mut socket: WebSocket){
    socket.send(Message::text("Servidor en uso")).await.unwrap();
    return ;
}

pub struct State{
    pub state: Mutex<bool>
}
impl State {
    pub fn new()->State{
        State { state: Mutex::new(false) }
    }

    pub fn in_use(& self){
        *self.state.lock().unwrap() = true;
    }

    pub fn free(& self){
        *self.state.lock().unwrap() = false;
    }
}