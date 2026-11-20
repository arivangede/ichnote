//! Serving attachment files to the webview through the `ntc` URI scheme.
//!
//! Notes store `notetice-attachment:<id>`; the frontend turns that into the platform URL for this
//! scheme (`ntc://localhost/<id>` on macOS/Linux, `http://ntc.localhost/<id>` on Windows).
//! Range requests are supported so audio and video can play and seek.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext};
use uuid::Uuid;

use crate::state::AppState;

/// URI scheme name; must match `ATTACHMENT_PROTOCOL` in `src/lib/attachments.ts`.
pub const SCHEME: &str = "ntc";

/// Decodes a `%XX`-encoded string (as produced by `encodeURIComponent`).
pub fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Parses a single `bytes=start-end` range against a file of `len` bytes (inclusive end).
fn parse_range(value: &str, len: u64) -> Option<(u64, u64)> {
    let spec = value.strip_prefix("bytes=")?;
    if spec.contains(',') || len == 0 {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    let (start, end) = match (start.trim(), end.trim()) {
        ("", suffix) => {
            let n: u64 = suffix.parse().ok()?;
            (len.saturating_sub(n), len - 1)
        }
        (s, "") => (s.parse().ok()?, len - 1),
        (s, e) => (s.parse().ok()?, e.parse::<u64>().ok()?.min(len - 1)),
    };
    (start <= end && start < len).then_some((start, end))
}

fn respond(status: StatusCode) -> Response<Vec<u8>> {
    let mut res = Response::new(Vec::new());
    *res.status_mut() = status;
    res
}

fn serve(state: &AppState, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let id = request.uri().path().trim_start_matches('/');
    let Ok(id) = Uuid::parse_str(id) else {
        return respond(StatusCode::BAD_REQUEST);
    };
    let Ok((meta, path)) = state.store().attachment(id) else {
        return respond(StatusCode::NOT_FOUND);
    };
    let Ok(mut file) = File::open(&path) else {
        return respond(StatusCode::NOT_FOUND);
    };
    let len = meta.size;
    let range = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|v| parse_range(v, len));
    let (status, start, end) = match range {
        None => (StatusCode::OK, 0, len.saturating_sub(1)),
        Some(Some((s, e))) => (StatusCode::PARTIAL_CONTENT, s, e),
        Some(None) => {
            let mut res = respond(StatusCode::RANGE_NOT_SATISFIABLE);
            if let Ok(v) = format!("bytes */{len}").parse() {
                res.headers_mut().insert(header::CONTENT_RANGE, v);
            }
            return res;
        }
    };
    let count = if len == 0 { 0 } else { end - start + 1 };
    let mut body = vec![0; usize::try_from(count).unwrap_or(0)];
    if file.seek(SeekFrom::Start(start)).is_err() || file.read_exact(&mut body).is_err() {
        return respond(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut res = Response::new(body);
    *res.status_mut() = status;
    let headers = res.headers_mut();
    if let Ok(v) = meta.mime.parse() {
        headers.insert(header::CONTENT_TYPE, v);
    }
    headers.insert(
        header::ACCEPT_RANGES,
        header::HeaderValue::from_static("bytes"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("private, max-age=31536000, immutable"),
    );
    if status == StatusCode::PARTIAL_CONTENT
        && let Ok(v) = format!("bytes {start}-{end}/{len}").parse()
    {
        headers.insert(header::CONTENT_RANGE, v);
    }
    res
}

/// Protocol handler registered for [`SCHEME`].
pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    match ctx.app_handle().try_state::<AppState>() {
        Some(state) => serve(&state, &request),
        None => respond(StatusCode::SERVICE_UNAVAILABLE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decoding() {
        assert_eq!(
            percent_decode("caf%C3%A9%20menu.png").as_deref(),
            Some("café menu.png")
        );
        assert_eq!(percent_decode("plain").as_deref(), Some("plain"));
        assert_eq!(percent_decode("bad%2"), None);
        assert_eq!(percent_decode("%FF"), None);
    }

    #[test]
    fn ranges() {
        assert_eq!(parse_range("bytes=0-9", 100), Some((0, 9)));
        assert_eq!(parse_range("bytes=90-", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=50-500", 100), Some((50, 99)));
        assert_eq!(parse_range("bytes=100-", 100), None);
        assert_eq!(parse_range("bytes=0-1,4-5", 100), None);
        assert_eq!(parse_range("items=0-1", 100), None);
    }

    #[test]
    fn serves_full_and_partial_content() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::open(dir.path()).unwrap();
        let ws = state.store().list_workspaces().unwrap()[0].id;
        let a = state
            .store()
            .save_attachment(ws, "clip.mp4", "video/mp4", b"0123456789")
            .unwrap();
        let get = |range: Option<&str>| {
            let mut b = Request::builder().uri(format!("ntc://localhost/{}", a.id));
            if let Some(r) = range {
                b = b.header(header::RANGE, r);
            }
            serve(&state, &b.body(Vec::new()).unwrap())
        };
        let full = get(None);
        assert_eq!(full.status(), StatusCode::OK);
        assert_eq!(full.body(), b"0123456789");
        assert_eq!(full.headers()[header::CONTENT_TYPE], "video/mp4");
        let part = get(Some("bytes=2-4"));
        assert_eq!(part.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(part.body(), b"234");
        assert_eq!(part.headers()[header::CONTENT_RANGE], "bytes 2-4/10");
        assert_eq!(
            get(Some("bytes=20-")).status(),
            StatusCode::RANGE_NOT_SATISFIABLE
        );

        let missing = Request::builder()
            .uri(format!("ntc://localhost/{}", Uuid::now_v7()))
            .body(Vec::new())
            .unwrap();
        assert_eq!(serve(&state, &missing).status(), StatusCode::NOT_FOUND);
    }
}
