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
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

/// Number of consecutive decoded frames required after a (re)connect before
/// the window becomes visible. Short enough that healthy reconnects feel
/// instant, long enough to ride past the first-frame chroma greyness from
/// H.264 codec warm-up.
const FRAMES_TO_OK: u32 = 3;

/// Fixed delay between reconnect attempts. Applied whether a session ended
/// cleanly (peer Closed/Failed) or never came up at all.
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// How long the session loop tolerates no new decoded frames before it
/// declares the session dropped. webrtc-rs sometimes parks the peer state in
/// `Disconnected` for the full ICE timeout when the remote peer reboots
/// (TCP/UDP just stops, no graceful BYE), so the public state callback never
/// reaches `Failed`. This watchdog is the ground-truth liveness signal:
/// frames flowing = session healthy; no frames for N seconds = session dead,
/// regardless of what ICE thinks. 5 s is long enough to ride past normal
/// encoder pauses (H.264 still produces I-frames within a couple of seconds
/// on any live source), short enough to catch a reboot quickly.
const FRAME_WATCHDOG: Duration = Duration::from_secs(5);

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
    /// RCAS sharpening strength. AMD RCAS convention: lower = sharper,
    /// 0 = maximum sharpening, ~2 = no sharpening. Default 0.0.
    #[arg(long, default_value_t = 0.0)]
    sharpness: f32,
}

#[derive(Debug, Clone)]
enum UserEvent {
    /// Connection has reached the OK state — the renderer should be visible
    /// and rendering. On first occurrence the window is created; on
    /// subsequent occurrences (after a drop+reconnect) the existing window
    /// is just unhidden.
    Show,
    /// Connection has dropped or is not yet up. The window must be hidden;
    /// renderer state stays alive but no draws are submitted.
    Hide,
    /// One-shot signal from the tokio thread to terminate the event loop.
    /// Used only by the `--reset-edid` exit path.
    Quit,
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
    sharpness: f32,
    /// Tracks the desired visibility state. Drives the render loop: while
    /// `false`, `RedrawRequested` short-circuits and does not re-queue
    /// itself, so we stop calling `get_current_texture()` on a hidden
    /// surface. Re-enabled by `UserEvent::Show`.
    visible: bool,
}

impl App {
    fn new(
        frame_rx: watch::Receiver<Option<Frame>>,
        input_tx: mpsc::UnboundedSender<InputEvent>,
        monitor_tx: oneshot::Sender<(u32, u32, u32)>,
        sharpness: f32,
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
            sharpness,
            visible: false,
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
        let renderer = pollster::block_on(Renderer::new(window.clone(), self.sharpness))?;
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
            UserEvent::Show => {
                // First Show ever: create the window and renderer.
                if self.window.is_none() {
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
                        return;
                    }
                }
                // Reset local input state on every Show — the previous
                // session is gone, the remote is fresh, no key can still be
                // "held" from the user's perspective during the gap.
                self.keyboard = KeyboardState::default();
                self.mouse = MouseState::default();
                self.visible = true;
                if let Some(w) = &self.window {
                    w.set_visible(true);
                    w.request_redraw();
                }
            }
            UserEvent::Hide => {
                self.visible = false;
                self.keyboard = KeyboardState::default();
                self.mouse = MouseState::default();
                if let Some(w) = &self.window {
                    w.set_visible(false);
                }
            }
            UserEvent::Quit => {
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
                // While hidden, suspend the render loop entirely. Submitting
                // to a hidden surface is undefined on some platforms, and
                // self-requeueing requests would spin the CPU during gaps.
                if !self.visible {
                    return;
                }
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
                if let Some(win) = &self.window {
                    let size = win.inner_size();
                    let ev = self.mouse.handle_motion(
                        position.x,
                        position.y,
                        size.width as f64,
                        size.height as f64,
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
                    error!("tokio init failed: {e}");
                    let _ = proxy_for_tokio.send_event(UserEvent::Quit);
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
                run_with_reconnect(
                    args_for_tokio,
                    proxy_for_tokio,
                    frame_tx_for_tokio,
                    input_rx,
                    local_w,
                    local_h,
                    local_refresh,
                )
                .await;
            });
        })?;

    let sharpness = args.sharpness;
    let _ = (args, proxy); // moved into tokio thread / handled there
    let mut app = App::new(frame_rx, input_tx, monitor_tx, sharpness);
    event_loop.run_app(&mut app)?;
    Ok(())
}

