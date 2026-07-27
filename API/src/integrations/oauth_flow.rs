use serde_json::Value;
use crate::errors::{AppError, AppResult};
use super::models::MicrosoftDeviceCodeInfo;

const MS_DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/devicecode";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token";

/// Initiate the Microsoft Identity Device Authorization Grant (P6-002).
/// Returns the device code info that the UI uses to show the user_code and
/// verification_url.  The user must visit the URL and enter the code while
/// the app polls for the token.
pub fn initiate_microsoft_device_code(
    client_id: &str,
    tenant_id: &str,
    scopes: &str,
) -> AppResult<MicrosoftDeviceCodeInfo> {
    let url = MS_DEVICE_CODE_URL.replace("{tenant}", tenant_id);
    let body = format!("client_id={}&scope={}", client_id, urlencoding_encode(scopes));

    let response: Value = ureq::post(&url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body)
        .map_err(|e| AppError::AiProvider(format!("Microsoft device code request failed: {e}")))?
        .into_json()
        .map_err(|e| AppError::Parse(format!("Microsoft device code response parse error: {e}")))?;

    let device_code = response["device_code"].as_str().ok_or_else(|| AppError::Parse("missing device_code".into()))?.to_string();
    let user_code = response["user_code"].as_str().ok_or_else(|| AppError::Parse("missing user_code".into()))?.to_string();
    let verification_url = response["verification_uri"].as_str().unwrap_or("https://microsoft.com/devicelogin").to_string();
    let expires_in = response["expires_in"].as_i64().unwrap_or(900);
    let interval = response["interval"].as_i64().unwrap_or(5);
    let message = response["message"].as_str().unwrap_or("").to_string();

    Ok(MicrosoftDeviceCodeInfo { device_code, user_code, verification_url, expires_in, interval, message })
}

/// Poll for a Microsoft access token after the user has completed device-code
/// authentication (P6-002).  Polls at `interval_secs` frequency for at most
/// `timeout_secs` seconds (typically matches the device code `expires_in`).
/// Returns `(access_token, refresh_token, expires_in_seconds)` on success.
pub fn poll_microsoft_token(
    device_code: &str,
    client_id: &str,
    tenant_id: &str,
    interval_secs: u64,
    timeout_secs: u64,
) -> AppResult<(String, Option<String>, i64)> {
    let url = MS_TOKEN_URL.replace("{tenant}", tenant_id);
    let body = format!(
        "grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code={}&client_id={}",
        device_code, client_id
    );

    let start = std::time::Instant::now();
    let sleep_dur = std::time::Duration::from_secs(interval_secs.max(2));

    loop {
        if start.elapsed().as_secs() >= timeout_secs {
            return Err(AppError::AiProvider("Microsoft device code authentication timed out".into()));
        }

        std::thread::sleep(sleep_dur);

        let response = match ureq::post(&url)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&body)
        {
            Ok(r) => r.into_json::<Value>().unwrap_or(Value::Null),
            Err(ureq::Error::Status(_, r)) => r.into_json::<Value>().unwrap_or(Value::Null),
            Err(e) => return Err(AppError::AiProvider(format!("token poll network error: {e}"))),
        };

        if let Some(error) = response["error"].as_str() {
            match error {
                "authorization_pending" => continue,
                "slow_down" => {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }
                "expired_token" | "bad_verification_code" => {
                    return Err(AppError::AiProvider(format!("device code error: {error}")));
                }
                _ => return Err(AppError::AiProvider(format!("unexpected OAuth error: {error}"))),
            }
        }

        if let Some(access_token) = response["access_token"].as_str() {
            let refresh_token = response["refresh_token"].as_str().map(|s| s.to_string());
            let expires_in = response["expires_in"].as_i64().unwrap_or(3600);
            return Ok((access_token.to_string(), refresh_token, expires_in));
        }
    }
}

/// Refresh a Microsoft access token using the stored refresh token (P6-002).
pub fn refresh_microsoft_token(
    refresh_token: &str,
    client_id: &str,
    tenant_id: &str,
) -> AppResult<(String, Option<String>, i64)> {
    let url = MS_TOKEN_URL.replace("{tenant}", tenant_id);
    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        refresh_token, client_id
    );

    let response: Value = ureq::post(&url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body)
        .map_err(|e| AppError::AiProvider(format!("token refresh failed: {e}")))?
        .into_json()
        .map_err(|e| AppError::Parse(format!("refresh response parse error: {e}")))?;

    let access_token = response["access_token"].as_str()
        .ok_or_else(|| AppError::AiProvider("refresh did not return access_token".into()))?
        .to_string();
    let new_refresh_token = response["refresh_token"].as_str().map(|s| s.to_string());
    let expires_in = response["expires_in"].as_i64().unwrap_or(3600);

    Ok((access_token, new_refresh_token, expires_in))
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            ':' => "%3A".to_string(),
            '+' | '.' | '_' | '-' | '~' => c.to_string(),
            c if c.is_ascii_alphanumeric() => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
