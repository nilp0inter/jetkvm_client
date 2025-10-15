use anyhow::{anyhow, Result as AnyResult};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use webrtc::track::track_remote::TrackRemote;

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
        info!(
            "Video track set: codec={}",
            track.codec().capability.mime_type
        );
        let mut t = self.track.lock().await;
        *t = Some(track);
    }

    pub async fn has_track(&self) -> bool {
        let t = self.track.lock().await;
        t.is_some()
    }

    pub async fn capture_screenshot_png(&self) -> AnyResult<Vec<u8>> {
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

        let capsfilter = gst::ElementFactory::make("capsfilter").build()?;

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

        let appsink = gst::ElementFactory::make("appsink")
            .property("emit-signals", true)
            .property("sync", false)
            .build()?;

        pipeline.add_many([
            &appsrc,
            &capsfilter,
            &rtph264depay,
            &h264parse,
            &avdec_h264,
            &videoconvert,
            &pngenc,
            &appsink,
        ])?;

        gst::Element::link_many([
            &appsrc,
            &capsfilter,
            &rtph264depay,
            &h264parse,
            &avdec_h264,
            &videoconvert,
            &pngenc,
            &appsink,
        ])?;

        let appsrc = appsrc.dynamic_cast::<gst_app::AppSrc>().unwrap();
        let appsink = appsink.dynamic_cast::<gst_app::AppSink>().unwrap();

        let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);

        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.as_slice().to_vec();
                    let _ = frame_tx.try_send(data);
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        info!("Starting pipeline...");
        pipeline.set_state(gst::State::Playing)?;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        info!("Feeding RTP packets to pipeline...");

        let appsrc_clone = appsrc.clone();
        let track_clone = track.clone();
        let feed_task = tokio::spawn(async move {
            while let Ok((rtp_packet, _)) = track_clone.read_rtp().await {
                let header_size = 12 + (rtp_packet.header.csrc.len() * 4);
                let total_size = header_size + rtp_packet.payload.len();
                let mut packet_bytes = Vec::with_capacity(total_size);

                let b0 = (rtp_packet.header.version << 6)
                    | ((rtp_packet.header.padding as u8) << 5)
                    | ((rtp_packet.header.extension as u8) << 4)
                    | (rtp_packet.header.csrc.len() as u8);
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
        });

        let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(10));
        tokio::pin!(timeout);

        let png_data = tokio::select! {
            Some(data) = frame_rx.recv() => {
                info!("First frame captured ({} bytes)", data.len());
                data
            }
            _ = &mut timeout => {
                return Err(anyhow!("Timeout waiting for first frame"));
            }
        };

        let _ = appsrc.end_of_stream();
        feed_task.abort();

        pipeline.set_state(gst::State::Null)?;

        info!("Screenshot captured successfully");
        Ok(png_data)
    }
}

impl Default for VideoFrameCapture {
    fn default() -> Self {
        Self::new()
    }
}
