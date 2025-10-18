use crate::auth;
use crate::rpc_client::RpcClient;
use crate::signaling::{legacy, websocket};
use crate::video::VideoFrameCapture;
use anyhow::{anyhow, Result as AnyResult};
use clap::ValueEnum;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tokio::time::Duration;
use tracing::{debug, info, warn};

use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::track::track_remote::TrackRemote;

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum SignalingMethod {
    #[default]
    Auto,
    Legacy,
    WebSocket,
}

/// `JetKvmRpcClient` encapsulates both an authenticated HTTP client and an established
/// WebRTC JSON-RPC connection.
use webrtc::peer_connection::RTCPeerConnection;

pub struct JetKvmRpcClient {
    pub host: String,
    pub password: String,
    pub api: String,
    pub no_auto_logout: bool,
    pub http_client: Option<Client>,
    pub auth_token: Option<String>,
    pub rpc_client: Option<RpcClient>,
    pub serial_client: Option<Arc<RTCDataChannel>>,
    pub peer_connection: Option<Arc<RTCPeerConnection>>,
    pub screen_size: Arc<Mutex<Option<(u32, u32)>>>,
    pub signaling_method: SignalingMethod,
    pub video_capture: Arc<VideoFrameCapture>,
    video_track_tx: watch::Sender<Option<Arc<TrackRemote>>>,
    video_track_rx: watch::Receiver<Option<Arc<TrackRemote>>>,
    peer_state_tx: watch::Sender<RTCPeerConnectionState>,
    peer_state_rx: watch::Receiver<RTCPeerConnectionState>,
}

impl JetKvmRpcClient {
    /// Creates a new `JetKvmRpcClient` without connecting.
    pub fn new(
        host: String,
        password: String,
        api: String,
        no_auto_logout: bool,
        signaling_method: SignalingMethod,
    ) -> Self {
        debug!("Initializing JetKvmRpcClient with host: {}", host);
        let (video_track_tx, video_track_rx) = watch::channel(None);
        let (peer_state_tx, peer_state_rx) = watch::channel(RTCPeerConnectionState::New);
        Self {
            host,
            password,
            api,
            no_auto_logout,
            http_client: None,
            auth_token: None,
            rpc_client: None,
            serial_client: None,
            peer_connection: None,
            screen_size: Arc::new(Mutex::new(None)),
            signaling_method,
            video_capture: Arc::new(VideoFrameCapture::new()),
            video_track_tx,
            video_track_rx,
            peer_state_tx,
            peer_state_rx,
        }
    }

    /// Returns a `watch::Receiver` that receives the remote video `TrackRemote`
    /// once the WebRTC `on_track` callback fires. Useful for the viewer, which
    /// reads RTP packets directly off the track and feeds them into its own
    /// streaming GStreamer pipeline.
    pub fn video_track_watcher(&self) -> watch::Receiver<Option<Arc<TrackRemote>>> {
        self.video_track_rx.clone()
    }

    /// Returns a `watch::Receiver` that observes the WebRTC peer-connection
    /// state. Consumers can `await` `changed()` and inspect the latest value
    /// via `borrow()` to detect transitions to `Disconnected`, `Failed`, or
    /// `Closed` — used by the viewer's reconnect loop to detect drops.
    pub fn peer_state_watcher(&self) -> watch::Receiver<RTCPeerConnectionState> {
        self.peer_state_rx.clone()
    }

