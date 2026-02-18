use std::sync::Arc;
use std::time::Duration;

use webrtc::api::{APIBuilder, interceptor_registry::register_default_interceptors};
use webrtc::api::media_engine::MediaEngine;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;

use axum::extract::ws::WebSocket;
use axum::extract::ws::Message;

use tokio::sync::mpsc;

use serde_json;

pub async fn crear_api() -> webrtc::api::API{
    let mut m = MediaEngine::default();

    m.register_default_codecs().unwrap();

    let mut registry = Registry::new();

    registry = register_default_interceptors(registry, &mut m).unwrap();

    APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build()
}

pub async fn create_peer_connection(api: &webrtc::api::API) -> RTCPeerConnection {
    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer{
                urls: vec![],
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    api.new_peer_connection(config).await.unwrap()
}

pub async fn connect_peers(pc: RTCPeerConnection,mut ws: WebSocket){

    pc.on_peer_connection_state_change(Box::new(|d|{
        println!("{}",d.to_string());

        Box::pin(async{})
    }));
    
    while let Some(Ok(msg)) = ws.recv().await{
        match msg{
            Message::Text(text) => {
                let offer : RTCSessionDescription = serde_json::from_str(text.as_str()).unwrap();
                pc.set_remote_description(offer).await.unwrap();

                let answer = pc.create_answer(None).await.unwrap();
                pc.set_local_description(answer).await.unwrap();

                tokio::time::sleep(Duration::from_secs(1)).await;
                let answer = pc.local_description().await.unwrap();

                let answer_sdp = serde_json::to_string(&answer).unwrap();
                ws.send(Message::text(answer_sdp)).await.unwrap();
            }
            _=>()
        }
    }
}