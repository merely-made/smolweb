//! The Titan protocol (`titan://`, port 1965, <https://transjovian.org/titan>).
//!
//! Titan is the upload/write companion to Gemini. The client opens a TLS
//! connection (same TOFU verifier as Gemini), sends a request line of the form
//!
//!   `<titan-url>;size=<n>[;mime=<type>][;token=<token>]\r\n`
//!
//! immediately followed by `<n>` bytes of body, then reads a Gemini-format
//! response header (`<code> <meta>\r\n`) and optional body.
//!
//! ## Navigation (`fetch`)
//!
//! When the host navigates to a `titan://` URL without a payload (e.g. the user
//! clicks a titan:// link), `fetch` sends a zero-byte upload. The server
//! typically replies with a redirect (`30`/`31`) to the read location or a
//! failure; whatever it returns is parsed as a Gemini response.
//!
//! ## Upload (`upload`)
//!
//! For actual writes, call [`upload`] directly with the body bytes, MIME type,
//! and optional token.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use crate::client::{ClientError, ClientIdentity, Response, parse_response, tofu_connect_inner};

/// Titan shares gemini's port.
pub const DEFAULT_PORT: u16 = 1965;

/// Navigate to a `titan://` URL by sending a zero-byte upload and returning the
/// server's Gemini-format response.
pub async fn fetch(url: &Url) -> Result<Response, ClientError> {
    upload_inner(url, &[], "", None, None).await
}

/// Upload `body` to `url` with the given `mime` type and optional `token`.
/// Returns the server's Gemini-format response.
///
/// The request line is `<url>;size=<n>[;mime=<type>][;token=<token>]\r\n`
/// followed immediately by the body bytes.
pub async fn upload(
    url: &Url,
    body: &[u8],
    mime: &str,
    token: Option<&str>,
) -> Result<Response, ClientError> {
    upload_inner(url, body, mime, token, None).await
}

/// Upload while presenting one caller-selected Gemini-family client identity.
///
/// The caller remains responsible for capsule scoping. Server trust still
/// passes through the installed Gemini TOFU store before request bytes leave
/// the process.
pub async fn upload_with_identity(
    url: &Url,
    body: &[u8],
    mime: &str,
    token: Option<&str>,
    identity: ClientIdentity<'_>,
) -> Result<Response, ClientError> {
    upload_inner(url, body, mime, token, Some(identity)).await
}

async fn upload_inner(
    url: &Url,
    body: &[u8],
    mime: &str,
    token: Option<&str>,
    identity: Option<ClientIdentity<'_>>,
) -> Result<Response, ClientError> {
    let host = url
        .host_str()
        .ok_or_else(|| ClientError::BadUrl("titan URL has no host".into()))?;
    let port = url.port().unwrap_or(DEFAULT_PORT);

    let request = request_line(url, body.len(), mime, token);

    // Titan is Gemini's write companion on the same host and port. Reuse the
    // exact TOFU handshake so a changed certificate is refused before a
    // mutation request can leave the process.
    let mut tls = tofu_connect_inner(host, port, identity).await?;

    // Send request line + body.
    tls.write_all(request.as_bytes())
        .await
        .map_err(|e| ClientError::Io(e.to_string()))?;
    if !body.is_empty() {
        tls.write_all(body)
            .await
            .map_err(|e| ClientError::Io(e.to_string()))?;
    }

    // Read and parse the Gemini-format response.
    let mut raw = Vec::new();
    tls.read_to_end(&mut raw)
        .await
        .map_err(|e| ClientError::Io(e.to_string()))?;

    parse_response(&raw)
}

/// Build titan's request line:
/// `<url>;size=<n>[;mime=<type>][;token=<token>]\r\n`.
fn request_line(url: &Url, size: usize, mime: &str, token: Option<&str>) -> String {
    let mut request = format!("{url};size={size}");
    if !mime.is_empty() {
        request.push_str(";mime=");
        request.push_str(mime);
    }
    if let Some(token) = token {
        request.push_str(";token=");
        request.push_str(token);
    }
    request.push_str("\r\n");
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titan_shares_gemini_port() {
        assert_eq!(DEFAULT_PORT, crate::client::DEFAULT_PORT);
    }

    #[test]
    fn the_request_line_carries_size_mime_and_token_in_order() {
        let url = Url::parse("titan://example.org/raw/page").unwrap();
        assert_eq!(
            request_line(&url, 12, "text/gemini", Some("hunter2")),
            "titan://example.org/raw/page;size=12;mime=text/gemini;token=hunter2\r\n"
        );
    }

    #[test]
    fn an_empty_mime_and_no_token_leave_only_the_size() {
        let url = Url::parse("titan://example.org/raw/page").unwrap();
        assert_eq!(
            request_line(&url, 0, "", None),
            "titan://example.org/raw/page;size=0\r\n"
        );
    }
}
