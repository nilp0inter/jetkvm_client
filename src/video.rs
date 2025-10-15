use anyhow::{anyhow, Result as AnyResult};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use webrtc::track::track_remote::TrackRemote;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

#[derive(Clone)]
pub struct VideoFrameCapture {
    track: Arc<Mutex<Option<Arc<TrackRemote>>>>,
}

impl VideoFrameCapture {
    pub fn new() -> Self {
        Self {
            track: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_track(&self, track: Arc<TrackRemote>) {
        info!("Video track set: codec={}", track.codec().capability.mime_type);
        let mut t = self.track.lock().await;
        *t = Some(track);
    }

    pub async fn has_track(&self) -> bool {
        let t = self.track.lock().await;
        t.is_some()
    }



    pub async fn save_screenshot_as_png(&self, output_path: &str, _width: u32, _height: u32) -> AnyResult<()> {
        gst::init()?;
        
        let track_guard = self.track.lock().await;
        let track = track_guard
            .as_ref()
            .ok_or_else(|| anyhow!("No video track available"))?
            .clone();
        drop(track_guard);
        
        info!("Building GStreamer pipeline...");
        
        let pipeline = gst::Pipeline::new();
        
        let appsrc = gst::ElementFactory::make("appsrc")
            .name("src")
            .property("format", gst::Format::Time)
            .property_from_str("stream-type", "stream")
            .build()?;
        
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .build()?;
        
        let caps = gst::Caps::builder("application/x-rtp")
            .field("media", "video")
            .field("clock-rate", 90000)
            .field("encoding-name", "H264")
            .field("payload", 102)
            .build();
        capsfilter.set_property("caps", &caps);
        
        let rtph264depay = gst::ElementFactory::make("rtph264depay").build()?;
        let h264parse = gst::ElementFactory::make("h264parse").build()?;
        let avdec_h264 = gst::ElementFactory::make("avdec_h264").build()?;
        let videoconvert = gst::ElementFactory::make("videoconvert").build()?;
        let pngenc = gst::ElementFactory::make("pngenc").build()?;
        
        let identity = gst::ElementFactory::make("identity")
            .property("signal-handoffs", true)
            .property("sync", true)
            .build()?;
        
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", output_path)
            .property("sync", false)
            .build()?;
        
        pipeline.add_many(&[
            &appsrc, &capsfilter, &rtph264depay, &h264parse,
            &avdec_h264, &videoconvert, &pngenc, &identity, &filesink
        ])?;
        
        gst::Element::link_many(&[
            &appsrc, &capsfilter, &rtph264depay, &h264parse,
            &avdec_h264, &videoconvert, &pngenc, &identity, &filesink
        ])?;
        
        let appsrc = appsrc.dynamic_cast::<gst_app::AppSrc>().unwrap();
        
        let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<()>(1);
        
        identity.connect("handoff", false, move |_| {
            let _ = frame_tx.try_send(());
            None
        });
        
        info!("Starting pipeline...");
        pipeline.set_state(gst::State::Playing)?;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        info!("Feeding RTP packets to pipeline...");
        
        let appsrc_clone = appsrc.clone();
        let track_clone = track.clone();
        let feed_task = tokio::spawn(async move {
            loop {
                match track_clone.read_rtp().await {
                    Ok((rtp_packet, _)) => {
                        let header_size = 12 + (rtp_packet.header.csrc.len() * 4);
                        let total_size = header_size + rtp_packet.payload.len();
                        let mut packet_bytes = Vec::with_capacity(total_size);
                        
                        let b0 = (rtp_packet.header.version << 6) | 
                                 ((rtp_packet.header.padding as u8) << 5) |
                                 ((rtp_packet.header.extension as u8) << 4) |
                                 (rtp_packet.header.csrc.len() as u8);
                        packet_bytes.push(b0);
                        
                        let b1 = ((rtp_packet.header.marker as u8) << 7) | rtp_packet.header.payload_type;
                        packet_bytes.push(b1);
                        
                        packet_bytes.extend_from_slice(&rtp_packet.header.sequence_number.to_be_bytes());
                        packet_bytes.extend_from_slice(&rtp_packet.header.timestamp.to_be_bytes());
                        packet_bytes.extend_from_slice(&rtp_packet.header.ssrc.to_be_bytes());
                        
                        for csrc in &rtp_packet.header.csrc {
                            packet_bytes.extend_from_slice(&csrc.to_be_bytes());
                        }
                        
                        packet_bytes.extend_from_slice(&rtp_packet.payload);
                        
                        let buffer = gst::Buffer::from_slice(packet_bytes);
                        if appsrc_clone.push_buffer(buffer).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(10));
        tokio::pin!(timeout);
        
        tokio::select! {
            _ = frame_rx.recv() => {
                info!("First frame written, stopping pipeline");
            }
            _ = &mut timeout => {
                return Err(anyhow!("Timeout waiting for first frame"));
            }
        }
        
        let _ = appsrc.end_of_stream();
        feed_task.abort();
        
        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::from_seconds(2)) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(..) => {
                    break;
                }
                MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null)?;
                    return Err(anyhow!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    ));
                }
                _ => {}
            }
        }
        
        pipeline.set_state(gst::State::Null)?;
        
        if std::path::Path::new(output_path).exists() && std::fs::metadata(output_path)?.len() > 0 {
            info!("Screenshot saved to {}", output_path);
            Ok(())
        } else {
            Err(anyhow!("Screenshot file was not created"))
        }
    }
}

impl Default for VideoFrameCapture {
    fn default() -> Self {
        Self::new()
    }
}
