use reqwest::StatusCode;
use serde::Deserialize;

use crate::consts::XERR_HINTS;

#[derive(Debug)]
pub enum AuthenticationError {
    HttpResponseError(StatusCode),
    RequestError(reqwest::Error),
    MicrosoftError {
        error_type: String,
        description: String,
    },
    MaxAttemptsExceeded,
    XboxError {
        xerr: String,
        message: String,
        hint: String,
    },
    XSTSMissingUserHash
}

#[derive(Deserialize)]
pub(crate) struct MicrosoftErrorResponse {
    error: String,
    error_description: String,
    error_codes: Vec<u32>,
    timestamp: String,
    trace_id: String,
    correlation_id: String,
    // Redirect is used for consoles.
    error_uri: String,
}

#[derive(Deserialize)]
pub(crate) struct XboxErrorResponse {
    #[serde(rename = "Identity")]
    identity: String,
    #[serde(rename = "XErr")]
    xerr: u32,
    #[serde(rename = "Message")]
    message: String,
    // Redirect is used for consoles.
    #[serde(rename = "Redirect")]
    redirect: String,
}

#[derive(Deserialize)]
pub(crate) struct MincraftProfileErrorResponse {
    #[serde(rename = "errorType")]
    _error_type: String,
    error: String,
    #[serde(rename = "errorMessage")]
    error_message: String,
    #[serde(rename = "developerMessage")]
    _developer_message: String,
}


pub type AuthenticationResult<T> = Result<T, AuthenticationError>;

impl From<reqwest::Error> for AuthenticationError {
    fn from(e: reqwest::Error) -> Self {
        AuthenticationError::RequestError(e)
    }
}

impl From<MicrosoftErrorResponse> for AuthenticationError {
    fn from(e: MicrosoftErrorResponse) -> Self {
        AuthenticationError::MicrosoftError {
            error_type: e.error,
            description: e.error_description,
        }
    }
}

impl From<XboxErrorResponse> for AuthenticationError {
    fn from(value: XboxErrorResponse) -> Self {
        let xerr = value.xerr.to_string();
        let hint = XERR_HINTS.get(&xerr).unwrap_or(&"");
        AuthenticationError::XboxError {
            xerr: xerr,
            message: value.message,
            hint: hint.to_string(),
        }
    }
}

impl From<MincraftProfileErrorResponse> for AuthenticationError {
    fn from(value: MincraftProfileErrorResponse) -> Self {
        todo!()
    }
}