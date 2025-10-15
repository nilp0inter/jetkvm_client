use anyhow::{anyhow, Result as AnyResult};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, info};

/// Logs in to JetKVM via HTTP and returns an authenticated reqwest::Client and an optional authToken.
pub async fn login_local(host: &str, password: &str) -> AnyResult<(Client, Option<String>)> {
    let login_url = format!("http://{}/auth/login-local", host);
    let client = Client::builder().cookie_store(true).build()?;
    if password.is_empty() {
        return Ok((client, None));
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

    let auth_token = header_map
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|cookie| {
            let cookie_str = cookie.to_str().unwrap_or_default();
            debug!("Set-Cookie: {}", cookie_str);
            if cookie_str.starts_with("authToken=") {
                cookie_str.split(';').next().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    Ok((client, auth_token))
}
