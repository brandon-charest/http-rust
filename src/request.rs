use std::io::Read;
use thiserror::Error;

#[derive(Debug)]
pub struct Request {
    request_line: RequestLine,
    state: ParseState,
}

impl Request {
    pub fn request_line(&self) -> &RequestLine {
        &self.request_line
    }
}

#[derive(Debug)]
pub struct RequestLine {
    version: String,
    request_target: String,
    method: String,
}

impl RequestLine {
    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn request_target(&self) -> &str {
        &self.request_target
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
#[derive(Debug, PartialEq)]
enum ParseState {
    Init,
    ReadingRequestLine,
    ReadingHeaders,
    ReadingBody,
    Complete,
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("missing request line")]
    MissingRequestLine,

    #[error("invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("invalid request path: {0}")]
    InvalidPath(String),

    #[error("invalid HTTP version: {0}, expected HTTP/1.0 or HTTP/1.1")]
    InvalidVersion(String),

    #[error("malformed request line: expected 'METHOD PATH VERSION'")]
    MalformedRequestLine,

    #[error("missing CRLF line terminator")]
    MissingCRLF,

    #[error("missing required header: {0}")]
    MissingHeader(String),

    #[error("invalid header format: {0}")]
    InvalidHeader(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

const CRLF: &str = "\r\n";

impl Request {
    fn new() -> Self {
        Request {
            request_line: RequestLine {
                version: String::new(),
                request_target: String::new(),
                method: String::new(),
            },
            state: ParseState::Init,
        }
    }

    /// Parse data from reader in chunks, maintaining state between reads
    fn parse_chunk(&mut self, chunk: &str, buffer: &mut String) -> Result<bool, HttpError> {
        buffer.push_str(chunk);

        loop {
            match self.state {
                ParseState::Init => {
                    self.state = ParseState::ReadingRequestLine;
                    continue;
                }
                ParseState::ReadingRequestLine => {
                    // Look for CRLF to complete the request line
                    if let Some(index) = buffer.find(CRLF) {
                        let line = &buffer[..index];
                        self.request_line = parse_request_line_raw(line)?;

                        // Remove processed data from buffer
                        *buffer = buffer[index + CRLF.len()..].to_string();
                        self.state = ParseState::ReadingHeaders;

                        // For now, we'll skip header parsing and go straight to complete
                        self.state = ParseState::Complete;
                        return Ok(true);
                    } else {
                        // Need more data
                        return Ok(false);
                    }
                }
                ParseState::ReadingHeaders => {
                    // TODO: Implement header parsing
                    self.state = ParseState::Complete;
                    return Ok(true);
                }
                ParseState::ReadingBody => {
                    // TODO: Implement body parsing
                    self.state = ParseState::Complete;
                    return Ok(true);
                }
                ParseState::Complete => return Ok(true),
            }
        }
    }
}

/// Parse an HTTP request from a reader, reading in chunks
pub fn request_from_reader(reader: &mut impl std::io::Read) -> Result<Request, HttpError> {
    let mut request = Request::new();
    let mut buffer = String::new();
    let mut read_buffer = [0u8; 1024]; // Small chunks for testing

    loop {
        let bytes_read = reader.read(&mut read_buffer)?;

        if bytes_read == 0 {
            // End of stream
            if request.state != ParseState::Complete {
                return Err(HttpError::MissingRequestLine);
            }
            break;
        }

        // Convert bytes to string
        let chunk = String::from_utf8_lossy(&read_buffer[..bytes_read]);

        // Parse the chunk
        let complete = request.parse_chunk(&chunk, &mut buffer)?;

        if complete {
            break;
        }
    }

    Ok(request)
}

/// Parse a request line that already has CRLF found and removed
fn parse_request_line_raw(line: &str) -> Result<RequestLine, HttpError> {
    let mut parts = line.split_whitespace();

    let method = parts.next().ok_or(HttpError::MalformedRequestLine)?;
    let request_target = parts.next().ok_or(HttpError::MalformedRequestLine)?;
    let version = parts.next().ok_or(HttpError::MalformedRequestLine)?;

    if !version.starts_with("HTTP/") || !version.contains("1.1") {
        return Err(HttpError::InvalidVersion(version.to_string()));
    }

    Ok(RequestLine {
        version: version.to_string(),
        request_target: request_target.to_string(),
        method: method.to_string(),
    })
}

#[cfg(test)]
struct ChunkReader {
    data: String,
    num_bytes_per_read: u64,
    pos: u64,
}

#[cfg(test)]
impl ChunkReader {
    fn new(data: String) -> Self {
        Self {
            data,
            num_bytes_per_read: 1,
            pos: 0,
        }
    }
}

#[cfg(test)]
impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        if self.pos >= self.data.len() as u64 {
            return Ok(0);
        }

        let mut end_index = self.pos + self.num_bytes_per_read;
        if end_index > self.data.len() as u64 {
            end_index = self.data.len() as u64;
        }

        let chunk = &self.data[self.pos as usize..end_index as usize];
        let bytes = chunk.as_bytes();

        // Copy data into the provided buffer
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);

        self.pos = end_index;
        Ok(len)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_request_line() {
        let line = "GET / HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.68.0\r\nAccept: */*\r\n\r\n";
        let request_line = parse_request_line_raw(line).unwrap();
        assert_eq!(request_line.method, "GET");
        assert_eq!(request_line.request_target, "/");
        assert_eq!(request_line.version, "HTTP/1.1");
    }

    #[test]
    fn test_parse_request_line_invalid_version() {
        let line = "GET / HTTP/1.0\r\nHost: localhost:7878\r\nUser-Agent: curl/7.68.0\r\nAccept: */*\r\n\r\n";
        assert!(parse_request_line_raw(line).is_err());
    }

    #[test]
    fn test_chunked_parsing() {
        // Test reading data one byte at a time
        let data = "GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n".to_string();
        let mut reader = ChunkReader::new(data);

        let request = request_from_reader(&mut reader).unwrap();

        assert_eq!(request.request_line().method(), "GET");
        assert_eq!(request.request_line().request_target(), "/test");
        assert_eq!(request.request_line().version(), "HTTP/1.1");
    }
}