    /// Connects the client to the JetKVM service.
    pub async fn connect(&mut self) -> AnyResult<()> {
        debug!("Connecting to JetKVM...");

        // 1. Authenticate via HTTP.
        let (http_client, auth_token) = auth::login_local(&self.host, &self.password).await?;
        debug!("Authentication successful.");
        self.http_client = Some(http_client.clone());
        self.auth_token = auth_token;

        let (peer_connection, rpc_channel) = match self.signaling_method {
            SignalingMethod::Legacy => legacy::connect(&http_client, &self.host, &self.api).await?,
            SignalingMethod::WebSocket => {
                websocket::connect(&self.host, self.auth_token.as_deref()).await?
            }
            SignalingMethod::Auto => {
                match websocket::connect(&self.host, self.auth_token.as_deref()).await {
                    Ok(conn) => {
                        info!("Successfully connected using WebSocket signaling.");
                        conn
                    }
                    Err(e) => {
                        warn!(
                            "WebSocket connection failed: {}. Falling back to legacy signaling.",
                            e
                        );
                        legacy::connect(&http_client, &self.host, &self.api).await?
                    }
                }
            }
        };

        let video_capture = Arc::clone(&self.video_capture);
        let video_track_tx = self.video_track_tx.clone();
        peer_connection.on_track(Box::new(move |track, _, _| {
            let video_capture = Arc::clone(&video_capture);
            let video_track_tx = video_track_tx.clone();
            Box::pin(async move {
                debug!("Received track: kind={}", track.kind());
                if track.kind() == webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video {
                    debug!("Setting video track for capture");
                    video_capture.set_track(track.clone()).await;
                    let _ = video_track_tx.send(Some(track));
                }
            })
        }));

        let peer_state_tx = self.peer_state_tx.clone();
        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            let peer_state_tx = peer_state_tx.clone();
            Box::pin(async move {
                debug!("Peer connection state changed: {:?}", state);
                let _ = peer_state_tx.send(state);
            })
        }));

        let rpc_client = RpcClient::new(rpc_channel);
        rpc_client.install_message_handler();
        self.rpc_client = Some(rpc_client);
        self.peer_connection = Some(peer_connection);

        debug!("JetKvmRpcClient connected successfully.");
        Ok(())
    }

    /// Sends an RPC request if the client is connected.
    pub async fn send_rpc(&self, method: &str, params: Value) -> AnyResult<Value> {
        match &self.rpc_client {
            Some(rpc) => rpc.send_rpc(method, params).await,
            None => Err(anyhow!(
                "RPC client is not connected. Call `connect()` first."
            )),
        }
    }

    /// Waits for the WebRTC DataChannel to be open.
    pub async fn wait_for_channel_open(&self) -> AnyResult<()> {
        if let Some(rpc_client) = &self.rpc_client {
            loop {
                if format!("{:?}", rpc_client.dc.ready_state()) == "Open" {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } else {
            Err(anyhow!(
                "RPC client is not connected. Call `connect()` first."
            ))
        }
    }
    pub async fn ensure_connected(&mut self) -> AnyResult<()> {
        if self.rpc_client.is_none() {
            self.connect().await?;
        }
        Ok(())
    }

    /// Creates a new serial data channel.
    pub async fn create_serial_channel(&self) -> AnyResult<Arc<RTCDataChannel>> {
        match &self.peer_connection {
            Some(pc) => {
                let serial_channel = pc.create_data_channel("serial", None).await?;
                serial_channel.on_open(Box::new(move || {
                    Box::pin(async move {
                        debug!("✅ DataChannel 'serial' is now open!");
                    })
                }));
                Ok(serial_channel)
            }
            None => Err(anyhow!(
                "Peer connection is not available. Call `connect()` first."
            )),
        }
    }
    /// Asynchronous logout function for normal use.
    pub async fn logout(&self) -> AnyResult<()> {
        if let Some(client) = &self.http_client {
            let url = format!("http://{}/auth/logout", self.host);
            let resp = client.post(&url).send().await;

            match resp {
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Failed to read body".to_string());
                    tracing::info!("Logout Response [{}]: {}", status, body);
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Logout request failed: {}", e);
                    Err(anyhow::anyhow!("Logout request failed: {}", e))
                }
            }
        } else {
            tracing::warn!("No HTTP client available, skipping logout.");
            Ok(())
        }
    }

    /// Gracefully disconnects by logging out and closing the RPC connection.
    pub async fn shutdown(&mut self) {
        if self.no_auto_logout {
            tracing::info!("Auto-logout is disabled in config, skipping logout.");
        } else if let Err(e) = self.logout().await {
            tracing::warn!("Failed to logout on shutdown: {}", e);
        }

        if let Some(rpc) = self.rpc_client.take() {
            tracing::info!("Closing WebRTC RPC connection...");
            let _ = rpc.dc.close().await;
        }

        tracing::info!("JetKvmRpcClient shutdown completed.");
    }
}

impl Drop for JetKvmRpcClient {
    fn drop(&mut self) {
        tracing::info!("JetKvmRpcClient dropped.");
    }
}
