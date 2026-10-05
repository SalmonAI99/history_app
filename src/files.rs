// File metadata, protected upload/download handlers, and slide-media listings.

use crate::auth::User;
use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

// ----------------------------------------------------------
//  SHARED TYPE — one file, as the browser may see it.
//  (stored_name deliberately NOT here: never leak the disk layout.)
// ----------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FileMeta {
    pub id: i64,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

impl FileMeta {
    pub fn is_video(&self) -> bool {
        self.mime_type.starts_with("video/")
    }
    pub fn href(&self) -> String {
        format!("/files?id={}", self.id)
    }
    pub fn size_label(&self) -> String {
        format!("{:.1} MB", self.size_bytes as f64 / 1_048_576.0)
    }
}

/// Metadata for an interactive PPTX slide deck generated in `public/slides`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlideDeckMeta {
    pub id: String,
    pub title: String,
    pub slide_count: usize,
    pub url: String,
}

// ----------------------------------------------------------
//  SERVER FUNCTIONS
// ----------------------------------------------------------

/// Scans the `public/slides` folder for generated slide lecture notes.
#[server]
pub async fn list_slide_decks() -> Result<Vec<SlideDeckMeta>, ServerFnError> {
    let mut decks = Vec::new();
    let slides_dir = std::path::Path::new("public/slides");
    if !tokio::fs::try_exists(slides_dir).await.unwrap_or(false) {
        return Ok(decks);
    }

    let Ok(mut entries) = tokio::fs::read_dir(slides_dir).await else {
        return Ok(decks);
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let file_type = match entry.file_type().await {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !file_type.is_dir() {
            continue;
        }

        let deck_id = entry.file_name().to_string_lossy().to_string();
        let deck_dir = entry.path();
        let lecture_json_path = deck_dir.join("lecture.json");

        if !tokio::fs::try_exists(&lecture_json_path).await.unwrap_or(false) {
            continue;
        }

        let mut title = deck_id.replace('_', " ");
        let mut slide_count = 0;

        if let Ok(content) = tokio::fs::read_to_string(&lecture_json_path).await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(t) = val.get("title").and_then(|v| v.as_str()) {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        title = trimmed.to_string();
                    }
                }
                if let Some(slides) = val.get("slides").and_then(|v| v.as_array()) {
                    slide_count = slides.len();
                }
            }
        }

        decks.push(SlideDeckMeta {
            id: deck_id.clone(),
            title,
            slide_count,
            url: format!("/slides/{}", deck_id),
        });
    }

    decks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(decks)
}

/// Every file in the library (materials AND videos), newest first.
/// Any logged-in user may browse the catalogue; downloading is
/// gated again inside the /files route itself.
#[server]
pub async fn list_files() -> Result<Vec<FileMeta>, ServerFnError> {
    crate::auth::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;
    let files = sqlx::query_as::<_, FileMeta>(
        "SELECT id, original_name, mime_type, size_bytes, created_at
         FROM files ORDER BY id DESC",
    )
    .fetch_all(&mut conn)
    .await?;
    Ok(files)
}

