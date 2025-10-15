use anyhow::{anyhow, Result as AnyResult};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;
use webrtc::{
    api::{media_engine::MediaEngine, APIBuilder},
    data_channel::RTCDataChannel,
    dtls::extension::extension_use_srtp::SrtpProtectionProfile,
    peer_connection::{
        configuration::RTCConfiguration,
        sdp::{sdp_type::RTCSdpType, session_description::RTCSessionDescription},
        RTCPeerConnection,
    },
};

#[derive(Serialize, Deserialize)]
struct WebRTCSessionRequest {
    sd: String,
}

#[derive(Serialize, Deserialize)]
struct WebRTCSessionResponse {
    sd: String,
}

pub async fn connect(
    http_client: &Client,
    host: &str,
    api: &str,
) -> AnyResult<(Arc<RTCPeerConnection>, Arc<RTCDataChannel>)> {
    // 2. Initialize WebRTC.
    let mut setting_engine = webrtc::api::setting_engine::SettingEngine::default();
    setting_engine.set_srtp_protection_profiles(vec![
        SrtpProtectionProfile::Srtp_Aead_Aes_128_Gcm,
        SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80,
    ]);
    let media_engine = MediaEngine::default();
    let webrtc_api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .with_media_engine(media_engine)
        .build();
    let config_rtc = RTCConfiguration::default();
    let peer_connection = Arc::new(webrtc_api.new_peer_connection(config_rtc).await?);
    debug!("PeerConnection created.");

    // 3. Create a DataChannel named "rpc".
    let data_channel = peer_connection.create_data_channel("rpc", None).await?;
    data_channel.on_open(Box::new(move || {
        Box::pin(async move {
            debug!("✅ DataChannel 'rpc' is now open!");
        })
    }));

    debug!("DataChannel created and awaiting connection.");

    // 5. Create an SDP offer and set it locally.
    let offer = peer_connection.create_offer(None).await?;
    peer_connection.set_local_description(offer.clone()).await?;
    debug!("Local SDP Offer set.");

    // 6. Wrap the offer in JSON.
    let offer_type_str = match offer.sdp_type {
        RTCSdpType::Offer => "offer",
        RTCSdpType::Answer => "answer",
        RTCSdpType::Pranswer => "pranswer",
        RTCSdpType::Rollback => "rollback",
        _ => return Err(anyhow!("Unsupported SDP type")),
    };

    #[derive(Serialize)]
    struct LocalOfferJson {
        sdp: String,
        #[serde(rename = "type")]
        sdp_type: String,
    }

    let local_offer_json = LocalOfferJson {
        sdp: offer.sdp.clone(),
        sdp_type: offer_type_str.to_owned(),
    };

    let offer_str = serde_json::to_string(&local_offer_json)?;
    let encoded_offer = general_purpose::STANDARD.encode(offer_str);
    let session_request = WebRTCSessionRequest { sd: encoded_offer };

    // 7. Send the offer to the server.
    let url = format!("http://{}{}", host, api);
    let response = http_client.post(&url).json(&session_request).send().await?;
    let response_text = response.text().await?;

    let session_response: WebRTCSessionResponse = serde_json::from_str(&response_text)?;
    let decoded_answer = general_purpose::STANDARD.decode(session_response.sd)?;
    let answer_value: Value = serde_json::from_slice(&decoded_answer)?;

    let sdp_field = answer_value
        .get("sdp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing sdp field in answer"))?;

    let sdp_type_str = answer_value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("answer");

    let remote_desc = match sdp_type_str {
        "offer" => RTCSessionDescription::offer(sdp_field.to_owned())?,
        "answer" => RTCSessionDescription::answer(sdp_field.to_owned())?,
        "pranswer" => RTCSessionDescription::pranswer(sdp_field.to_owned())?,
        "rollback" => return Err(anyhow!("Rollback not supported")),
        other => return Err(anyhow!("Unknown SDP type: {}", other)),
    };

    peer_connection.set_remote_description(remote_desc).await?;
    debug!("Remote SDP Answer set.");

    Ok((peer_connection, data_channel))
}
