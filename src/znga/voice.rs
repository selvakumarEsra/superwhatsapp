//! Voice notes in short (REQ-ZNGA-023): transcription through an
//! OpenAI-compatible `/audio/transcriptions` endpoint. The default is a
//! server on this computer (for example whisper.cpp's server started with
//! `--inference-path /v1/audio/transcriptions`), so nothing leaves it unless
//! the person points it at a cloud service.

use std::path::Path;
use std::time::Duration;

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8080/v1/audio/transcriptions";
pub const DEFAULT_MODEL: &str = "whisper-1";

/// A multipart/form-data body with the model name and the audio file
/// (REQ-ZNGA-023.A1). Returns (content type, body).
pub fn multipart(
    model: &str,
    file_name: &str,
    mime: &str,
    audio: &[u8],
    boundary: &str,
) -> (String, Vec<u8>) {
    let mut body = Vec::with_capacity(audio.len() + 512);
    let file_name: String = file_name
        .chars()
        .filter(|c| !matches!(c, '"' | '\r' | '\n'))
        .collect();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\n{model}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"response_format\"\r\n\r\njson\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: {mime}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(audio);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// The transcript in an OpenAI-style answer (`{"text": "…"}`).
pub fn parse_transcript(status: u16, body: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| format!("The transcription service answered with HTTP {status}."))?;
    if !(200..300).contains(&status) {
        let message = value["error"]["message"]
            .as_str()
            .or_else(|| value["error"].as_str())
            .unwrap_or("it returned an error");
        return Err(format!("Transcription failed: {message}"));
    }
    let text = value["text"].as_str().unwrap_or_default().trim().to_owned();
    if text.is_empty() {
        Err("The voice note had no words Znga could hear.".into())
    } else {
        Ok(text)
    }
}

/// Whether `endpoint` is on this computer.
pub fn is_local(endpoint: &str) -> bool {
    let rest = endpoint
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host = rest.split(['/', ':']).next().unwrap_or_default();
    matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1")
}

/// Transcribes `path`. Blocking: run on its own thread.
pub fn transcribe(
    endpoint: &str,
    model: &str,
    key: Option<&str>,
    path: &Path,
    mime: &str,
) -> Result<String, String> {
    let audio =
        std::fs::read(path).map_err(|_| "The voice note's file could not be read.".to_owned())?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("voice.ogg");
    let boundary = format!("znga-{:x}", crate::util::now());
    let (content_type, body) = multipart(model, name, mime, &audio, &boundary);
    let local = is_local(endpoint);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(300)))
        .http_status_as_error(false)
        .proxy(if local {
            None
        } else {
            crate::proxy::ureq_proxy()
        })
        .build()
        .into();
    let mut call = agent.post(endpoint).header("content-type", &content_type);
    if let Some(key) = key.filter(|key| !key.is_empty()) {
        call = call.header("authorization", &format!("Bearer {key}"));
    }
    let mut response = call.send(&body[..]).map_err(|error| {
        if local {
            format!(
                "Could not reach the transcription server at {endpoint} ({error}). Is it running?"
            )
        } else {
            format!("Could not reach the transcription service ({error}).")
        }
    })?;
    let status = response.status().as_u16();
    let text = response
        .body_mut()
        .read_to_string()
        .map_err(|_| "The transcription could not be read.".to_owned())?;
    parse_transcript(status, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_upload_is_multipart_with_the_model_and_the_file() {
        let (content_type, body) =
            multipart("whisper-1", "note\".ogg", "audio/ogg", b"OGGDATA", "b0");
        assert_eq!(content_type, "multipart/form-data; boundary=b0");
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("name=\"model\"\r\n\r\nwhisper-1\r\n"));
        assert!(text.contains(
            "name=\"file\"; filename=\"note.ogg\"\r\nContent-Type: audio/ogg\r\n\r\nOGGDATA\r\n"
        ));
        assert!(text.ends_with("--b0--\r\n"));
    }

    #[test]
    fn transcripts_and_errors_are_read() {
        assert_eq!(
            parse_transcript(200, r#"{"text":" hi there "}"#).unwrap(),
            "hi there"
        );
        assert!(
            parse_transcript(401, r#"{"error":{"message":"bad key"}}"#)
                .unwrap_err()
                .contains("bad key")
        );
        assert!(parse_transcript(200, r#"{"text":""}"#).is_err());
        assert!(is_local(DEFAULT_ENDPOINT));
        assert!(!is_local("https://api.openai.com/v1/audio/transcriptions"));
    }
}
