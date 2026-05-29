//! BrightSDK download library — resolve versions, fetch and extract SDK archives.
//!
//! Exposes both a Rust API and a C FFI surface for consumption from C#, Java, etc.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn make_agent() -> ureq::Agent {
    let tls = native_tls::TlsConnector::new().expect("TLS init failed");
    ureq::AgentBuilder::new()
        .tls_connector(Arc::new(tls))
        .build()
}

mod extract;

const RELEASES_URL: &str = "https://bright-sdk.com/sdk_api/sdk/integration/config";

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub last_version: Option<String>,
    pub url: Option<String>,
    pub url_tpl: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasesConfig {
    pub platforms: HashMap<String, PlatformConfig>,
    #[serde(default)]
    pub templates: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResult {
    pub platform: String,
    pub version: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub platform: String,
    pub version: String,
    pub url: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub key: String,
    pub last_version: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    MissingApiKey,
    Http(String),
    Json(String),
    Io(io::Error),
    UnknownPlatform(String),
    NoVersion(String),
    NoUrl(String),
    Extract(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingApiKey => write!(f, "SDK_API_KEY environment variable is required"),
            Error::Http(e) => write!(f, "HTTP error: {e}"),
            Error::Json(e) => write!(f, "JSON parse error: {e}"),
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::UnknownPlatform(p) => write!(f, "Unknown platform '{p}'"),
            Error::NoVersion(p) => write!(f, "No latest version for platform '{p}'"),
            Error::NoUrl(p) => write!(f, "Cannot resolve download URL for '{p}'"),
            Error::Extract(e) => write!(f, "Extraction error: {e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Progress step reported during `fetch_sdk_with_progress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Resolve,
    Download,
    Verify,
    Extract,
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Resolve => write!(f, "resolve"),
            Step::Download => write!(f, "download"),
            Step::Verify => write!(f, "verify"),
            Step::Extract => write!(f, "extract"),
        }
    }
}

/// Progress callback: (step, bytes_done, bytes_total or 0 if unknown)
pub type ProgressFn = Box<dyn FnMut(Step, u64, u64)>;

// --- Public API ---

pub fn fetch_releases() -> Result<ReleasesConfig, Error> {
    let api_key = std::env::var("SDK_API_KEY").map_err(|_| Error::MissingApiKey)?;
    let agent = make_agent();
    let resp = agent
        .get(RELEASES_URL)
        .set("api-key", &api_key)
        .set("User-Agent", "bright-sdk-download-rs/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| Error::Http(e.to_string()))?;
    let body = resp.into_string().map_err(|e| Error::Http(e.to_string()))?;
    serde_json::from_str(&body).map_err(|e| Error::Json(e.to_string()))
}

pub fn resolve_sdk(platform_key: &str, version: &str) -> Result<ResolveResult, Error> {
    let releases = fetch_releases()?;
    let platform_data = releases.platforms.get(platform_key).ok_or_else(|| {
        let available: Vec<&str> = releases.platforms.keys().map(|s| s.as_str()).collect();
        Error::UnknownPlatform(format!(
            "{}. Available: {}",
            platform_key,
            available.join(", ")
        ))
    })?;

    let ver = if version == "latest" {
        platform_data
            .last_version
            .clone()
            .ok_or_else(|| Error::NoVersion(platform_key.to_string()))?
    } else {
        version.to_string()
    };

    let url = if let Some(ref u) = platform_data.url {
        u.clone()
    } else {
        resolve_url_tpl(&releases, platform_key, &ver)
            .ok_or_else(|| Error::NoUrl(platform_key.to_string()))?
    };

    Ok(ResolveResult {
        platform: platform_key.to_string(),
        version: ver,
        url,
        sha256: platform_data.sha256.clone(),
    })
}

pub fn fetch_sdk(platform_key: &str, version: &str, output: &str) -> Result<FetchResult, Error> {
    fetch_sdk_with_progress(platform_key, version, output, None)
}

/// Download + extract with optional progress callback.
pub fn fetch_sdk_with_progress(
    platform_key: &str,
    version: &str,
    output: &str,
    mut on_progress: Option<ProgressFn>,
) -> Result<FetchResult, Error> {
    if let Some(ref mut cb) = on_progress {
        cb(Step::Resolve, 0, 0);
    }
    let resolved = resolve_sdk(platform_key, version)?;
    if let Some(ref mut cb) = on_progress {
        cb(Step::Resolve, 1, 1);
    }

    let out_dir = PathBuf::from(output).canonicalize().unwrap_or_else(|_| {
        let p = PathBuf::from(output);
        fs::create_dir_all(&p).ok();
        p.canonicalize().unwrap_or(p)
    });
    fs::create_dir_all(&out_dir)?;

    let url_path = resolved.url.split('?').next().unwrap_or(&resolved.url);
    let ext = if url_path.ends_with(".tar.gz") {
        ".tar.gz"
    } else {
        Path::new(url_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| if e == "zip" { ".zip" } else { ".tar.gz" })
            .unwrap_or(".zip")
    };
    let archive_name = format!("brightsdk-{}-{}{}", platform_key, resolved.version, ext);
    let archive_path = out_dir.join(&archive_name);

    if let Some(ref mut cb) = on_progress {
        cb(Step::Download, 0, 0);
    }
    // Take on_progress out so the download closure can own it temporarily
    let mut dl_cb = on_progress.take();
    download_to_file(&resolved.url, &archive_path, &mut |done, total| {
        if let Some(ref mut cb) = dl_cb {
            cb(Step::Download, done, total);
        }
    })?;
    on_progress = dl_cb; // restore after download

    if let Some(ref expected_sha) = resolved.sha256 {
        if let Some(ref mut cb) = on_progress {
            cb(Step::Verify, 0, 1);
        }
        let actual_sha = sha256_file(&archive_path)?;
        if actual_sha != expected_sha.to_lowercase() {
            fs::remove_file(&archive_path)?;
            return Err(Error::Extract(format!(
                "SHA-256 mismatch: expected {expected_sha}, got {actual_sha}"
            )));
        }
        if let Some(ref mut cb) = on_progress {
            cb(Step::Verify, 1, 1);
        }
    }

    if let Some(ref mut cb) = on_progress {
        cb(Step::Extract, 0, 0);
    }
    extract::extract(&archive_path, &out_dir)?;
    fs::remove_file(&archive_path)?;
    if let Some(ref mut cb) = on_progress {
        cb(Step::Extract, 1, 1);
    }

    Ok(FetchResult {
        platform: resolved.platform,
        version: resolved.version,
        url: resolved.url,
        output: out_dir.to_string_lossy().to_string(),
    })
}

pub fn list_platforms() -> Result<Vec<PlatformInfo>, Error> {
    let releases = fetch_releases()?;
    let mut list: Vec<PlatformInfo> = releases
        .platforms
        .into_iter()
        .map(|(key, val)| PlatformInfo {
            key,
            last_version: val.last_version,
        })
        .collect();
    list.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(list)
}

// --- Internal helpers ---

fn resolve_url_tpl(releases: &ReleasesConfig, platform_key: &str, ver: &str) -> Option<String> {
    let platform = releases.platforms.get(platform_key)?;
    let url_tpl = platform.url_tpl.as_ref()?;
    let mut url = url_tpl.clone();

    let base = releases.templates.get("base").cloned().unwrap_or_default();
    for (key, val) in &releases.templates {
        if key == "base" {
            continue;
        }
        let pattern = format!("{{{{{key}}}}}");
        url = url.replace(&pattern, val);
    }
    url = url.replace("{{base}}", &base);
    url = url.replace("{{platform}}", platform_key);
    url = url.replace("{{version}}", ver);
    Some(url)
}

fn sha256_file(path: &Path) -> Result<String, Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn download_to_file(
    url: &str,
    dest: &Path,
    on_progress: &mut dyn FnMut(u64, u64),
) -> Result<(), Error> {
    let agent = make_agent();
    let resp = agent
        .get(url)
        .set("User-Agent", "bright-sdk-download-rs/0.1")
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| Error::Http(e.to_string()))?;

    let total: u64 = resp
        .header("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut reader = resp.into_reader();
    let mut file = fs::File::create(dest)?;
    let mut buf = [0u8; 65536];
    let mut downloaded: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| Error::Http(e.to_string()))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    Ok(())
}

// --- C FFI ---

/// Resolve SDK version + URL. Returns a JSON string (caller must free with `sdk_free_string`).
/// Returns null on error.
#[no_mangle]
pub extern "C" fn sdk_resolve(platform: *const c_char, version: *const c_char) -> *mut c_char {
    if platform.is_null() || version.is_null() {
        return std::ptr::null_mut();
    }
    let platform = match unsafe { CStr::from_ptr(platform) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let version = match unsafe { CStr::from_ptr(version) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match resolve_sdk(platform, version) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(e) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
    }
}

/// Fetch (download + extract) SDK. Returns JSON string or null on error.
#[no_mangle]
pub extern "C" fn sdk_fetch(
    platform: *const c_char,
    version: *const c_char,
    output_dir: *const c_char,
) -> *mut c_char {
    if platform.is_null() || version.is_null() || output_dir.is_null() {
        return std::ptr::null_mut();
    }
    let platform = match unsafe { CStr::from_ptr(platform) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let version = match unsafe { CStr::from_ptr(version) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let output = match unsafe { CStr::from_ptr(output_dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match fetch_sdk(platform, version, output) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(e) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
    }
}

/// List platforms. Returns JSON string or null.
#[no_mangle]
pub extern "C" fn sdk_list_platforms() -> *mut c_char {
    match list_platforms() {
        Ok(list) => match serde_json::to_string(&list) {
            Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(e) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
    }
}

/// Get last error message. Returns null if no error.
/// Caller must free with `sdk_free_string`.
#[no_mangle]
pub extern "C" fn sdk_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        let e = e.borrow();
        match e.as_ref() {
            Some(msg) => CString::new(msg.as_str()).unwrap_or_default().into_raw(),
            None => std::ptr::null_mut(),
        }
    })
}

/// Free a string returned by sdk_* functions.
#[no_mangle]
pub extern "C" fn sdk_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(err: &Error) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(err.to_string());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_url_tpl_basic() {
        let mut platforms = HashMap::new();
        platforms.insert(
            "android".to_string(),
            PlatformConfig {
                last_version: Some("1.623.17".to_string()),
                url: None,
                url_tpl: Some("{{base}}/{{platform}}/{{version}}/sdk.zip".to_string()),
                sha256: None,
            },
        );
        let mut templates = HashMap::new();
        templates.insert("base".to_string(), "https://cdn.example.com".to_string());

        let releases = ReleasesConfig {
            platforms,
            templates,
        };
        let url = resolve_url_tpl(&releases, "android", "1.623.17");
        assert_eq!(
            url,
            Some("https://cdn.example.com/android/1.623.17/sdk.zip".to_string())
        );
    }

    #[test]
    fn resolve_url_tpl_with_named_templates() {
        let mut platforms = HashMap::new();
        platforms.insert(
            "ios".to_string(),
            PlatformConfig {
                last_version: Some("2.0.0".to_string()),
                url: None,
                url_tpl: Some("{{base}}/{{region}}/{{platform}}-{{version}}.zip".to_string()),
                sha256: None,
            },
        );
        let mut templates = HashMap::new();
        templates.insert("base".to_string(), "https://cdn.example.com".to_string());
        templates.insert("region".to_string(), "us-east".to_string());

        let releases = ReleasesConfig {
            platforms,
            templates,
        };
        let url = resolve_url_tpl(&releases, "ios", "2.0.0");
        assert_eq!(
            url,
            Some("https://cdn.example.com/us-east/ios-2.0.0.zip".to_string())
        );
    }

    #[test]
    fn resolve_url_tpl_missing_platform() {
        let releases = ReleasesConfig {
            platforms: HashMap::new(),
            templates: HashMap::new(),
        };
        assert_eq!(resolve_url_tpl(&releases, "nonexistent", "1.0.0"), None);
    }

    #[test]
    fn resolve_url_tpl_no_template() {
        let mut platforms = HashMap::new();
        platforms.insert(
            "win".to_string(),
            PlatformConfig {
                last_version: Some("1.0.0".to_string()),
                url: None,
                url_tpl: None,
                sha256: None,
            },
        );
        let releases = ReleasesConfig {
            platforms,
            templates: HashMap::new(),
        };
        assert_eq!(resolve_url_tpl(&releases, "win", "1.0.0"), None);
    }

    #[test]
    fn fetch_releases_missing_api_key() {
        std::env::remove_var("SDK_API_KEY");
        let result = fetch_releases();
        assert!(matches!(result, Err(Error::MissingApiKey)));
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            Error::MissingApiKey.to_string(),
            "SDK_API_KEY environment variable is required"
        );
        assert_eq!(
            Error::UnknownPlatform("foo".into()).to_string(),
            "Unknown platform 'foo'"
        );
        assert_eq!(
            Error::Http("timeout".into()).to_string(),
            "HTTP error: timeout"
        );
    }

    #[test]
    fn ffi_null_pointer_safety() {
        let result = sdk_resolve(std::ptr::null(), std::ptr::null());
        assert!(result.is_null());

        let result = sdk_fetch(std::ptr::null(), std::ptr::null(), std::ptr::null());
        assert!(result.is_null());
    }

    #[test]
    fn ffi_free_null_is_safe() {
        sdk_free_string(std::ptr::null_mut()); // should not crash
    }
}
