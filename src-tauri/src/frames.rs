//! `zapframe:` serves the call's latest video frames to the page
//! (REQ-WEBUI-010.A1): the peer's (`remote`) and our own camera's
//! (`local`), as JPEG. The core keeps capturing and decoding video, as
//! egui's call screen does; the page only shows the frames it is handed.
//! Frames live in memory only and are never written to disk or logged.

use std::sync::{Arc, Mutex};

use egui::ColorImage;
use tauri::http::{Request, Response, StatusCode};

#[derive(Default)]
struct Slot {
    frame: Option<Arc<ColorImage>>,
    serial: u64,
}

static REMOTE: Mutex<Slot> = Mutex::new(Slot {
    frame: None,
    serial: 0,
});
static LOCAL: Mutex<Slot> = Mutex::new(Slot {
    frame: None,
    serial: 0,
});

fn slot(name: &str) -> Option<&'static Mutex<Slot>> {
    match name {
        "remote" => Some(&REMOTE),
        "local" => Some(&LOCAL),
        _ => None,
    }
}

/// Keeps `frame` as the latest `name` frame and returns the address the
/// page loads it from, which changes with every new frame so the page
/// fetches it again; `None` when there is no frame.
pub fn publish(name: &'static str, frame: Option<Arc<ColorImage>>) -> Option<String> {
    let mut slot = slot(name)?.lock().ok()?;
    let changed = match (&slot.frame, &frame) {
        (Some(old), Some(new)) => !Arc::ptr_eq(old, new),
        (None, None) => false,
        _ => true,
    };
    if changed {
        slot.serial = slot.serial.wrapping_add(1);
        slot.frame = frame;
    }
    slot.frame
        .as_ref()
        .map(|_| format!("http://zapframe.localhost/{name}?n={}", slot.serial))
}

/// RGB JPEG of a straight-alpha frame, as the image crate writes it.
fn jpeg(frame: &ColorImage) -> Option<Vec<u8>> {
    let [width, height] = frame.size;
    let rgb: Vec<u8> = frame
        .pixels
        .iter()
        .flat_map(|pixel| [pixel.r(), pixel.g(), pixel.b()])
        .collect();
    let buffer = image::RgbImage::from_raw(width as u32, height as u32, rgb)?;
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(buffer)
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .ok()?;
    Some(out.into_inner())
}

/// The JPEG behind a `zapframe` address, for a dumped state
/// (`--dump-state`, REQ-WEBUI-018).
pub fn inline(address: &str) -> Option<Vec<u8>> {
    let rest = address.strip_prefix("http://zapframe.localhost/")?;
    let name = rest.split('?').next()?;
    let frame = slot(name)?.lock().ok()?.frame.clone()?;
    jpeg(&frame)
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
    let Some(slot) = slot(request.uri().path().trim_start_matches('/')) else {
        return refuse(StatusCode::NOT_FOUND);
    };
    let frame = slot.lock().ok().and_then(|slot| slot.frame.clone());
    match frame.as_deref().and_then(jpeg) {
        Some(bytes) => Response::builder()
            .header("Content-Type", "image/jpeg")
            .header("Cache-Control", "no-store")
            .body(bytes)
            .expect("response"),
        None => refuse(StatusCode::NOT_FOUND),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_frame_gets_a_new_address() {
        let one = Arc::new(ColorImage::filled([4, 3], egui::Color32::RED));
        let first = publish("remote", Some(one.clone())).expect("address");
        assert_eq!(
            publish("remote", Some(one)).as_deref(),
            Some(first.as_str())
        );
        let two = Arc::new(ColorImage::filled([4, 3], egui::Color32::BLUE));
        let second = publish("remote", Some(two.clone())).expect("address");
        assert_ne!(first, second);
        assert!(jpeg(&two).is_some_and(|bytes| bytes.starts_with(&[0xFF, 0xD8])));
        assert_eq!(publish("remote", None), None);
    }
}
