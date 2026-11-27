//! `zapmedia:` serves avatars and media files to the page, only from
//! SuperWhatsapp's own cache folders (REQ-WEBUI-003.A2).

use std::path::{Path, PathBuf};

use tauri::http::{Request, Response, StatusCode};

fn decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The folders media may come from: SuperWhatsapp's cache and the temporary
/// folder demo runs use.
fn allowed(path: &Path) -> bool {
    let Ok(path) = path.canonicalize() else {
        return false;
    };
    let roots: Vec<PathBuf> = [
        Some(superwhatsapp::paths::AppDirs::discover().cache),
        Some(std::env::temp_dir()),
    ]
    .into_iter()
    .flatten()
    .filter_map(|root| root.canonicalize().ok())
    .collect();
    // The chat background image Settings imported (wallpaper.<ext>).
    let wallpaper = path.parent().is_some_and(|parent| {
        superwhatsapp::paths::AppDirs::discover()
            .state
            .canonicalize()
            .is_ok_and(|state| parent == state)
    }) && path.file_stem().is_some_and(|stem| stem == "wallpaper");
    wallpaper || roots.iter().any(|root| path.starts_with(root))
}

fn kind(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    }
}

pub fn serve<R: tauri::Runtime>(
    _ctx: tauri::UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let refuse = |status: StatusCode| {
        Response::builder()
            .status(status)
            .body(Vec::new())
            .expect("response")
    };
    let Some(path) = decode(request.uri().path().trim_start_matches('/')) else {
        return refuse(StatusCode::BAD_REQUEST);
    };
    let path = PathBuf::from(path);
    if !allowed(&path) {
        return refuse(StatusCode::FORBIDDEN);
    }
    match std::fs::read(&path) {
        Ok(bytes) => Response::builder()
            .header("Content-Type", kind(&path))
            .header("Cache-Control", "max-age=3600")
            .body(bytes)
            .expect("response"),
        Err(_) => refuse(StatusCode::NOT_FOUND),
    }
}

/// Replaces every `zapmedia` address in `value` with the file as a `data:`
/// address, so a dumped state shows its pictures in a plain browser
/// (`--dump-state`, REQ-WEBUI-018). Files that are not allowed stay as they are.
pub fn inline(value: &mut serde_json::Value) {
    use base64::Engine as _;
    match value {
        serde_json::Value::String(text) => {
            if let Some(bytes) = crate::frames::inline(text) {
                let data = base64::engine::general_purpose::STANDARD.encode(bytes);
                *text = format!("data:image/jpeg;base64,{data}");
                return;
            }
            let Some(rest) = text.strip_prefix("http://zapmedia.localhost/") else {
                return;
            };
            let Some(path) = decode(rest).map(PathBuf::from) else {
                return;
            };
            if !allowed(&path) {
                return;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                let data = base64::engine::general_purpose::STANDARD.encode(bytes);
                *text = format!("data:{};base64,{data}", kind(&path));
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(inline),
        serde_json::Value::Object(map) => map.values_mut().for_each(inline),
        _ => {}
    }
}
