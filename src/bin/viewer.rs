//! `jetkvm_viewer` — borderless fullscreen viewer for a JetKVM device.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result as AnyResult};
use clap::Parser;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{debug, error, info, warn};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::PhysicalKey;
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, Window, WindowId};

use jetkvm_client::jetkvm_rpc_client::{JetKvmRpcClient, SignalingMethod};
use jetkvm_client::keyboard::rpc_keyboard_report;
use jetkvm_client::mouse::{rpc_abs_mouse_report, rpc_wheel_report};
use jetkvm_client::system::rpc_set_edid;
use jetkvm_client::viewer::edid::{
    build_safe_edid_hex, jetkvm_default_edid_hex, try_build_edid_hex,
};
use jetkvm_client::viewer::input::{InputEvent, KeyboardState, MouseState};
use jetkvm_client::viewer::pipeline::{serialize_rtp, Frame, StreamingPipeline};
use jetkvm_client::viewer::render::Renderer;

#[derive(Parser, Debug, Clone)]
#[command(name = "jetkvm_viewer", version, about = "JetKVM live viewer")]
struct Args {
    #[arg(short = 'H', long)]
    host: String,
    #[arg(short = 'P', long)]
    password: String,
    #[arg(short = 'p', long, default_value_t = 80)]
    port: u16,
    #[arg(short = 'a', long, default_value = "/webrtc/session")]
    api: String,
    #[arg(short = 'd', long)]
    debug: bool,
    #[arg(long, value_enum, default_value_t = SignalingMethod::Auto)]
    signaling_method: SignalingMethod,
    /// Override the EDID refresh rate sent to the device (Hz). Defaults to the
    /// primary monitor's reported refresh rate, capped at 60 Hz.
    #[arg(long)]
    refresh: Option<u32>,
    /// Override the EDID resolution width (px). Defaults to the primary monitor.
    #[arg(long)]
    width: Option<u32>,
    /// Override the EDID resolution height (px). Defaults to the primary monitor.
    #[arg(long)]
    height: Option<u32>,
    /// Skip the setEDID step. Stream whatever the device is already configured
    /// to send; the renderer will letterbox if the resolution doesn't match the
    /// local window.
    #[arg(long)]
    no_edid: bool,
    /// Push the JetKVM factory-default EDID to the device and exit. Use when a
    /// previous custom EDID has wedged the host's HDMI output.
    #[arg(long)]
    reset_edid: bool,
}

#[derive(Debug, Clone)]
enum UserEvent {
    Ready { remote_w: u32, remote_h: u32 },
    FatalError(String),
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_rx: watch::Receiver<Option<Frame>>,
    input_tx: mpsc::UnboundedSender<InputEvent>,
    monitor_tx: Option<oneshot::Sender<(u32, u32, u32)>>,
    keyboard: KeyboardState,
    mouse: MouseState,
    last_cursor: PhysicalPosition<f64>,
    remote_size: Option<(u32, u32)>,
}

impl App {
    fn new(
        frame_rx: watch::Receiver<Option<Frame>>,
        input_tx: mpsc::UnboundedSender<InputEvent>,
        monitor_tx: oneshot::Sender<(u32, u32, u32)>,
    ) -> Self {
        Self {
            window: None,
            renderer: None,
            frame_rx,
            input_tx,
            monitor_tx: Some(monitor_tx),
            keyboard: KeyboardState::default(),
            mouse: MouseState::default(),
            last_cursor: PhysicalPosition::new(0.0, 0.0),
            remote_size: None,
        }
    }

