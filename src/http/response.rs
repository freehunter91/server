use tokio::io::{AsyncWrite, AsyncWriteExt};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, Debug)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
}

impl StatusCode {
    fn reason_phrase(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} {}", *self as u16, self.reason_phrase())
    }
}

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
    body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<Vec<u8>>) -> Self {
        Self { status_code, body }
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(&self, stream: &mut W) -> std::io::Result<()> {
        let mut response_bytes = Vec::new();
        let body_len = self.body.as_ref().map_or(0, |b| b.len());

        let response_str = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
            self.status_code,
            body_len
        );
        
        response_bytes.extend_from_slice(response_str.as_bytes());
        if let Some(body) = &self.body {
            response_bytes.extend_from_slice(body);
        }

        stream.write_all(&response_bytes).await
    }
}
