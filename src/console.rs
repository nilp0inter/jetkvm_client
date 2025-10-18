use anyhow::Result as AnyResult;
use bytes::Bytes;
use serde_json::{json, Value};
use std::io::{self, Write};
use std::sync::Arc;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use webrtc::data_channel::RTCDataChannel;

pub async fn open_console(serial_channel: Arc<RTCDataChannel>) -> AnyResult<Value> {
    serial_channel.on_message(Box::new(move |msg| {
        let mut stderr = io::stderr();
        stderr.write_all(&msg.data).unwrap();
        stderr.flush().unwrap();
        Box::pin(async {})
    }));

    let mut stdin = io::stdin().keys();
    let mut stderr = io::stderr().into_raw_mode()?;

    let result = async {
        loop {
            if let Some(Ok(key)) = stdin.next() {
                match key {
                    termion::event::Key::Char(c) => {
                        serial_channel.send(&Bytes::from(vec![c as u8])).await?;
                    }
                    termion::event::Key::Ctrl(c) => {
                        if c.is_alphabetic() {
                            serial_channel
                                .send(&Bytes::from(vec![(c as u8) - 96]))
                                .await?;
                        } else if c == '4' {
                            break;
                        }
                    }
                    termion::event::Key::Backspace => {
                        serial_channel.send(&Bytes::from(vec![8])).await?;
                    }
                    termion::event::Key::Esc => {
                        serial_channel.send(&Bytes::from(vec![27])).await?;
                    }
                    termion::event::Key::Up => {
                        serial_channel.send(&Bytes::from_static(b"\x1b[A")).await?;
                    }
                    termion::event::Key::Down => {
                        serial_channel.send(&Bytes::from_static(b"\x1b[B")).await?;
                    }
                    termion::event::Key::Right => {
                        serial_channel.send(&Bytes::from_static(b"\x1b[C")).await?;
                    }
                    termion::event::Key::Left => {
                        serial_channel.send(&Bytes::from_static(b"\x1b[D")).await?;
                    }
                    _ => {}
                }
            }
        }
        AnyResult::Ok(())
    }.await;

    stderr.flush()?;
    drop(stderr);

    result.map(|_| json!({ "status": "ok" }))
}
