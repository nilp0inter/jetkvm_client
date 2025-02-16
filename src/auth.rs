use anyhow::{anyhow, Result as AnyResult};
use reqwest::Client;
use serde_json::json;

/// Logs in to JetKVM via HTTP and returns an authenticated reqwest::Client.
pub async fn login_local(host: &str, password: &str) -> AnyResult<Client> {
    let login_url = format!("http://{}/auth/login-local", host);
    let client = Client::builder().cookie_store(true).build()?;
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
    println!("Login successful. Server responded: {}", body_text);
    for cookie in header_map.get_all(reqwest::header::SET_COOKIE).iter() {
        println!("Set-Cookie: {}", cookie.to_str().unwrap_or_default());
    }
    Ok(client)
}