/// Delete a file: its database row AND its bytes on disk. Admin only.
#[server]
pub async fn delete_file(id: i64) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    if user.role != "admin" {
        return Err(ServerFnError::ServerError("Admins only.".into()));
    }

    let mut conn = crate::db::db().await?;
    let row: Option<(String,)> =
        sqlx::query_as("SELECT stored_name FROM files WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut conn)
            .await?;
    let Some((stored,)) = row else {
        return Err(ServerFnError::ServerError("No such file.".into()));
    };

    sqlx::query("DELETE FROM files WHERE id = ?")
        .bind(id)
        .execute(&mut conn)
        .await?;

    // Delete the bytes too — basename ONLY, so a weird stored value
    // can never escape the storage folder.
    if let Some(name) = std::path::Path::new(&stored).file_name() {
        let _ = tokio::fs::remove_file(
            std::path::Path::new(&ssr::upload_dir()).join(name),
        )
        .await;
    }
    Ok(())
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use axum::body::Body;
    use axum::extract::{Multipart, Query};
    use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
    use axum::response::{IntoResponse, Redirect, Response};
    use serde::Deserialize;
    use std::path::{Path, PathBuf};
    use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

    #[derive(Deserialize)]
    pub struct FileQuery {
        pub id: i64,
    }

    /// Where uploads live. Tests override this with UPLOAD_DIR.
    pub fn upload_dir() -> String {
        std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "storage".into())
    }

    /// Pull the session token out of the Cookie header.
    /// (We can't reuse auth::ssr::session_token — it reads Leptos
    /// request context, which doesn't exist in a raw axum route.)
    fn session_token(headers: &HeaderMap) -> Option<String> {
        let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
        cookie
            .split(';')
            .map(str::trim)
            .find_map(|c| c.strip_prefix("session="))
            .map(str::to_string)
    }

    /// Session token -> (user_id, role). None = not logged in.
    async fn user_from_headers(headers: &HeaderMap) -> Option<(i64, String)> {
        let token = session_token(headers)?;
        let mut conn = crate::db::db().await.ok()?;
        sqlx::query_as::<_, (i64, String)>(
            "SELECT u.id, u.role FROM users u
             JOIN sessions s ON s.user_id = u.id
             WHERE s.token = ?",
        )
        .bind(token)
        .fetch_optional(&mut conn)
        .await
        .ok()
        .flatten()
    }

    /// Map a file extension to its MIME type — this is ALSO the
    /// upload whitelist: extensions not listed here are refused.
    fn mime_from_ext(ext: &str) -> Option<&'static str> {
        Some(match ext {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "pdf" => "application/pdf",
            "txt" => "text/plain; charset=utf-8",
            "md" => "text/markdown; charset=utf-8",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "mp3" => "audio/mpeg",
            _ => return None,
        })
    }

    /// A filename safe for a Content-Disposition header:
    /// keep ASCII letters/digits/dot/dash, map everything else to '_'.
    fn safe_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Parse "Range: bytes=start-end" against a file of `size` bytes.
    /// Ok(None)      = no usable range header -> serve the whole file
    /// Ok(Some(s,e)) = serve bytes s..=e   -> 206 Partial Content
    /// Err(())       = asked for something impossible -> 416
    fn parse_range(header: Option<&str>, size: i64) -> Result<Option<(i64, i64)>, ()> {
        let Some(h) = header else { return Ok(None) };
        let Some(h) = h.strip_prefix("bytes=") else { return Ok(None) };
        let Some((a, b)) = h.split_once('-') else { return Ok(None) };
        let Ok(start) = a.parse::<i64>() else {
            return Ok(None); // "bytes=-500" (suffix) — we simply serve whole
        };
        let end = if b.is_empty() {
            size - 1
        } else {
            b.parse::<i64>().map_err(|_| ())?.min(size - 1)
        };
        if start > end || start >= size {
            return Err(());
        }
        Ok(Some((start, end)))
    }

    fn err(status: StatusCode, msg: &'static str) -> Response {
        (status, msg).into_response()
    }

    /// GET /files?id=N — the ONLY door to stored bytes.
    /// Order of operations matters: session check FIRST, disk LAST.
    pub async fn serve_file(
        Query(q): Query<FileQuery>,
        headers: HeaderMap,
    ) -> Result<Response, Response> {
        // 1. Who are you? No session, no bytes.
        if user_from_headers(&headers).await.is_none() {
            return Err(err(StatusCode::UNAUTHORIZED, "log in first"));
        }

        // 2. Which file? Look it up in the database.
        let mut conn = crate::db::db()
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT stored_name, original_name, mime_type FROM files WHERE id = ?",
        )
        .bind(q.id)
        .fetch_optional(&mut conn)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;
        let Some((stored, original, mime)) = row else {
            return Err(err(StatusCode::NOT_FOUND, "no such file"));
        };

        // 3. Path safety: basename ONLY, never a stored path verbatim.
        let name = Path::new(&stored)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let path = PathBuf::from(upload_dir()).join(name);

        // 4. Stream it — with Range support so <video> can seek.
        stream_path(&path, &mime, &original, headers.get(header::RANGE)).await
    }

    /// Open `path`, honor Range, and stream the requested bytes.
    async fn stream_path(
        path: &Path,
        mime: &str,
        original: &str,
        range_hdr: Option<&HeaderValue>,
    ) -> Result<Response, Response> {
        let range_str = range_hdr.and_then(|v| v.to_str().ok());

        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|_| err(StatusCode::NOT_FOUND, "file not on disk"))?;
        let size = file
            .metadata()
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "stat failed"))?
            .len() as i64;

        let range = match parse_range(range_str, size) {
            Ok(r) => r,
            Err(()) => {
                return Err(Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_RANGE, format!("bytes */{size}"))
                    .body(Body::empty())
                    .unwrap())
            }
        };

        let (status, start, end) = match range {
            None => (StatusCode::OK, 0, size - 1),
            Some((s, e)) => (StatusCode::PARTIAL_CONTENT, s, e),
        };
        let len = (end - start + 1) as u64;

        // Seek to the first wanted byte, then cap the stream at `len`.
        file.seek(std::io::SeekFrom::Start(start as u64))
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "seek failed"))?;
        let reader = file.take(len);
        let body = Body::from_stream(tokio_util::io::ReaderStream::new(reader));

        let mut resp = Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_LENGTH, len)
            .header(header::ACCEPT_RANGES, "bytes");
        if status == StatusCode::PARTIAL_CONTENT {
            resp = resp.header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{size}"),
            );
        }
        // Materials download with their original name; videos stay inline
        // so the <video> tag can play them.
        if !mime.starts_with("video/") {
            resp = resp.header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", safe_name(original)),
            );
        }
        resp.body(body)
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "response build failed"))
    }

    /// POST /upload (multipart/form-data) — admins only.
    /// Reads the form field "file", whitelists the extension,
    /// stores bytes on disk, then records the metadata row.
    pub async fn upload(headers: HeaderMap, mut multipart: Multipart) -> Response {
        // Admin gate — same rule as delete_file, just in raw axum.
        let Some((uid, role)) = user_from_headers(&headers).await else {
            return err(StatusCode::UNAUTHORIZED, "log in first");
        };
        if role != "admin" {
            return err(StatusCode::FORBIDDEN, "admins only");
        }

        let mut back = "/materials".to_string();
        let mut saved: Option<i64> = None;

        loop {
            let field = match multipart.next_field().await {
                Ok(Some(f)) => f,
                Ok(None) => break,          // end of the form
                Err(_) => return err(StatusCode::BAD_REQUEST, "bad upload"),
            };
            match field.name() {
                Some("back") => {
                    if let Ok(text) = field.text().await {
                        // Redirect target must be a local path.
                        if text.starts_with('/') {
                            back = text;
                        }
                    }
                }
                Some("file") => {
                    match save_upload(uid, field).await {
                        Ok(id) => saved = Some(id),
                        Err(response) => return response,
                    }
                }
                _ => { /* ignore other fields */ }
            }
        }

        match saved {
            Some(_) => Redirect::to(&back).into_response(),
            None => err(StatusCode::BAD_REQUEST, "no file in the form"),
        }
    }

    /// Stream one multipart file field to disk, then insert the row.
    /// Returns the new file id, or a ready-made error Response.
    async fn save_upload(
        uid: i64,
        mut field: axum::extract::multipart::Field<'_>,
    ) -> Result<i64, Response> {
        // 1. Sanitize + whitelist the extension.
        let original = field
            .file_name()
            .and_then(|n| Path::new(n).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string();
        let ext = original
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let Some(mime) = mime_from_ext(&ext) else {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "that file type is not allowed",
            ));
        };

        // 2. Insert the row first — its id becomes the on-disk name
        //    ("17.mp4"), which can't collide and can't be forged.
        let mut conn = crate::db::db()
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;
        let id = sqlx::query(
            "INSERT INTO files (user_id, original_name, stored_name, mime_type, size_bytes)
             VALUES (?, ?, '', ?, 0)",
        )
        .bind(uid)
        .bind(&original)
        .bind(mime)
        .execute(&mut conn)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
        .last_insert_rowid();

        let stored = format!("{id}.{ext}");
        sqlx::query("UPDATE files SET stored_name = ? WHERE id = ?")
            .bind(&stored)
            .bind(id)
            .execute(&mut conn)
            .await
            .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

        // 3. Stream the bytes to disk, counting them as they pass.
        let path = PathBuf::from(upload_dir()).join(&stored);
        let mut out = match tokio::fs::File::create(&path).await {
            Ok(f) => f,
            Err(_) => return Err(err(StatusCode::INTERNAL_SERVER_ERROR, "write failed")),
        };
        let mut size: i64 = 0;
        loop {
            match field.chunk().await {
                Ok(Some(chunk)) => {
                    if out.write_all(&chunk).await.is_err() {
                        return Err(err(StatusCode::INTERNAL_SERVER_ERROR, "write failed"));
                    }
                    size += chunk.len() as i64;
                }
                Ok(None) => break,
                Err(_) => return Err(err(StatusCode::BAD_REQUEST, "upload interrupted")),
            }
        }
        if let Err(e) = out.flush().await {
            let _ = e;
        }

        // 4. Record the real size.
        let _ = sqlx::query("UPDATE files SET size_bytes = ? WHERE id = ?")
            .bind(size)
            .bind(id)
            .execute(&mut conn)
            .await;

        tracing::info!(
            event = "upload",
            file_id = id,
            user_id = uid,
            bytes = size,
            name = %original,
            "file uploaded"
        );
        Ok(id)
    }

    #[cfg(all(test, feature = "ssr"))]
    mod tests {
        use super::*;

        #[test]
        fn no_range_means_whole_file() {
            assert_eq!(parse_range(None, 1000), Ok(None));
            assert_eq!(parse_range(Some("items=0-9"), 1000), Ok(None));
        }

        #[test]
        fn open_ended_range_goes_to_eof() {
            assert_eq!(parse_range(Some("bytes=10-"), 1000), Ok(Some((10, 999))));
        }

        #[test]
        fn end_is_clamped_to_file_size() {
            assert_eq!(parse_range(Some("bytes=0-9999"), 1000), Ok(Some((0, 999))));
        }

        #[test]
        fn past_the_end_is_416() {
            assert_eq!(parse_range(Some("bytes=1000-2000"), 1000), Err(()));
            assert_eq!(parse_range(Some("bytes=5-2"), 1000), Err(()));
        }

        #[test]
        fn suffix_ranges_are_ignored() {
            // "bytes=-500" asks for the LAST 500 bytes; we don't support
            // it, so we serve the whole file instead of breaking.
            assert_eq!(parse_range(Some("bytes=-500"), 1000), Ok(None));
        }

        #[test]
        fn safe_name_replaces_non_ascii() {
            assert_eq!(safe_name("lesson 1 (final).mp4"), "lesson_1__final_.mp4");
            assert_eq!(safe_name("课件.pdf"), "__.pdf");
        }

        #[test]
        fn mime_whitelist() {
            assert_eq!(mime_from_ext("mp4"), Some("video/mp4"));
            assert_eq!(mime_from_ext("jpg"), Some("image/jpeg"));
            assert_eq!(mime_from_ext("exe"), None);
        }
    }
}
