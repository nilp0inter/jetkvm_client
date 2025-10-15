use anyhow::{anyhow, Result as AnyResult};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::header, protocol::Message},
};
use tracing::{debug, info, warn};
use webrtc::{
    api::{media_engine::MediaEngine, APIBuilder},
    data_channel::RTCDataChannel,
    dtls::extension::extension_use_srtp::SrtpProtectionProfile,
    ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
    peer_connection::{
        configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeviceMetadata {
    device_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OfferData {
    sd: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct IceCandidate {
    candidate: String,
    sdp_mid: String,
    sdp_m_line_index: u16,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "kebab-case")]
enum SignalingMessage {
    DeviceMetadata(DeviceMetadata),
    Offer(OfferData),
    Answer(String),
    NewIceCandidate(IceCandidate),
}

pub async fn connect(
    host: &str,
    auth_token: Option<&str>,
) -> AnyResult<(Arc<RTCPeerConnection>, Arc<RTCDataChannel>)> {
    let url = format!("ws://{}/webrtc/signaling/client", host);

    let (ws_stream, _) = if let Some(token) = auth_token {
        let mut request = url.into_client_request()?;
        let cookie_value = format!("{}", token);
        request.headers_mut().insert(
            header::COOKIE,
            header::HeaderValue::from_str(&cookie_value)?,
        );
        connect_async(request).await?
    } else {
        connect_async(url).await?
    };

    let (write, mut read) = ws_stream.split();
    let write = Arc::new(Mutex::new(write));

    // 1. Initialize WebRTC.
    let mut setting_engine = webrtc::api::setting_engine::SettingEngine::default();
    setting_engine.set_srtp_protection_profiles(vec![
        SrtpProtectionProfile::Srtp_Aead_Aes_128_Gcm,
        SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80,
    ]);
    let media_engine = MediaEngine::default();
    let api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .with_media_engine(media_engine)
        .build();
    let config_rtc = RTCConfiguration::default();
    let peer_connection = Arc::new(api.new_peer_connection(config_rtc).await?);

    // Set up ICE candidate handler to send candidates to the server
    let write_clone = Arc::clone(&write);
    peer_connection.on_ice_candidate(Box::new(
        move |c: Option<RTCIceCandidate>| {
            let write_clone = Arc::clone(&write_clone);
            Box::pin(async move {
                if let Some(c) = c {
                    match c.to_json() {
                        Ok(candidate) => {
                            let msg = SignalingMessage::NewIceCandidate(IceCandidate {
                                candidate: candidate.candidate,
                                sdp_mid: candidate.sdp_mid.unwrap_or_default(),
                                sdp_m_line_index: candidate.sdp_mline_index.unwrap_or_default(),
                            });
                            match serde_json::to_string(&msg) {
                                Ok(json_msg) => {
                                    let mut w = write_clone.lock().await;
                                    if let Err(e) = w.send(Message::Text(json_msg.into())).await {
                                        warn!("Failed to send ICE candidate: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to serialize ICE candidate: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to convert ICE candidate to JSON: {}", e);
                        }
                    }
                }
            })
        },
    ));

    // 2. Wait for device-metadata message
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let msg: SignalingMessage = serde_json::from_str(&text)?;
        if let SignalingMessage::DeviceMetadata(_) = msg {
            info!("Device supports new signaling protocol.");
        } else {
            return Err(anyhow!("Expected device-metadata message."));
        }
    } else {
        return Err(anyhow!("Failed to read device-metadata from websocket."));
    }

    // 3. Create DataChannel
    let data_channel = peer_connection.create_data_channel("rpc", None).await?;
    data_channel.on_open(Box::new(move || {
        Box::pin(async move {
            debug!("✅ DataChannel 'rpc' is now open!");
        })
    }));

    // 4. Create offer and send it
    let offer = peer_connection.create_offer(None).await?;
    peer_connection.set_local_description(offer.clone()).await?;

    let offer_sd =
        base64::engine::general_purpose::STANDARD.encode(serde_json::to_string(&offer)?);
    let offer_msg = SignalingMessage::Offer(OfferData { sd: offer_sd });
    let mut w = write.lock().await;
    w.send(Message::Text(
        serde_json::to_string(&offer_msg)?.into(),
    ))
    .await?;

    // 5. Wait for answer
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let msg: SignalingMessage = serde_json::from_str(&text)?;
        if let SignalingMessage::Answer(sd) = msg {
            let decoded_answer = base64::engine::general_purpose::STANDARD.decode(sd)?;
            let answer_sdp: RTCSessionDescription = serde_json::from_slice(&decoded_answer)?;
            peer_connection.set_remote_description(answer_sdp).await?;
        } else {
            return Err(anyhow!("Expected answer message."));
        }
    } else {
        return Err(anyhow!("Failed to read answer from websocket."));
    }

    // 6. Handle ICE candidates in the background
    let pc_clone = Arc::clone(&peer_connection);
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(SignalingMessage::NewIceCandidate(candidate)) =
                    serde_json::from_str(&text)
                {
                    let res = pc_clone
                        .add_ice_candidate(RTCIceCandidateInit {
                            candidate: candidate.candidate,
                            sdp_mid: Some(candidate.sdp_mid),
                            sdp_mline_index: Some(candidate.sdp_m_line_index),
                            ..Default::default()
                        })
                        .await;
                    if let Err(e) = res {
                        warn!("Failed to add ICE candidate: {}", e);
                    }
                }
            }
        }
    });

    Ok((peer_connection, data_channel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_device_metadata() {
        let msg = SignalingMessage::DeviceMetadata(DeviceMetadata {
            device_version: "1.0.0".to_string(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"type":"device-metadata","data":{"deviceVersion":"1.0.0"}}"#
        );
        let deserialized: SignalingMessage = serde_json::from_str(&json).unwrap();
        matches!(deserialized, SignalingMessage::DeviceMetadata(_));
    }

    #[test]
    fn test_serialize_deserialize_offer() {
        let msg = SignalingMessage::Offer(OfferData {
            sd: "offer_sd".to_string(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"offer","data":{"sd":"offer_sd"}}"#);
        let deserialized: SignalingMessage = serde_json::from_str(&json).unwrap();
        matches!(deserialized, SignalingMessage::Offer(_));
    }

    #[test]
    fn test_serialize_deserialize_answer() {
        let msg = SignalingMessage::Answer("answer_sd".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"answer","data":"answer_sd"}"#);
        let deserialized: SignalingMessage = serde_json::from_str(&json).unwrap();
        matches!(deserialized, SignalingMessage::Answer(_));
    }

    #[test]
    fn test_serialize_deserialize_new_ice_candidate() {
        let msg = SignalingMessage::NewIceCandidate(IceCandidate {
            candidate: "candidate_str".to_string(),
            sdp_mid: "sdp_mid_str".to_string(),
            sdp_m_line_index: 1,
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"new-ice-candidate","data":{"candidate":"candidate_str","sdpMid":"sdp_mid_str","sdpMLineIndex":1}}"#);
        let deserialized: SignalingMessage = serde_json::from_str(&json).unwrap();
        matches!(deserialized, SignalingMessage::NewIceCandidate(_));
    }
}
