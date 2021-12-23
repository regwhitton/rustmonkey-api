use lambda_http::{handler as handler_adaptor, lambda_runtime::Error};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

// 'mod' is a bit like C's #include in that it inserts the source at this point.
// However, it is scoped within a separate named module, so you either need to
// refer to items with the module's :: prefix, or include a 'use' statement
// with bring the items into the current scope.

mod account;
mod dynamodb;
mod error;
mod web;

// Re-export for easy access at crate scope.
pub use error::AppError;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init()?;
    info!("RustMonkey-api is warming up");

    let root = wire_up_components().await?;
    let request_handler = |req, ctx| root.handle_request(req, ctx);

    lambda_runtime::run(handler_adaptor(request_handler)).await
}

use account::{AccountService,AccountDao};

async fn wire_up_components() -> Result<web::RequestHandler, Error> {
    let ddb_client = dynamodb::create_client().await?;
    let account_dao = AccountDao::new(ddb_client);
    let account_service = AccountService::new(account_dao);
    Ok(web::create_request_handler(account_service))
}