    fn create_window_for_monitor(
        &mut self,
        event_loop: &ActiveEventLoop,
        monitor: MonitorHandle,
    ) -> AnyResult<()> {
        let attrs = Window::default_attributes()
            .with_title("JetKVM Viewer")
            .with_decorations(false)
            .with_resizable(false)
            .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        let window = event_loop
            .create_window(attrs)
            .map_err(|e| anyhow!("create_window: {e}"))?;
        let window = Arc::new(window);
        let _ = window.set_cursor_visible(false);
        let renderer = pollster::block_on(Renderer::new(window.clone()))?;
        self.window = Some(window);
        self.renderer = Some(renderer);
        Ok(())
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // On first resume, detect the primary monitor and ship its dimensions
        // off to the tokio client task so it can set the device EDID and start
        // streaming. The window itself is created later, in response to
        // UserEvent::Ready, once we have the first decoded frame.
        if let Some(tx) = self.monitor_tx.take() {
            let monitor = event_loop
                .primary_monitor()
                .or_else(|| event_loop.available_monitors().next());
            let (w, h, refresh) = match monitor {
                Some(m) => {
                    let s = m.size();
                    let r = (m.refresh_rate_millihertz().unwrap_or(60_000) + 500) / 1000;
                    (s.width, s.height, r)
                }
                None => {
                    warn!("no monitor detected — using 1920x1080@60");
                    (1920u32, 1080u32, 60u32)
                }
            };
            info!("local monitor: {}x{}@{} Hz", w, h, refresh);
            let _ = tx.send((w, h, refresh));
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Ready { remote_w, remote_h } => {
                self.remote_size = Some((remote_w, remote_h));
                let monitor = match event_loop
                    .primary_monitor()
                    .or_else(|| event_loop.available_monitors().next())
                {
                    Some(m) => m,
                    None => {
                        error!("no monitor available");
                        event_loop.exit();
                        return;
                    }
                };
                if let Err(e) = self.create_window_for_monitor(event_loop, monitor) {
                    error!("failed to create window: {e:#}");
                    event_loop.exit();
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            UserEvent::FatalError(msg) => {
                error!("fatal: {msg}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(r) = self.renderer.as_mut() {
                    if self.frame_rx.has_changed().unwrap_or(false) {
                        let f = self.frame_rx.borrow_and_update().clone();
                        if let Some(f) = f {
                            r.upload_frame(&f);
                        }
                    }
                    if let Err(e) = r.render() {
                        warn!("render error: {e}");
                    }
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor = position;
                if let (Some(win), Some((rw, rh))) = (&self.window, self.remote_size) {
                    let size = win.inner_size();
                    let ev = self.mouse.handle_motion(
                        position.x,
                        position.y,
                        size.width as f64,
                        size.height as f64,
                        rw,
                        rh,
                    );
                    let _ = self.input_tx.send(ev);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let ev = self.mouse.handle_button(button, state);
                let _ = self.input_tx.send(ev);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ev) = self.mouse.handle_wheel(delta) {
                    let _ = self.input_tx.send(ev);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        repeat,
                        ..
                    },
                is_synthetic,
                ..
            } => {
                debug!(
                    "KeyboardInput physical={physical_key:?} state={state:?} repeat={repeat} synthetic={is_synthetic}"
                );
                if repeat {
                    return;
                }
                let PhysicalKey::Code(code) = physical_key else {
                    debug!("dropping unidentified physical key");
                    return;
                };
                if let Some(ev) = self.keyboard.handle(code, state) {
                    if self.input_tx.send(ev).is_err() {
                        warn!("input channel closed; tokio side likely exited");
                    }
                } else {
                    debug!("keyboard state unchanged (no report emitted)");
                }
            }
            WindowEvent::Focused(focused) => {
                debug!("window focus = {focused}");
                if !focused {
                    // Release all held keys when focus is lost so the remote
                    // host doesn't see a stuck modifier.
                    self.keyboard = KeyboardState::default();
                    let _ = self.input_tx.send(InputEvent::Keyboard {
                        modifier: 0,
                        keys: vec![],
                    });
                }
            }
            _ => {}
        }
    }
}

fn install_logging(debug: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = if debug {
        EnvFilter::new(
            "jetkvm_client=debug,\
             jetkvm_viewer=debug,\
             info,\
             wgpu_hal::vulkan::instance=warn",
        )
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(
                "warn,\
                 jetkvm_viewer=info,\
                 jetkvm_client::jetkvm_rpc_client=info,\
                 wgpu_hal::vulkan::instance=off",
            )
        })
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn main() -> AnyResult<()> {
    let args = Args::parse();
    install_logging(args.debug);

    // rustls 0.23 panics if neither aws-lc-rs nor ring is unambiguously selected.
    // Both are pulled in via webrtc-dtls' transitive deps, so we must install a
    // process-wide default explicitly.
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| anyhow!("failed to install rustls crypto provider"))?;
    }

    let event_loop: EventLoop<UserEvent> = EventLoop::<UserEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let (frame_tx, frame_rx) = watch::channel::<Option<Frame>>(None);
    let (input_tx, input_rx) = mpsc::unbounded_channel::<InputEvent>();
    let (monitor_tx, monitor_rx) = oneshot::channel::<(u32, u32, u32)>();
    let proxy = event_loop.create_proxy();

    let args_for_tokio = args.clone();
    let frame_tx_for_tokio = frame_tx.clone();
    let proxy_for_tokio = proxy.clone();
    thread::Builder::new()
        .name("jetkvm-tokio".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = proxy_for_tokio
                        .send_event(UserEvent::FatalError(format!("tokio init: {e}")));
                    return;
                }
            };
            rt.block_on(async move {
                let (local_w, local_h, local_refresh) = match monitor_rx.await {
                    Ok(v) => v,
                    Err(_) => {
                        warn!("monitor channel dropped");
                        return;
                    }
                };
                if let Err(e) = run_client(
                    args_for_tokio,
                    proxy_for_tokio.clone(),
                    frame_tx_for_tokio,
                    input_rx,
                    local_w,
                    local_h,
                    local_refresh,
                )
                .await
                {
                    error!("client error: {e:#}");
                    let _ = proxy_for_tokio.send_event(UserEvent::FatalError(e.to_string()));
                }
            });
        })?;

    let _ = (args, proxy); // moved into tokio thread / handled there
    let mut app = App::new(frame_rx, input_tx, monitor_tx);
    event_loop.run_app(&mut app)?;
    Ok(())
}

