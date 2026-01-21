use crate::app_state::AppState;
use crate::component::v4;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Detects the type of input string
enum InputType {
    /// HTTP or HTTPS URL
    Url,
    /// Data URI (data:content-type;base64,...)
    DataUri,
    /// Raw base64 encoded string
    Base64,
}

fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        InputType::Url
    } else if trimmed.starts_with("data:") {
        InputType::DataUri
    } else {
        InputType::Base64
    }
}

/// Extracts filename from a URL path
fn filename_from_url(url: &str) -> Option<String> {
    url.split('?')
        .next()
        .and_then(|path| path.split('/').last())
        .filter(|name| !name.is_empty() && name.contains('.'))
        .map(|s| s.to_string())
}

/// Generates a filename based on content type
fn filename_from_content_type(content_type: &str) -> String {
    let extension = match content_type {
        "application/pdf" => "pdf",
        "application/json" => "json",
        "application/xml" => "xml",
        "application/zip" => "zip",
        "application/gzip" => "gz",
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/css" => "css",
        "text/javascript" => "js",
        "text/csv" => "csv",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        _ => "bin",
    };
    format!("file.{}", extension)
}

/// Detects content type from bytes using magic number detection
fn detect_content_type(bytes: &[u8]) -> String {
    infer::get(bytes)
        .map(|kind| kind.mime_type().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

// ============================================================================
// Internal error type (version-agnostic)
// ============================================================================

/// Internal error type for file operations - converted to version-specific errors by macros
#[derive(Debug)]
#[allow(dead_code)] // Timeout and Other are needed for WIT compatibility but not currently used
enum NormalizeError {
    FetchFailed(String),
    InvalidInput(String),
    Timeout(String),
    Other(String),
}

/// Parses a data URI and returns (content_type, decoded_bytes)
fn parse_data_uri(data_uri: &str) -> Result<(String, Vec<u8>), NormalizeError> {
    // Format: data:[<mediatype>][;base64],<data>
    let without_prefix = data_uri
        .strip_prefix("data:")
        .ok_or_else(|| NormalizeError::InvalidInput("Invalid data URI format".to_string()))?;

    let (metadata, data) = without_prefix
        .split_once(',')
        .ok_or_else(|| NormalizeError::InvalidInput("Data URI missing comma separator".to_string()))?;

    let is_base64 = metadata.ends_with(";base64");
    let content_type = if is_base64 {
        metadata.strip_suffix(";base64").unwrap_or("application/octet-stream")
    } else if metadata.is_empty() {
        "text/plain;charset=US-ASCII"
    } else {
        metadata
    };

    let bytes = if is_base64 {
        BASE64
            .decode(data)
            .map_err(|e| NormalizeError::InvalidInput(format!("Invalid base64 in data URI: {}", e)))?
    } else {
        // URL-encoded data
        urlencoding_decode(data)
            .map_err(|e| NormalizeError::InvalidInput(format!("Invalid URL encoding: {}", e)))?
    };

    Ok((content_type.to_string(), bytes))
}

/// Simple URL decoding for non-base64 data URIs
fn urlencoding_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte);
                } else {
                    return Err(format!("Invalid hex sequence: %{}", hex));
                }
            } else {
                return Err("Incomplete percent encoding".to_string());
            }
        } else {
            result.push(c as u8);
        }
    }

    Ok(result)
}

