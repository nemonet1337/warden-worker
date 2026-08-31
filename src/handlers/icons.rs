use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use worker::{Env, Fetch, HttpMetadata, Method, Request, RequestInit};

use crate::handlers::attachments::{get_storage_backend, StorageBackend};
use crate::handlers::enforce_ip_rate_limit;

const ATTACHMENTS_BUCKET: &str = "ATTACHMENTS_BUCKET";
const ATTACHMENTS_KV: &str = "ATTACHMENTS_KV";
const RATE_LIMITER: &str = "ICON_RATE_LIMITER";

const CACHE_TTL_SECS: u64 = 2_592_000; // 30 days (Vaultwarden default)
const NEG_TTL_SECS: u64 = 259_200; // 3 days
const MAX_ICON_BYTES: usize = 256 * 1024;
const MIN_ICON_BYTES: usize = 16;

/// 1x1 transparent PNG used when no favicon can be fetched.
const FALLBACK_ICON: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

pub(crate) struct Icon {
    bytes: Vec<u8>,
    kind: &'static str,
    miss: bool,
}

impl IntoResponse for Icon {
    fn into_response(self) -> Response {
        let cache_control = if self.miss {
            HeaderValue::from_static("public, max-age=259200")
        } else {
            HeaderValue::from_static("public, max-age=2592000")
        };
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, content_type_value(self.kind));
        headers.insert(header::CACHE_CONTROL, cache_control);
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        (headers, self.bytes).into_response()
    }
}

fn fallback() -> Icon {
    Icon {
        bytes: FALLBACK_ICON.to_vec(),
        kind: "png",
        miss: true,
    }
}

fn content_type_value(kind: &'static str) -> HeaderValue {
    match kind {
        "png" => HeaderValue::from_static("image/png"),
        "x-icon" => HeaderValue::from_static("image/x-icon"),
        "jpeg" => HeaderValue::from_static("image/jpeg"),
        "gif" => HeaderValue::from_static("image/gif"),
        "webp" => HeaderValue::from_static("image/webp"),
        "bmp" => HeaderValue::from_static("image/bmp"),
        _ => HeaderValue::from_static("image/png"),
    }
}

fn storage_key(host: &str) -> String {
    format!("icons/{host}")
}

/// GET /icons/{domain}/icon.png
#[worker::send]
pub async fn get_icon(
    State(env): State<Arc<Env>>,
    headers: HeaderMap,
    Path(domain): Path<String>,
) -> Icon {
    let Some(host) = normalize_host(&domain) else {
        log::warn!("Invalid icon host: {domain}");
        return fallback();
    };

    if let Some(cached) = load_cached(&env, &host).await {
        return cached;
    }

    if enforce_ip_rate_limit(
        &env,
        &headers,
        RATE_LIMITER,
        "icon",
        "Too many icon requests. Please try again later.",
    )
    .await
    .is_err()
    {
        return fallback();
    }

    let icon = download_icon(&host).await.unwrap_or_else(fallback);
    store_cached(&env, &host, &icon).await;
    icon
}

fn normalize_host(raw: &str) -> Option<String> {
    let host = raw.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.len() < 3 || host.len() > 253 {
        return None;
    }
    if host.starts_with('[') || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    if host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".lan")
    {
        return None;
    }
    if !host.contains('.') {
        return None;
    }
    if host
        .chars()
        .any(|c| matches!(c, '/' | '\\' | ':' | '?' | '#' | '@' | ' ' | '%'))
    {
        return None;
    }
    let labels_ok = host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    });
    labels_ok.then_some(host)
}

fn icon_type(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [137, 80, 78, 71, 13, 10, 26, 10, ..] => Some("png"),
        [0, 0, 1, 0, n1, n2, ..] if u16::from_le_bytes([*n1, *n2]) > 0 => Some("x-icon"),
        [82, 73, 70, 70, _, _, _, _, 87, 69, 66, 80, ..] => Some("webp"),
        [255, 216, 255, b, ..] if *b >= 0xC0 => Some("jpeg"),
        [71, 73, 70, 56, 55 | 57, 97, ..] => Some("gif"),
        [66, 77, _, _, _, _, 0, 0, 0, 0, ..] => Some("bmp"),
        _ => None,
    }
}

async fn download_icon(host: &str) -> Option<Icon> {
    let urls = [
        format!("https://{host}/favicon.ico"),
        format!("https://www.google.com/s2/favicons?domain={host}&sz=64"),
    ];
    for url in urls {
        if let Some(icon) = fetch_image(&url).await {
            return Some(icon);
        }
    }
    None
}

async fn fetch_image(url: &str) -> Option<Icon> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let mut req = Request::new_with_init(url, &init).ok()?;
    if let Ok(headers) = req.headers_mut() {
        let _ = headers.set(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36",
        );
        let _ = headers.set("Accept", "image/*,*/*;q=0.8");
    }

    let mut resp = Fetch::Request(req).send().await.ok()?;
    if !(200..300).contains(&resp.status_code()) {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.len() < MIN_ICON_BYTES || bytes.len() > MAX_ICON_BYTES {
        return None;
    }
    let kind = icon_type(&bytes)?;
    Some(Icon {
        bytes,
        kind,
        miss: false,
    })
}