async fn run_client(
    args: Args,
    proxy: EventLoopProxy<UserEvent>,
    frame_tx: watch::Sender<Option<Frame>>,
    mut input_rx: mpsc::UnboundedReceiver<InputEvent>,
    local_w: u32,
    local_h: u32,
    local_refresh: u32,
) -> AnyResult<()> {
    let host = if args.port == 80 {
        args.host.clone()
    } else {
        format!("{}:{}", args.host, args.port)
    };
    let mut client = JetKvmRpcClient::new(
        host,
        args.password.clone(),
        args.api.clone(),
        false,
        args.signaling_method.clone(),
    );

    info!("connecting...");
    client.connect().await?;
    client.wait_for_channel_open().await?;
    info!("data channel open");

    // --reset-edid: push the JetKVM factory default, then exit. Useful when a
    // previous custom EDID wedged the host's HDMI output and we can't even
    // get a video track.
    if args.reset_edid {
        info!("pushing JetKVM factory-default EDID");
        rpc_set_edid(&client, jetkvm_default_edid_hex()).await?;
        info!("default EDID restored");
        return Ok(());
    }

    // Set EDID BEFORE waiting for the video track. If a previous run left a
    // broken EDID on the device, the host computer is stuck on "no signal" and
    // the device will never add a video track to the RTC session. Pushing a
    // sane EDID first gives the upstream host a chance to renegotiate.
    if args.no_edid {
        info!("--no-edid set; keeping the device's existing EDID");
    } else if args.width.is_some() || args.height.is_some() || args.refresh.is_some() {
        // Explicit override: build a custom CVT-RB v1 mode from the user's
        // (possibly partial) arguments. Missing parts default to the local
        // monitor's reported values, with refresh capped at 60 Hz.
        let w = args.width.unwrap_or(local_w);
        let h = args.height.unwrap_or(local_h);
        let r = args.refresh.unwrap_or_else(|| local_refresh.min(60));
        match try_build_edid_hex(w, h, r) {
            Ok(edid_hex) => {
                info!("setting custom EDID to {w}x{h}@{r}");
                match rpc_set_edid(&client, edid_hex).await {
                    Ok(_) => info!("EDID set"),
                    Err(e) => warn!("setEDID failed: {e}; continuing"),
                }
            }
            Err(e) => {
                return Err(anyhow!(
                    "{e}. Pass --refresh 60 or smaller --width/--height, or \
                     --no-edid to skip the EDID override entirely."
                ));
            }
        }
    } else {
        // Auto-detected path: pick a curated safe mode whose aspect ratio
        // matches the local display and whose resolution is the closest
        // one that fits. CTA-861 timings for 1280x720/1920x1080, CVT-RB v1
        // for everything else.
        let (edid_hex, mode) = build_safe_edid_hex(local_w, local_h);
        info!(
            "setting safe EDID: {}x{}@{} ({:.2} MHz pixel clock)",
            mode.width,
            mode.height,
            mode.refresh_hz,
            mode.pixel_clock_khz as f64 / 1000.0,
        );
        match rpc_set_edid(&client, edid_hex).await {
            Ok(_) => info!("EDID set"),
            Err(e) => warn!("setEDID failed: {e}; continuing"),
        }
    }

    let mut track_rx = client.video_track_watcher();
    info!("waiting for video track (HDMI source must be connected to JetKVM)...");
    let track = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Some(t) = track_rx.borrow().clone() {
                return Ok::<_, anyhow::Error>(t);
            }
            track_rx
                .changed()
                .await
                .map_err(|_| anyhow!("track watcher closed"))?;
        }
    })
    .await
    .map_err(|_| {
        anyhow!(
            "timed out after 30s waiting for the JetKVM to send a video track. \
             Most likely the device reports no HDMI signal — connect a source \
             to the JetKVM and try again."
        )
    })??;
    info!("video track obtained");

    let pipeline = Arc::new(StreamingPipeline::new(frame_tx.clone())?);

    let pipeline_for_pump = pipeline.clone();
    let track_for_pump = track.clone();
    tokio::spawn(async move {
        loop {
            match track_for_pump.read_rtp().await {
                Ok((rtp, _)) => {
                    let bytes = serialize_rtp(&rtp);
                    if let Err(e) = pipeline_for_pump.push_rtp(bytes) {
                        warn!("pipeline push failed: {e}");
                        break;
                    }
                }
                Err(e) => {
                    warn!("read_rtp failed: {e}");
                    break;
                }
            }
        }
        debug!("RTP pump exiting");
    });

    let mut frame_watch = frame_tx.subscribe();
    let first_frame = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Some(f) = frame_watch.borrow().clone() {
                return Ok::<_, anyhow::Error>(f);
            }
            frame_watch.changed().await?;
        }
    })
    .await
    .map_err(|_| anyhow!("timeout waiting for first frame"))??;

    info!(
        "first frame: {}x{} stride_y={} stride_uv={}",
        first_frame.width, first_frame.height, first_frame.stride_y, first_frame.stride_uv
    );
    let _ = proxy.send_event(UserEvent::Ready {
        remote_w: first_frame.width,
        remote_h: first_frame.height,
    });

    // Dispatch input events serially: ordering matters for keyboard
    // (key-up after a key-down must arrive after, not in parallel).
    let client = Arc::new(client);
    while let Some(ev) = input_rx.recv().await {
        match ev {
            InputEvent::Keyboard { modifier, keys } => {
                debug!("keyboard report: mod=0x{modifier:02x} keys={keys:?}");
                if let Err(e) =
                    rpc_keyboard_report(&client, modifier as u64, keys).await
                {
                    warn!("keyboard report failed: {e}");
                }
            }
            InputEvent::AbsMouse { x, y, buttons } => {
                if let Err(e) =
                    rpc_abs_mouse_report(&client, x, y, buttons as u64).await
                {
                    warn!("abs mouse report failed: {e}");
                }
            }
            InputEvent::Wheel { dy } => {
                if let Err(e) = rpc_wheel_report(&client, dy).await {
                    warn!("wheel report failed: {e}");
                }
            }
        }
    }

    Ok(())
}