/// Fetches content from a URL with optional headers
fn fetch_url(
    client: &reqwest::blocking::Client,
    url: &str,
    headers: Option<&Vec<(String, String)>>,
) -> Result<(Vec<u8>, Option<String>, Option<String>), NormalizeError> {
    let mut request = client.get(url);

    // Add custom headers if provided
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            request = request.header(key, value);
        }
    }

    let response = request
        .send()
        .map_err(|e| NormalizeError::FetchFailed(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(NormalizeError::FetchFailed(format!(
            "HTTP {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());

    let filename = filename_from_url(url);

    let bytes = response
        .bytes()
        .map_err(|e| NormalizeError::FetchFailed(format!("Failed to read response body: {}", e)))?
        .to_vec();

    Ok((bytes, content_type, filename))
}

// ============================================================================
// Shared file normalization logic (used by all versions with file interface)
// ============================================================================

fn normalize_file(
    client: &std::sync::Arc<std::sync::Mutex<reqwest::blocking::Client>>,
    source: &str,
    headers: Option<&Vec<(String, String)>>,
    filename: Option<String>,
) -> Result<(String, String, String), NormalizeError> {
    let (bytes, content_type, url_filename) = match detect_input_type(source) {
        InputType::Url => {
            let client = client.lock().unwrap();
            fetch_url(&client, source, headers)?
        }
        InputType::DataUri => {
            let (ct, bytes) = parse_data_uri(source)?;
            (bytes, Some(ct), None)
        }
        InputType::Base64 => {
            let bytes = BASE64
                .decode(source)
                .map_err(|e| NormalizeError::InvalidInput(format!("Invalid base64: {}", e)))?;
            (bytes, None, None)
        }
    };

    let detected_type = content_type.unwrap_or_else(|| detect_content_type(&bytes));

    // Priority: explicit filename > URL filename > generated from content type
    let final_filename = filename
        .or(url_filename)
        .unwrap_or_else(|| filename_from_content_type(&detected_type));

    Ok((BASE64.encode(&bytes), detected_type, final_filename))
}

// ============================================================================
// Macro to implement file::Host for any version that has the file interface
//
// When adding v5 (if it has the file interface), just add:
//   impl_file_host!(v5);
// ============================================================================

macro_rules! impl_file_host {
    ($v:ident) => {
        impl $v::standout::app::file::Host for AppState {
            fn normalize(
                &mut self,
                source: String,
                headers: Option<Vec<(String, String)>>,
                filename: Option<String>,
            ) -> Result<$v::standout::app::file::FileData, $v::standout::app::file::FileError> {
                match normalize_file(&self.client, &source, headers.as_ref(), filename) {
                    Ok((base64, content_type, filename)) => Ok($v::standout::app::file::FileData {
                        base64,
                        content_type,
                        filename,
                    }),
                    Err(e) => Err(match e {
                        NormalizeError::FetchFailed(msg) => $v::standout::app::file::FileError::FetchFailed(msg),
                        NormalizeError::InvalidInput(msg) => $v::standout::app::file::FileError::InvalidInput(msg),
                        NormalizeError::Timeout(msg) => $v::standout::app::file::FileError::Timeout(msg),
                        NormalizeError::Other(msg) => $v::standout::app::file::FileError::Other(msg),
                    }),
                }
            }
        }
    };
}

// Generate file::Host implementation for v4
// Note: v3 doesn't have the file interface, so no impl needed
// When adding v5, add: impl_file_host!(v5);
impl_file_host!(v4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_input_type_url() {
        assert!(matches!(
            detect_input_type("https://example.com/file.pdf"),
            InputType::Url
        ));
        assert!(matches!(
            detect_input_type("http://example.com/file.pdf"),
            InputType::Url
        ));
    }

    #[test]
    fn test_detect_input_type_data_uri() {
        assert!(matches!(
            detect_input_type("data:application/pdf;base64,JVBERi0="),
            InputType::DataUri
        ));
    }

    #[test]
    fn test_detect_input_type_base64() {
        assert!(matches!(
            detect_input_type("JVBERi0xLjQK"),
            InputType::Base64
        ));
    }

    #[test]
    fn test_filename_from_url() {
        assert_eq!(
            filename_from_url("https://example.com/path/to/file.pdf"),
            Some("file.pdf".to_string())
        );
        assert_eq!(
            filename_from_url("https://example.com/file.pdf?token=abc"),
            Some("file.pdf".to_string())
        );
        assert_eq!(filename_from_url("https://example.com/"), None);
    }

    #[test]
    fn test_filename_from_content_type() {
        assert_eq!(filename_from_content_type("application/pdf"), "file.pdf");
        assert_eq!(filename_from_content_type("image/png"), "file.png");
        assert_eq!(filename_from_content_type("unknown/type"), "file.bin");
    }

    #[test]
    fn test_parse_data_uri_base64() {
        let (content_type, bytes) = parse_data_uri("data:text/plain;base64,SGVsbG8=").unwrap();
        assert_eq!(content_type, "text/plain");
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_parse_data_uri_no_base64() {
        let (content_type, bytes) = parse_data_uri("data:text/plain,Hello%20World").unwrap();
        assert_eq!(content_type, "text/plain");
        assert_eq!(bytes, b"Hello World");
    }
}
