//! Streaming H.264 → NV12 GStreamer pipeline for the viewer.

use anyhow::{anyhow, Result as AnyResult};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use gstreamer_video::prelude::*;
use tokio::sync::watch;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub stride_y: u32,
    pub stride_uv: u32,
    pub y_plane: Vec<u8>,
    pub uv_plane: Vec<u8>,
}

pub struct StreamingPipeline {
    pipeline: gst::Pipeline,
    appsrc: gst_app::AppSrc,
}

impl StreamingPipeline {
    pub fn new(frame_tx: watch::Sender<Option<Frame>>) -> AnyResult<Self> {
        gst::init()?;

        let pipeline = gst::Pipeline::new();

        let appsrc = gst::ElementFactory::make("appsrc")
            .name("rtpsrc")
            .property("format", gst::Format::Time)
            .property("is-live", true)
            .property("do-timestamp", true)
            .property_from_str("stream-type", "stream")
            .build()?;

        let caps = gst::Caps::builder("application/x-rtp")
            .field("media", "video")
            .field("clock-rate", 90000)
            .field("encoding-name", "H264")
            .field("payload", 102)
            .build();
        appsrc.set_property("caps", &caps);

        let jitterbuffer = gst::ElementFactory::make("rtpjitterbuffer")
            .property("latency", 20u32)
            .build()?;
        let depay = gst::ElementFactory::make("rtph264depay").build()?;
        let parse = gst::ElementFactory::make("h264parse").build()?;
        let dec = gst::ElementFactory::make("avdec_h264").build()?;
        let convert = gst::ElementFactory::make("videoconvert").build()?;

        let nv12_caps = gst::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .build();
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", &nv12_caps)
            .build()?;

        let appsink = gst::ElementFactory::make("appsink")
            .property("emit-signals", true)
            .property("sync", false)
            .property("max-buffers", 2u32)
            .property("drop", true)
            .build()?;

        pipeline.add_many([
            &appsrc,
            &jitterbuffer,
            &depay,
            &parse,
            &dec,
            &convert,
            &capsfilter,
            &appsink,
        ])?;
        gst::Element::link_many([
            &appsrc,
            &jitterbuffer,
            &depay,
            &parse,
            &dec,
            &convert,
            &capsfilter,
            &appsink,
        ])?;

        let appsrc = appsrc.dynamic_cast::<gst_app::AppSrc>().unwrap();
        let appsink = appsink.dynamic_cast::<gst_app::AppSink>().unwrap();

        let frame_tx_cb = frame_tx.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                    let info = gst_video::VideoInfo::from_caps(caps)
                        .map_err(|_| gst::FlowError::Error)?;

                    let frame =
                        gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info)
                            .map_err(|_| gst::FlowError::Error)?;

                    let width = info.width();
                    let height = info.height();
                    let stride_y = frame.plane_stride()[0] as u32;
                    let stride_uv = frame.plane_stride()[1] as u32;

                    let y_data = frame.plane_data(0).map_err(|_| gst::FlowError::Error)?;
                    let uv_data = frame.plane_data(1).map_err(|_| gst::FlowError::Error)?;

                    let y_plane = y_data.to_vec();
                    let uv_plane = uv_data.to_vec();

                    let f = Frame {
                        width,
                        height,
                        stride_y,
                        stride_uv,
                        y_plane,
                        uv_plane,
                    };

                    let _ = frame_tx_cb.send(Some(f));
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline.set_state(gst::State::Playing)?;
        debug!("Streaming pipeline started");

        Ok(Self { pipeline, appsrc })
    }

    pub fn push_rtp(&self, rtp_bytes: Vec<u8>) -> AnyResult<()> {
        let buffer = gst::Buffer::from_slice(rtp_bytes);
        self.appsrc
            .push_buffer(buffer)
            .map_err(|e| anyhow!("appsrc push_buffer failed: {:?}", e))?;
        Ok(())
    }

    pub fn stop(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Null) {
            warn!("Failed to stop pipeline cleanly: {:?}", e);
        }
    }
}

impl Drop for StreamingPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Serialise a webrtc::rtp::packet::Packet back to wire bytes for appsrc.
pub fn serialize_rtp(rtp: &webrtc::rtp::packet::Packet) -> Vec<u8> {
    let header_size = 12 + rtp.header.csrc.len() * 4;
    let total = header_size + rtp.payload.len();
    let mut out = Vec::with_capacity(total);

    let b0 = (rtp.header.version << 6)
        | ((rtp.header.padding as u8) << 5)
        | ((rtp.header.extension as u8) << 4)
        | (rtp.header.csrc.len() as u8);
    out.push(b0);

    let b1 = ((rtp.header.marker as u8) << 7) | rtp.header.payload_type;
    out.push(b1);

    out.extend_from_slice(&rtp.header.sequence_number.to_be_bytes());
    out.extend_from_slice(&rtp.header.timestamp.to_be_bytes());
    out.extend_from_slice(&rtp.header.ssrc.to_be_bytes());

    for csrc in &rtp.header.csrc {
        out.extend_from_slice(&csrc.to_be_bytes());
    }

    out.extend_from_slice(&rtp.payload);
    out
}
