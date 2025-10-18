use crate::jetkvm_rpc_client::JetKvmRpcClient;
use anyhow::Result as AnyResult;
use serde_json::{json, Value};
use std::io::{self, Write};

use termion::raw::IntoRawMode;
use termion::input::TermRead;

pub async fn open_console(client: &JetKvmRpcClient) -> AnyResult<Value> {
    let serial_client = client
        .serial_client
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Serial client not connected"))?;

    serial_client.on_message(Box::new(move |msg| {
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
                    // termion::event::Key::Ctrl('\\') => {
                    //     break;
                    // }
                    termion::event::Key::Char(c) => {
                        client.send_serial(&[c as u8]).await?;
                    }
                    termion::event::Key::Ctrl(c) => {
                        if c.is_alphabetic() {
                            client.send_serial(&[(c as u8) - 96]).await?;
                        } else if c == '4' {  // Corresponds to Ctrl+\
                            break;
                        }
                    }
                    termion::event::Key::Backspace => {
                        client.send_serial(&[8]).await?;
                    }
                    termion::event::Key::Esc => {
                        client.send_serial(&[27]).await?;
                    }
                    termion::event::Key::Up => {
                        client.send_serial(b"\x1b[A").await?;
                    }
                    termion::event::Key::Down => {
                        client.send_serial(b"\x1b[B").await?;
                    }
                    termion::event::Key::Right => {
                        client.send_serial(b"\x1b[C").await?;
                    }
                    termion::event::Key::Left => {
                        client.send_serial(b"\x1b[D").await?;
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
