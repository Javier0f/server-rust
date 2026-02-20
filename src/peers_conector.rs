use std::sync::Arc;
use std::time::Duration;
use axum::extract::ws::{WebSocket, Message};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::*;
use serde_json;
use tokio;

pub async fn create_api() -> api::API{
    let mut media_engine = api::media_engine::MediaEngine::default();
    media_engine.register_default_codecs().unwrap();

    let mut registry = interceptor::registry::Registry::new();
    registry = api::interceptor_registry::register_default_interceptors(registry, &mut media_engine).unwrap();

    api::APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build()
}

pub async fn rtc_peer_connection(api: &api::API)-> peer_connection::RTCPeerConnection{
    let config = peer_connection::configuration::RTCConfiguration{
        ice_servers: vec![
            ice_transport::ice_server::RTCIceServer{
                ..Default::default()
            }
        ],
        ..Default::default()
    };
    api.new_peer_connection(config).await.unwrap()
}

pub async fn peer_states(pc: &peer_connection::RTCPeerConnection){
    pc.on_peer_connection_state_change(Box::new(|state|{
        println!("peer: {}", state.to_string());
        Box::pin(async{})
    }));
}

async fn add_video_track(pc: &peer_connection::RTCPeerConnection){
    let video_track = Arc::new(
        track::track_local::track_local_static_sample::TrackLocalStaticSample::new(
            rtp_transceiver::rtp_codec::RTCRtpCodecCapability{
                mime_type: "video/vp8".to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "".to_string(),
                rtcp_feedback: vec![],
            },
            "video".to_string(),
            "video_rust".to_string()
        )
    );
    let _ =pc.add_track(Arc::clone(&video_track) as Arc<dyn track::track_local::TrackLocal +Send +Sync>)
    .await.unwrap();
}

async fn get_offer(pc: &peer_connection::RTCPeerConnection)-> String{
    let offer = pc.create_offer(None).await.unwrap();
    pc.set_local_description(offer.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    // println!("{}", pc.local_description().await.unwrap().to_string());
    let offer = serde_json::to_string(
        &offer
    ).unwrap();
    println!("enviando offer");
    offer
}

async fn save_answer(pc: &peer_connection::RTCPeerConnection, answer: &str){
    let answer:RTCSessionDescription = serde_json::from_str(answer).unwrap();
    pc.set_remote_description(answer).await.unwrap();
    println!("guardando respuesta");
}

pub async fn connect_peers(pc: peer_connection::RTCPeerConnection, mut ws:WebSocket){
    peer_states(&pc).await;
    add_video_track(&pc).await;

    ws.send(Message::text(get_offer(&pc).await)).await.unwrap();

    while let Some(Ok(msg)) = ws.recv().await{
        if let Message::Text(data) = msg{
            save_answer(&pc, data.as_str()).await;   
        }
    }
}