async fn load_cached(env: &Env, host: &str) -> Option<Icon> {
    let key = storage_key(host);
    match get_storage_backend(env) {
        Some(StorageBackend::R2) => load_cached_r2(env, &key).await,
        Some(StorageBackend::KV) => load_cached_kv(env, &key).await,
        None => None,
    }
}

async fn load_cached_r2(env: &Env, key: &str) -> Option<Icon> {
    let bucket = env.bucket(ATTACHMENTS_BUCKET).ok()?;
    let obj = bucket.get(key).execute().await.ok()??;
    let miss = obj
        .custom_metadata()
        .ok()
        .and_then(|m| m.get("miss").cloned())
        .as_deref()
        == Some("1");
    let ttl = if miss { NEG_TTL_SECS } else { CACHE_TTL_SECS };
    let age_secs = worker::Date::now()
        .as_millis()
        .saturating_sub(obj.uploaded().as_millis())
        / 1000;
    if ttl > 0 && age_secs >= ttl {
        return None;
    }
    let bytes = obj.body()?.bytes().await.ok()?;
    if bytes.is_empty() {
        return Some(fallback());
    }
    let kind = icon_type(&bytes).unwrap_or("png");
    Some(Icon { bytes, kind, miss })
}

#[derive(Serialize, Deserialize)]
struct IconKvMeta {
    content_type: String,
    #[serde(default)]
    miss: bool,
}

async fn load_cached_kv(env: &Env, key: &str) -> Option<Icon> {
    let kv = env.kv(ATTACHMENTS_KV).ok()?;
    let (bytes, meta) = kv.get(key).bytes_with_metadata::<IconKvMeta>().await.ok()?;
    let bytes = bytes?;
    let miss = meta.map(|m| m.miss).unwrap_or(false);
    if bytes.is_empty() {
        return Some(fallback());
    }
    let kind = icon_type(&bytes).unwrap_or("png");
    Some(Icon { bytes, kind, miss })
}

async fn store_cached(env: &Env, host: &str, icon: &Icon) {
    let key = storage_key(host);
    let ttl = if icon.miss {
        NEG_TTL_SECS
    } else {
        CACHE_TTL_SECS
    };
    match get_storage_backend(env) {
        Some(StorageBackend::R2) => {
            let Ok(bucket) = env.bucket(ATTACHMENTS_BUCKET) else {
                return;
            };
            let mut custom = HashMap::new();
            if icon.miss {
                custom.insert("miss".to_string(), "1".to_string());
            }
            if let Err(e) = bucket
                .put(&key, icon.bytes.clone())
                .http_metadata(HttpMetadata {
                    content_type: Some(format!("image/{}", icon.kind)),
                    cache_control: Some(format!("public, max-age={ttl}")),
                    ..Default::default()
                })
                .custom_metadata(custom)
                .execute()
                .await
            {
                log::warn!("Failed to cache icon for {host} in R2: {e}");
            }
        }
        Some(StorageBackend::KV) => {
            let Ok(kv) = env.kv(ATTACHMENTS_KV) else {
                return;
            };
            let meta = IconKvMeta {
                content_type: format!("image/{}", icon.kind),
                miss: icon.miss,
            };
            match kv
                .put_bytes(&key, &icon.bytes)
                .and_then(|b| b.metadata(meta))
            {
                Ok(builder) => {
                    if let Err(e) = builder.expiration_ttl(ttl).execute().await {
                        log::warn!("Failed to cache icon for {host} in KV: {e}");
                    }
                }
                Err(e) => log::warn!("Failed to cache icon for {host} in KV: {e}"),
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_host_accepts_fqdn() {
        assert_eq!(normalize_host("GitHub.COM"), Some("github.com".to_string()));
        assert_eq!(
            normalize_host("sub.example.co.uk."),
            Some("sub.example.co.uk".to_string())
        );
    }

    #[test]
    fn normalize_host_rejects_ssrf_and_junk() {
        assert_eq!(normalize_host("localhost"), None);
        assert_eq!(normalize_host("foo.local"), None);
        assert_eq!(normalize_host("127.0.0.1"), None);
        assert_eq!(normalize_host("::1"), None);
        assert_eq!(normalize_host("[::1]"), None);
        assert_eq!(normalize_host("nodot"), None);
        assert_eq!(normalize_host("foo/bar.com"), None);
        assert_eq!(normalize_host("foo.com:443"), None);
        assert_eq!(normalize_host(""), None);
        assert_eq!(normalize_host("-bad.com"), None);
    }

    #[test]
    fn icon_type_detects_common_formats() {
        assert_eq!(
            icon_type(&[137, 80, 78, 71, 13, 10, 26, 10, 0, 0]),
            Some("png")
        );
        assert_eq!(icon_type(&[0, 0, 1, 0, 1, 0, 0, 0]), Some("x-icon"));
        assert_eq!(icon_type(&[255, 216, 255, 0xE0, 0, 0]), Some("jpeg"));
        assert_eq!(icon_type(&[0, 1, 2, 3]), None);
        assert_eq!(icon_type(&[]), None);
        assert_eq!(icon_type(FALLBACK_ICON), Some("png"));
    }
}
