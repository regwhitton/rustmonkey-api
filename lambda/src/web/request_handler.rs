use http::StatusCode;
use lambda_http::{lambda_runtime::Context, lambda_runtime::Error, Body, Request, Response};
use serde::Serialize;

use super::request_router::RequestRouter;
use crate::AppError;

/// The [`RequestHandler`] component routes a request then handles
/// any business error by converting it a JSON response to client..
pub struct RequestHandler {
    router: RequestRouter,
}

impl RequestHandler {
    pub fn new(router: RequestRouter) -> Self {
        Self { router }
    }

    /// Routes request to handling function, handles
    /// any error by converting it a JSON response to client.
    pub async fn handle_request(
        &self,
        request: Request,
        ctx: Context,
    ) -> Result<Response<Body>, Error> {
        log::info!(
            "requestId:{} request start {} {}",
            ctx.request_id,
            request.method(),
            request.uri().path()
        );

        match self.router.route(request).await {
            Ok(response) => {
                log::info!("requestId:{} request end", ctx.request_id);
                Ok(response)
            }
            Err(app_err) => match app_err {
                AppError::Internal(error) => {
                    // Pass through to Lambda Runtime to log and deal with
                    Err(error)
                }
                AppError::Business(message, status_code) => {
                    // Convert business rule violations into JSON error message for client.
                    log::info!(
                        "requestId:{} client error: {} {}",
                        ctx.request_id,
                        status_code,
                        message
                    );
                    to_error_json(status_code, message)
                }
            },
        }
    }
}

// Serialises argument into a JSON payload response with given HTTP status.
fn to_error_json(status_code: StatusCode, message: String) -> Result<Response<Body>, Error> {
    let body = serde_json::to_string(&ErrorDetails { error: message })?;

    Ok(Response::builder()
        .status(status_code)
        .header(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        )
        .body(Body::Text(body))?)
}

/// Used to serialise JSON describing an error.
#[derive(Debug, Serialize, Default)]
struct ErrorDetails {
    #[serde(default)]
    error: String,
}

#[cfg(test)]
mod test {
    use super::{RequestHandler, RequestRouter};
    use faux::when;
    use lambda_http::{lambda_runtime::Context, Body, Request, Response};

    const REQUEST_BODY_TEXT: &str = "{\"accountId\":\"sid\"}";
    const RESPONSE_BODY_TEXT: &str = "{\"balance\":25.10}";

    #[tokio::test]
    async fn should_pass_request_to_router() {
        // Given
        let mut router = RequestRouter::faux();
        when!(router.route).then(|req| {
            // Then
            assert!(matches!(req.body(), Body::Text(txt) if *txt == REQUEST_BODY_TEXT));
            Ok(response_with_text(""))
        });
        let handler = RequestHandler::new(router);

        let request = request_with_text(REQUEST_BODY_TEXT);
        let ctx = Context::default();

        // When
        let _ = handler.handle_request(request, ctx).await;
    }

    #[tokio::test]
    async fn should_return_ok_response() {
        // Given
        let mut router = RequestRouter::faux();
        when!(router.route).then(|_req| {
            Ok(response_with_text(RESPONSE_BODY_TEXT))
        });
        let handler = RequestHandler::new(router);

        let request = request_with_text(REQUEST_BODY_TEXT);
        let ctx = Context::default();

        // When
        let result = handler.handle_request(request, ctx).await;

        // Then
        assert!(
            matches!(result, Ok(resp) if 
                matches!(resp.body(), Body::Text(txt) if
                    *txt == RESPONSE_BODY_TEXT)));
    }

    fn request_with_text(text: &str) -> Request {
        Request::new(Body::Text(text.to_string()))
    }

    fn response_with_text(text: &str) -> Response<Body> {
        Response::builder().body(Body::Text(text.to_string())).unwrap()
    }
}
