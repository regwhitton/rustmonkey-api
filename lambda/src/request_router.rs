use http::{Method, StatusCode};
use lambda_http::{Body, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

use crate::account::AccountService;
use crate::AppError;

/// The [`RequestRouter`] component routes a request to its handling code,
/// deals with the deserialisation of the incoming parameters and payload, and
/// the serialisation of the outgoing payload.
///
/// The idea of a true lambda is to perform one function, so perhaps this sort of
/// routing should not exist.
pub struct RequestRouter {
    account_service: AccountService,
}

impl RequestRouter {
    pub fn new(account_service: AccountService) -> Self {
        Self { account_service }
    }

    /// Routes request to handling code.
    /// Deserialises JSON payload and serialises response.
    pub async fn route(&self, request: Request) -> Result<Response<Body>, AppError> {
        let path = request.uri().path().trim_end_matches('/');

        // Distinguishes between events defined in template.yaml
        if path.ends_with("/balance") {
            let account_id = get_account_id(&request)?;
            to_json_ok(
                self.account_service
                    .adjust_balance(account_id, from_payload(request)?)
                    .await?,
            )
        } else if request.method() == Method::POST {
            self.account_service
                .create_account(from_payload(request)?)
                .await?;
            empty_created_response()
        } else {
            let account_id = get_account_id(&request)?;
            to_json_ok(self.account_service.read_account(account_id).await?)
        }
    }
}

fn get_account_id(request: &Request) -> Result<String, AppError> {
    Ok(request
        .path_parameters()
        .get("accountId")
        .ok_or_else(|| AppError::bad_request_str("missing account id"))?
        .to_string())
}

/// Deserialises payload into the expected type.
fn from_payload<'a, D>(request: Request) -> Result<D, AppError>
where
    for<'de> D: Deserialize<'de>,
{
    let result = request.payload();
    let op_payload = result.map_err(|payload_err| {
        AppError::bad_request(format!("invalid payload: {}", payload_err.to_string()))
    })?;
    let payload = op_payload.ok_or_else(|| AppError::bad_request_str("missing payload"))?;
    return Ok(payload);
}

// Serialise response into a JSON payload response with an 200 OK status.
fn to_json_ok<S>(response: S) -> Result<Response<Body>, AppError>
where
    for<'a> S: Serialize,
{
    to_json(StatusCode::OK, response)
}

/// Serialise response into a JSON payload response with given HTTP status.
fn to_json<S>(status_code: StatusCode, response: S) -> Result<Response<Body>, AppError>
where
    for<'a> S: Serialize,
{
    let body = serde_json::to_string(&response)?;

    Ok(Response::builder()
        .status(status_code)
        .header(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        )
        .body(Body::Text(body))?)
}

fn empty_created_response() -> Result<Response<Body>, AppError> {
    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::Empty)?)
}