/// Top-level reconnect loop. Runs sessions back-to-back forever; the only
/// exit conditions are the explicit `--reset-edid` branch (handled inline)
/// and the event loop being torn down from the GUI side (e.g.,
/// close-requested), at which point this future is simply dropped.
async fn run_with_reconnect(
    args: Args,
    proxy: EventLoopProxy<UserEvent>,
    frame_tx: watch::Sender<Option<Frame>>,
    mut input_rx: mpsc::UnboundedReceiver<InputEvent>,
    local_w: u32,
    local_h: u32,
    local_refresh: u32,
) {
    let host = if args.port == 80 {
        args.host.clone()
    } else {
        format!("{}:{}", args.host, args.port)
    };

    // --reset-edid: one-shot. Connect, push the factory EDID, ask the event
    // loop to exit. No reconnect logic applies.
    if args.reset_edid {
        match reset_edid_once(&args, &host).await {
            Ok(()) => info!("default EDID restored"),
            Err(e) => error!("reset-edid failed: {e:#}"),
        }
        let _ = proxy.send_event(UserEvent::Quit);
        return;
    }

    // EDID is pushed exactly once across the lifetime of the process. Held
    // across reconnects via this flag.
    let mut edid_pushed = false;

    loop {
        // Drain any queued input from a previous gap. Events arriving with
        // no active client would otherwise be replayed against the new
        // session with stale coordinates.
        while input_rx.try_recv().is_ok() {}

        let session_result = run_session(
            &args,
            &host,
            &proxy,
            &frame_tx,
            &mut input_rx,
            &mut edid_pushed,
            local_w,
            local_h,
            local_refresh,
        )
        .await;

        match session_result {
            Ok(()) => info!("session ended (peer Failed/Closed); reconnecting in 5s"),
            Err(e) => warn!("session failed ({e:#}); reconnecting in 5s"),
        }

        // Hide unconditionally — covers both "never reached Show" and "was
        // visible, now dropped". App::user_event clears local input state
        // so any held key from the previous session is forgotten on the
        // GUI side; the remote side gets a fresh zero-report on the next
        // Show via run_session's input-reset step.
        let _ = proxy.send_event(UserEvent::Hide);

        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Single connection attempt: connect, optionally push EDID, wait for the
/// video track, run frames + input until peer state goes Failed/Closed.
/// Returns `Ok` on a clean drop (peer-state-driven end); `Err` on any
/// failure during setup or runtime.
async fn run_session(
    args: &Args,
    host: &str,
    proxy: &EventLoopProxy<UserEvent>,
    frame_tx: &watch::Sender<Option<Frame>>,
    input_rx: &mut mpsc::UnboundedReceiver<InputEvent>,
    edid_pushed: &mut bool,
    local_w: u32,
    local_h: u32,
    local_refresh: u32,
) -> AnyResult<()> {
    let mut client = JetKvmRpcClient::new(
        host.to_string(),
        args.password.clone(),
        args.api.clone(),
        false,
        args.signaling_method.clone(),
    );

    info!("connecting...");
    client.connect().await?;
    client.wait_for_channel_open().await?;
    info!("data channel open");

    if !*edid_pushed {
        push_edid(args, &client, local_w, local_h, local_refresh).await?;
        *edid_pushed = true;
    } else {
        debug!("EDID already pushed earlier this process; skipping");
    }

    let mut peer_state_rx = client.peer_state_watcher();
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
             Most likely the device reports no HDMI signal."
        )
    })??;
    info!("video track obtained");

    let pipeline = Arc::new(StreamingPipeline::new(frame_tx.clone())?);

    let pipeline_for_pump = pipeline.clone();
    let track_for_pump = track.clone();
    let rtp_pump = tokio::spawn(async move {
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
                    debug!("read_rtp failed: {e}");
                    break;
                }
            }
        }
        debug!("RTP pump exiting");
    });

    // Clean-slate input report so the remote forgets any state that may
    // have lingered from before the session (held modifier, mouse button
    // stuck down). Best-effort: if the channel is already breaking we want
    // to proceed regardless.
    if let Err(e) = rpc_keyboard_report(&client, 0, vec![]).await {
        debug!("initial keyboard reset failed: {e}");
    }
    if let Err(e) = rpc_abs_mouse_report(&client, 0, 0, 0).await {
        debug!("initial mouse reset failed: {e}");
    }

    let mut frame_watch = frame_tx.subscribe();
    let mut frames_since_connect: u32 = 0;
    let mut shown = false;

    // Frame liveness watchdog. Reset to FRAME_WATCHDOG on every frame; if
    // it ever expires, the session is considered dead even if peer state
    // hasn't escalated.
    let watchdog = tokio::time::sleep(FRAME_WATCHDOG);
    tokio::pin!(watchdog);

    // Main per-session select loop. Four cooperating sources:
    //   1. Frames — count, send Show after FRAMES_TO_OK; reset watchdog.
    //   2. Peer state — detect drop via Failed/Closed.
    //   3. Watchdog — fallback drop signal when peer state wedges.
    //   4. Input events — dispatch RPCs serially (key ordering matters).
    let session_result = loop {
        tokio::select! {
            r = frame_watch.changed() => {
                if r.is_err() { break Err(anyhow!("frame channel closed")); }
                if frame_watch.borrow().is_some() {
                    watchdog
                        .as_mut()
                        .reset(tokio::time::Instant::now() + FRAME_WATCHDOG);
                    frames_since_connect = frames_since_connect.saturating_add(1);
                    if !shown && frames_since_connect >= FRAMES_TO_OK {
                        info!("connection OK after {} frames; showing window", frames_since_connect);
                        let _ = proxy.send_event(UserEvent::Show);
                        shown = true;
                    }
                }
            }
            r = peer_state_rx.changed() => {
                if r.is_err() { break Err(anyhow!("peer-state channel closed")); }
                let state = *peer_state_rx.borrow();
                debug!("peer-state observed: {:?}", state);
                if matches!(
                    state,
                    RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
                ) {
                    info!("peer state {:?}; ending session", state);
                    break Ok(());
                }
            }
            _ = &mut watchdog => {
                info!(
                    "no frames for {}s; ending session",
                    FRAME_WATCHDOG.as_secs()
                );
                break Ok(());
            }
            maybe_ev = input_rx.recv() => {
                let Some(ev) = maybe_ev else {
                    break Err(anyhow!("input channel closed (GUI exited)"));
                };
                match ev {
                    InputEvent::Keyboard { modifier, keys } => {
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
        }
    };

    // Tear down. Aborting the RTP pump first releases its borrow on the
    // track. Then explicitly close the peer connection — webrtc-rs's Drop
    // alone leaves the ICE agent running until lazy GC, which is what
    // produced the "agent is closed" warnings before. `pipeline` Drop runs
    // when the function returns and calls `set_state(Null)` on GStreamer.
    rtp_pump.abort();
    if let Some(pc) = client.peer_connection.as_ref() {
        if let Err(e) = pc.close().await {
            debug!("peer connection close error: {e}");
        }
    }

    session_result
}

/// One-shot used by `--reset-edid`: connect, push the factory EDID, return.
async fn reset_edid_once(args: &Args, host: &str) -> AnyResult<()> {
    let mut client = JetKvmRpcClient::new(
        host.to_string(),
        args.password.clone(),
        args.api.clone(),
        false,
        args.signaling_method.clone(),
    );
    info!("connecting (reset-edid mode)...");
    client.connect().await?;
    client.wait_for_channel_open().await?;
    info!("pushing JetKVM factory-default EDID");
    rpc_set_edid(&client, jetkvm_default_edid_hex()).await?;
    Ok(())
}

/// Pushes whichever EDID the CLI flags select (auto safe-mode, explicit
/// override, or skip). Idempotent shaping of what was previously inlined
/// into `run_client`.
async fn push_edid(
    args: &Args,
    client: &JetKvmRpcClient,
    local_w: u32,
    local_h: u32,
    local_refresh: u32,
) -> AnyResult<()> {
    if args.no_edid {
        info!("--no-edid set; keeping the device's existing EDID");
        return Ok(());
    }
    if args.width.is_some() || args.height.is_some() || args.refresh.is_some() {
        let w = args.width.unwrap_or(local_w);
        let h = args.height.unwrap_or(local_h);
        let r = args.refresh.unwrap_or_else(|| local_refresh.min(60));
        match try_build_edid_hex(w, h, r) {
            Ok(edid_hex) => {
                info!("setting custom EDID to {w}x{h}@{r}");
                match rpc_set_edid(client, edid_hex).await {
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
        let (edid_hex, mode) = build_safe_edid_hex(local_w, local_h);
        info!(
            "setting safe EDID: {}x{}@{} ({:.2} MHz pixel clock)",
            mode.width,
            mode.height,
            mode.refresh_hz,
            mode.pixel_clock_khz as f64 / 1000.0,
        );
        match rpc_set_edid(client, edid_hex).await {
            Ok(_) => info!("EDID set"),
            Err(e) => warn!("setEDID failed: {e}; continuing"),
        }
    }
    Ok(())
}
