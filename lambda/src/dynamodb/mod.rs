use std::env;
use lambda_http::lambda_runtime::Error;
use aws_sdk_dynamodb::{Client, Config, Credentials, Endpoint, Region};

/// Create a DynamoDB client.
///
/// If DYNAMONDB_SWITCH environment variable is "LOCAL" (see template.yaml)
/// then LOCAL_DYNAMODB_ENDPOINT and REGION are used to connect to the
/// dynamodb-local. Otherwise the connection is made to the AWS infrastructure.
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
        Ok(Client::new(&config))
    }
}
