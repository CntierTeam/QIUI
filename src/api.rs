//! HTTP client: API list discover, login, encrypted feign calls.

use crate::crypto::{self, CryptoError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const API_LIST_URL: &str =
    "https://qiui-cn.oss-cn-shenzhen.aliyuncs.com/file/urlList/api.html";
pub const DEFAULT_BASE: &str = "https://appapi.qiuitoy.com";
pub const APP_VERSION: &str = "7.0.51";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("crypto: {0}")]
    Crypto(#[from] CryptoError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("api state={state} message={message}")]
    Business { state: String, message: String },
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoints {
    pub http_url: String,
    pub http_image: String,
    pub http_chat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub token: Option<String>,
    pub uid: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "merchantId")]
    pub merchant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub state: Option<String>,
    pub message: Option<String>,
    pub data: Option<T>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::blocking::Client,
    pub base: String,
    pub token: Option<String>,
}

impl Client {
    pub fn new(base: impl Into<String>) -> Result<Self, ApiError> {
        let http = reqwest::blocking::Client::builder()
            .user_agent(format!("qiui-cli/0.1 (compat {APP_VERSION})"))
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            base: base.into().trim_end_matches('/').to_string(),
            token: None,
        })
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn discover() -> Result<(Self, ApiEndpoints), ApiError> {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let raw = http.get(API_LIST_URL).send()?.error_for_status()?.text()?;
        let plain = crypto::decrypt_api(raw.trim())?;
        let eps = parse_api_list(&plain)?;
        // App stores https://host/api then strips /api → https://host
        let base = normalize_http_url(&eps.http_url);
        Ok((Self::new(base)?, eps))
    }

    fn url(&self, path: &str) -> String {
        let p = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!("{}{p}", self.base)
    }

    /// POST encrypted JSON body; response is encrypted envelope.
    pub fn post_encrypted<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<ApiEnvelope<T>, ApiError> {
        let plain = serde_json::to_string(body)?;
        let enc = crypto::encrypt_api(&plain)?;
        let mut req = self
            .http
            .post(self.url(path))
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Connection", "keep-alive")
            .body(enc);
        if let Some(t) = &self.token {
            req = req.header("token", t);
        }
        let raw = req.send()?.error_for_status()?.text()?;
        let dec = crypto::decrypt_api(raw.trim())?;
        let env: ApiEnvelope<T> = serde_json::from_str(&dec)?;
        if let Some(state) = &env.state {
            if state != "success" && state != "ok" && state != "200" && state != "2000" {
                // QIUI uses various success markers; treat explicit failed.
                if state.eq_ignore_ascii_case("failed") || state.eq_ignore_ascii_case("error") {
                    return Err(ApiError::Business {
                        state: state.clone(),
                        message: env.message.clone().unwrap_or_default(),
                    });
                }
            }
        }
        Ok(env)
    }

    /// Password login. `login_type`: "2"=email+pwd, "3"=phone+pwd (from LoginActivity01).
    pub fn login_password(
        &mut self,
        user_name: &str,
        password: &str,
        login_type: &str,
    ) -> Result<LoginData, ApiError> {
        let body = json!({
            "platformType": "2",
            "userName": user_name,
            "versionNum": APP_VERSION,
            "loginType": login_type,
            "passWord": crypto::encrypt_password(password)?,
            "appLanguage": "0",
            "phoneId": format!("qiui-cli-{}", uuid::Uuid::new_v4()),
            "blackBox": "",
            "deviceType": "2",
        });
        let env: ApiEnvelope<LoginData> = self.post_encrypted("/feign/userInfo/login", &body)?;
        let data = env
            .data
            .ok_or_else(|| ApiError::Msg(format!("login missing data: {:?}", env.message)))?;
        if let Some(t) = &data.token {
            self.token = Some(t.clone());
        } else {
            return Err(ApiError::Msg(format!(
                "login ok-ish but no token (state={:?} msg={:?})",
                env.state, env.message
            )));
        }
        Ok(data)
    }

    pub fn get_binding_devices(&self) -> Result<Value, ApiError> {
        let env: ApiEnvelope<Value> =
            self.post_encrypted("/feign/toyUserBinding/getUserBindingToyDevices", &json!({}))?;
        if let Some(state) = &env.state {
            if state.eq_ignore_ascii_case("failed") {
                return Err(ApiError::Business {
                    state: state.clone(),
                    message: env.message.unwrap_or_default(),
                });
            }
        }
        Ok(env.data.unwrap_or(Value::Null))
    }

    /// Ask cloud for a BLE session token command (Cellmate). Returns plaintext hex for GATT write
    /// after `BLUETOOTH_COMMAND` decrypt.
    pub fn cellmate_get_toy_token(&self, toy_uid: &str) -> Result<String, ApiError> {
        let toy = format!(
            "{}_{}",
            toy_uid,
            chrono::Utc::now().timestamp_millis()
        );
        let body = json!({
            "toyUid": crypto::encrypt_timestamp_field(&toy)?,
        });
        let env: ApiEnvelope<Value> =
            self.post_encrypted("/feign/toyCellmateBluetooth/getToyToken", &body)?;
        extract_bt_command(env)
    }

    pub fn cellmate_close_lock(&self, toy_uid: &str) -> Result<String, ApiError> {
        let toy = format!(
            "{}_{}",
            toy_uid,
            chrono::Utc::now().timestamp_millis()
        );
        let body = json!({
            "toyUid": crypto::encrypt_timestamp_field(&toy)?,
        });
        let env: ApiEnvelope<Value> =
            self.post_encrypted("/feign/toyCellmateBluetooth/toyCloseLock", &body)?;
        extract_bt_command(env)
    }

    /// Upload notify hex for server-side decrypt.
    pub fn cellmate_decry_command(&self, lock_command_hex: &str) -> Result<Value, ApiError> {
        let body = json!({
            "lockCommand": crypto::encrypt_bt_command(lock_command_hex)?,
        });
        let env: ApiEnvelope<Value> =
            self.post_encrypted("/feign/toyCellmateBluetooth/decryBluetoothCommand", &body)?;
        Ok(env.data.unwrap_or(Value::Null))
    }
}

