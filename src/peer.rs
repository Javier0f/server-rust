use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use webrtc::api::{APIBuilder, interceptor_registry::register_default_interceptors};
use webrtc::api::media_engine::MediaEngine;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::track::track_local::TrackLocal;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpParameters};
use webrtc::media::io::h264_reader::H264Reader;
use webrtc::media::Sample;

use axum::extract::ws::WebSocket;
use axum::extract::ws::Message;

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

async fn video_track(pc: &RTCPeerConnection){
    let video_track = Arc::new(TrackLocalStaticSample::new(
    RTCRtpCodecCapability {
        mime_type: "video/vp8".to_string(),
        clock_rate: 90000,
        channels: 0,
        sdp_fmtp_line: "".to_string(),
        rtcp_feedback: vec![],
        },
    "video".to_string(),
    "webrtc-rs".to_string(),
    ));

    let _track_sender = pc
    .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
    .await.unwrap();

    let track_clone = Arc::clone(&video_track);

    tokio::spawn(async move {
        println!("enviando prueba de video");

        loop {
            let dummy_frame_data = Bytes::from(vec![0x00,0x01,0x02,0x03,0x04,0x05,]);
    
            let sample = Sample{
                data: dummy_frame_data,
                duration: Duration::from_millis(33),
                ..Default::default()
            };

            if let Err(e) = track_clone.write_sample(&sample).await{
                eprintln!("erro al enviar el sample: {e}");
                break;
            }
            tokio::time::sleep(Duration::from_millis(33)).await;
        }


    });
}

pub async fn connect_peers(pc: RTCPeerConnection,mut ws: WebSocket){

    pc.on_peer_connection_state_change(Box::new(|d|{
        println!("peer connection: {}",d.to_string());


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
                
                tokio::time::sleep(Duration::from_secs(1)).await;
                video_track(&pc).await;
                break;
            }
            _=>()
        }

        return;
    }
}