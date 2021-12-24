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
                    // Pass through to Lambda Runtime so that it is logged and a 500 sent to the client.
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
                    serialise_error_to_json(status_code, message)
                }
            },
        }
    }
}

fn serialise_error_to_json(status_code: StatusCode, message: String) -> Result<Response<Body>, Error> {
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
    use http::StatusCode;
    use lambda_http::{lambda_runtime::Context, Body, Request, Response};
    use crate::AppError;

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
    async fn should_transform_ok_result_into_response_to_client() {
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
        // TODO test for content-type
    }

    #[tokio::test]
    async fn should_return_internal_error_to_runtime_so_it_is_logged_and_500_returned() {
        // Given
        let mut router = RequestRouter::faux();
        when!(router.route).then(|_req| {
            let internal_error = std::io::Error::new(std::io::ErrorKind::Other, "internal error");
            let wrapped_error = AppError::Internal(Box::new(internal_error));
                Err(wrapped_error)
        });
        let handler = RequestHandler::new(router);

        let request = request_with_text(REQUEST_BODY_TEXT);
        let ctx = Context::default();

        // When
        let result = handler.handle_request(request, ctx).await;

        // Then
        assert!(
            matches!(result, Err(boxed_err) if boxed_err.to_string() == "internal error"));
    }

    #[tokio::test]
    async fn should_transform_business_error_into_response_to_client() {
        // Given
        let mut router = RequestRouter::faux();
        when!(router.route).then(|_unacceptable_request| {
            Err(AppError::unprocessable("I'm sorry Dave, I'm afraid I can't let you do that"))
        });
        let handler = RequestHandler::new(router);

        let request = request_with_text(REQUEST_BODY_TEXT);
        let ctx = Context::default();

        // When
        let result = handler.handle_request(request, ctx).await;

        // Then
        assert!(
            matches!(result, Ok(resp) if
                matches!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY) &&
                resp.headers().get("Content-Type").unwrap() == "application/json" &&
                matches!(resp.body(), Body::Text(txt) if *txt == "{\"error\":\"I'm sorry Dave, I'm afraid I can't let you do that\"}")
        ));
    }

    fn request_with_text(text: &str) -> Request {
        Request::new(Body::Text(text.to_string()))
    }

    fn response_with_text(text: &str) -> Response<Body> {
        Response::builder().body(Body::Text(text.to_string())).unwrap()
    }
}
