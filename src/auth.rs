use anyhow::{anyhow, Result as AnyResult};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, info};

/// Logs in to JetKVM via HTTP and returns an authenticated reqwest::Client.
pub async fn login_local(host: &str, password: &str) -> AnyResult<Client> {
    let login_url = format!("http://{}/auth/login-local", host);
    let client = Client::builder().cookie_store(true).build()?;
    if password.len() == 0 {
        return Ok(client);
    }
    let resp = client
        .post(&login_url)
        .json(&json!({ "password": password }))
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read>".into());
        return Err(anyhow!("Login failed. Status: {}, Body: {}", status, body));
    }
    let header_map = resp.headers().clone();
    let body_text = resp.text().await.unwrap_or_default();
    info!("Login successful. Server responded: {}", body_text);
    for cookie in header_map.get_all(reqwest::header::SET_COOKIE).iter() {
        debug!("Set-Cookie: {}", cookie.to_str().unwrap_or_default());
    }
    Ok(client)
}
