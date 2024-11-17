use crate::consts::XERR_HINTS;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize)]
pub enum AuthenticationError {
    #[serde(serialize_with = "serialize_status_code")]
    HttpResponseError(StatusCode),
    #[serde(serialize_with = "serialize_reqwest_error")]
    RequestError(reqwest::Error),
    MicrosoftError {
        error_type: String,
        description: String,
    },
    MaxAttemptsExceeded(String),
    XboxError {
        xerr: String,
        message: String,
        hint: String,
    },
    XSTSMissingUserHash,
    MinecraftTokenError(String),
    MinecraftProfileError {
        error: String,
        message: String,
    },
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthenticationError::HttpResponseError(status_code) => {
                f.write_fmt(format_args!("HttpResponseError: {}", status_code))
            }
            AuthenticationError::RequestError(error) => {
                f.write_fmt(format_args!("RequestError: {}", error))
            }
            AuthenticationError::MicrosoftError {
                error_type,
                description,
            } => f.write_fmt(format_args!("{}: {}", error_type, description)),
            AuthenticationError::MaxAttemptsExceeded(message) => {
                f.write_fmt(format_args!("MaxAttemptsExceeded: {}", message))
            }
            AuthenticationError::XboxError {
                xerr,
                message,
                hint,
            } => f.write_fmt(format_args!("{}: {} {}", xerr, message, hint)),
            AuthenticationError::XSTSMissingUserHash => f.write_str("XSTSMissingUserHash"),
            AuthenticationError::MinecraftTokenError(error) => f.write_str(error),
            AuthenticationError::MinecraftProfileError { error, message } => {
                f.write_fmt(format_args!("{}: {}", error, message))
            }
        }
    }
}

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

impl From<MinecraftTokenErrorResponse> for AuthenticationError {
    fn from(value: MinecraftTokenErrorResponse) -> Self {
        AuthenticationError::MinecraftTokenError(value.error)
    }
}

impl From<MincraftProfileErrorResponse> for AuthenticationError {
    fn from(value: MincraftProfileErrorResponse) -> Self {
        AuthenticationError::MinecraftProfileError {
            error: value.error,
            message: value.error_message,
        }
    }
}

fn serialize_status_code<S>(status_code: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(status_code.as_u16())
}

fn serialize_reqwest_error<S>(error: &reqwest::Error, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&error.to_string())
}

#[derive(Deserialize)]
/// Error response struct for the Microsoft OAuth authentication process.  
/// Commented out fields are currenty unused but exist in the response
pub(crate) struct MicrosoftErrorResponse {
    error: String,
    error_description: String,
    // error_codes: Vec<u32>,
    // timestamp: String,
    // trace_id: String,
    // correlation_id: String,
    // // Redirect is used for consoles.
    // error_uri: String,
}

#[derive(Deserialize)]
/// Error response struct for the XBox Live authentication process.  
/// Commented out fields are currenty unused but exist in the response
pub(crate) struct XboxErrorResponse {
    // #[serde(rename = "Identity")]
    // identity: String,
    #[serde(rename = "XErr")]
    xerr: u32,
    #[serde(rename = "Message")]
    message: String,
    // Redirect is used for consoles.
    // #[serde(rename = "Redirect")]
    // redirect: String,
}

#[derive(Deserialize)]
pub(crate) struct MinecraftTokenErrorResponse {
    error: String,
}

#[derive(Deserialize)]
/// Error response struct for the Minecraft Profile request.  
/// Commented out fields are currenty unused but exist in the response
pub(crate) struct MincraftProfileErrorResponse {
    // #[serde(rename = "errorType")]
    // error_type: String,
    error: String,
    #[serde(rename = "errorMessage")]
    error_message: String,
    // #[serde(rename = "developerMessage")]
    // developer_message: String,
}

pub type AuthenticationResult<T> = Result<T, AuthenticationError>;
