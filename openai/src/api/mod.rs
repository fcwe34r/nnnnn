use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Stream;

use crate::{debug, warn};

use self::models::resp::PostConvoResponse;

pub mod chatgpt;
pub mod models;
pub mod opengpt;

pub const HEADER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";
pub const URL_CHATGPT_API: &str = "https://chat.openai.com";
pub const URL_PLATFORM_API: &str = "https://api.openai.com";

pub type ApiResult<T, E = ApiError> = anyhow::Result<T, E>;

pub enum RequestMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("failed to cookie")]
    FailedGetCookieError,
    #[error("invalid cookie")]
    InvalidCookieError,
    #[error(transparent)]
    SerdeDeserializeError(#[from] serde_json::error::Error),
    #[error(transparent)]
    JsonReqwestDeserializeError(#[from] reqwest::Error),
    #[error(transparent)]
    JsonAnyhowDeserializeError(#[from] anyhow::Error),
    #[error("failed serialize `{0}`")]
    SerializeError(String),
    #[error("system time exception")]
    SystemTimeExceptionError,
    #[error("too many requests `{0}`")]
    TooManyRequestsError(String),
    #[error("failed authentication `{0}`")]
    BadAuthenticationError(String),
    #[error("failed request `{0}`")]
    FailedRequestError(String),
    #[error("redirection error")]
    RedirectionError,
    #[error("bad request `{0}`")]
    BadRequestError(String),
    #[error("server error")]
    ServerError,
    #[error("format prefix string error")]
    FormatPrefixStringError,
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("required parameter `{0}`")]
    RequiredParameter(String),
}

pub trait RefreshToken: Sync + Send {
    /// refresh access token
    fn refresh_token(&mut self, access_token: String);
}

pub trait Success {
    fn ok(&self) -> bool;
}

pub struct PostConvoStreamResponse {
    response: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    first_chunk: bool,
}

impl PostConvoStreamResponse {
    pub fn new(
        response: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    ) -> Self {
        Self {
            response,
            first_chunk: true,
        }
    }
}

impl Stream for PostConvoStreamResponse {
    type Item = PostConvoResponse;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.response.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    let mut utf8_str = String::from_utf8_lossy(&chunk).to_string();

                    if self.first_chunk {
                        let lines: Vec<&str> = utf8_str.lines().collect();
                        utf8_str = if lines.len() >= 2 {
                            lines[lines.len() - 2].to_string()
                        } else {
                            utf8_str.clone()
                        };
                        self.first_chunk = false;
                    }

                    let trimmed_str = utf8_str.trim_start_matches("data: ");

                    let json_result = serde_json::from_str::<Self::Item>(trimmed_str);

                    match json_result {
                        Ok(json) => {
                            return Poll::Ready(Some(json));
                        }
                        Err(e) => {
                            debug!("Error in stream: {:?}", e);
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    warn!("Error in stream: {:?}", e);
                    return Poll::Ready(None);
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
