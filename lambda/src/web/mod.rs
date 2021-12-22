use crate::account::AccountService;

mod request_handler;
pub use request_handler::RequestHandler;

mod request_router;
use request_router::RequestRouter;

pub fn create_request_handler(account_service: AccountService) -> RequestHandler {
    RequestHandler::new(RequestRouter::new(account_service))
}
