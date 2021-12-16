use bigdecimal::ParseBigDecimalError;
use http::StatusCode;
use lambda_http::lambda_runtime::Error;
use std::fmt;

/// Error used throughout this application.
///
/// When the AWS Lambda Runtime gets a [`lambda_http::lambda_runtime::Error`] it
/// will log the contents at ERROR level and return a 500 series status to the client.
///
/// [`AppError`] allows for situations where an operation has been prevented by
/// a business logic rule. In these cases we want to prevent the Lambda runtime
/// logging at ERROR level and have a more appropriate response is sent to the client.
///
/// Implementations of [`std::error::Error`] are normally converted directly into
/// [`lambda_http::lambda_runtime::Error`] by the standard library because this is
/// actually a type definition for [`Box<dyn std::error::Error + Send + Sync>`]
/// which the library has code for.
#[derive(Debug)]
pub enum AppError {
    /// An internal error that would not be expected to happen during normal operation.
    ///
    /// This will be passed to the Lambda Runtime to be logged as an ERROR and have a 500 status
    /// returned to the client. The no message will sent to the client as it is not expected
    /// to be meaningful to a user.
    ///
    /// An example of this type of error is a connection failure when retrieving data.
    Internal(Error),

    /// The operation was prevented by a business logic rule.
    /// A 4XX series status is returned to the client with a payload containing a message.
    /// The message is expected to be meaningful to a user.
    Business(String, StatusCode),
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Internal(error) => Some(error.as_ref()),
            AppError::Business(_message, _status) => None,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, fmttr: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Internal(error) => error.fmt(fmttr),
            AppError::Business(status, message) => {
                write!(fmttr, "Business: {} {}", status, message)
            }
        }
    }
}

impl AppError {
    pub fn bad_request_str(message: &str) -> AppError {
        AppError::bad_request(message.to_string())
    }

    pub fn bad_request(message: String) -> AppError {
        AppError::Business(message, StatusCode::BAD_REQUEST)
    }

    pub fn not_found() -> AppError {
        AppError::Business("not found".to_string(), StatusCode::NOT_FOUND)
    }

    pub fn unprocessable(message: &str) -> AppError {
        AppError::Business(message.to_string(), StatusCode::UNPROCESSABLE_ENTITY)
    }

    pub fn wrap_internal(error: &'static (dyn std::error::Error + Send + Sync)) -> AppError {
        AppError::Internal(Box::new(error))
    }

    pub fn internal_s(message: String) -> AppError {
        AppError::Internal(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            message.as_str(),
        )))
    }

    pub fn internal(message: &'static str) -> AppError {
        AppError::Internal(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            message,
        )))
    }
}

use aws_sdk_dynamodb::SdkError;
impl<T: std::error::Error + Send + Sync + 'static> From<SdkError<T>> for AppError {
    fn from(err: SdkError<T>) -> AppError {
        AppError::Internal(Box::new(err))
    }
}

use serde_json::Error as SerdeError;
impl From<SerdeError> for AppError {
    fn from(err: SerdeError) -> AppError {
        AppError::Internal(Box::new(err))
    }
}

impl From<http::Error> for AppError {
    fn from(err: http::Error) -> AppError {
        AppError::Internal(Box::new(err))
    }
}

impl From<ParseBigDecimalError> for AppError {
    fn from(err: ParseBigDecimalError) -> AppError {
        AppError::Internal(Box::new(err))
    }
}