fn extract_bt_command(env: ApiEnvelope<Value>) -> Result<String, ApiError> {
    if let Some(state) = &env.state {
        if state.eq_ignore_ascii_case("failed") {
            return Err(ApiError::Business {
                state: state.clone(),
                message: env.message.unwrap_or_default(),
            });
        }
    }
    let data = env
        .data
        .ok_or_else(|| ApiError::Msg(format!("no data: {:?}", env.message)))?;
    // data may be a string (encrypted bt cmd) or object with fields
    let enc = match &data {
        Value::String(s) => s.clone(),
        Value::Object(m) => m
            .get("lockCommand")
            .or_else(|| m.get("data"))
            .or_else(|| m.get("command"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::Msg(format!("unexpected bt payload: {data}")))?
            .to_string(),
        other => {
            return Err(ApiError::Msg(format!("unexpected bt payload: {other}")));
        }
    };
    Ok(crypto::decrypt_bt_command(&enc)?)
}

fn parse_api_list(plain: &str) -> Result<ApiEndpoints, ApiError> {
    // CH_HTTP_URL="appapi.qiuitoy.com/api",CH_HTTP_IMAGE="...",...
    let mut http_url = None;
    let mut http_image = None;
    let mut http_chat = None;
    for part in plain.split(',') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next().unwrap_or("").trim();
        let v = kv
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_string();
        match k {
            "CH_HTTP_URL" | "US_HTTP_URL" => http_url = Some(v),
            "CH_HTTP_IMAGE" | "US_HTTP_IMAGE" => http_image = Some(v),
            "CH_HTTP_CHAT" | "US_HTTP_CHAT" => http_chat = Some(v),
            _ => {}
        }
    }
    Ok(ApiEndpoints {
        http_url: http_url.ok_or_else(|| ApiError::Msg("missing HTTP_URL".into()))?,
        http_image: http_image.unwrap_or_default(),
        http_chat: http_chat.unwrap_or_default(),
    })
}

fn normalize_http_url(host_path: &str) -> String {
    let mut s = if host_path.starts_with("http") {
        host_path.to_string()
    } else {
        format!("https://{host_path}")
    };
    if let Some(i) = s.find("/api") {
        s.truncate(i);
    }
    s.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list() {
        let p = r#"CH_HTTP_URL="appapi.qiuitoy.com/api",CH_HTTP_IMAGE="accelerate.qiuitoy.com",CH_HTTP_CHAT="appsocket.qiuitoy.com/socket""#;
        let e = parse_api_list(p).unwrap();
        assert_eq!(normalize_http_url(&e.http_url), "https://appapi.qiuitoy.com");
    }
}
