use std::env;
use lambda_http::lambda_runtime::Error;
use aws_sdk_dynamodb::{Client, Config, Credentials, Endpoint, Region};

pub async fn create_client() -> Result<Client, Error> {
    if env::var("DYNAMODB_SWITCH")? == "LOCAL" {
        let dynamodb_url = env::var("LOCAL_DYNAMODB_ENDPOINT")?;
        let endpoint = Endpoint::immutable(dynamodb_url.parse()?);
        let region = Region::new(env::var("REGION")?);
        let creds = Credentials::new(
            "local_access_id",
            "local_access_key",
            None,
            None,
            "local_provider",
        );
        log::info!(
            "DYNAMODB_ENDPOINT={}, REGION={}",
            dynamodb_url,
            region.to_string()
        );
        let config = Config::builder()
            .credentials_provider(creds)
            .region(region)
            .endpoint_resolver(endpoint)
            .build();
        Ok(Client::from_conf(config))
    } else {
        let config = aws_config::from_env().load().await;
        log::info!("Got config");
        Ok(Client::new(&config))
    }
}
