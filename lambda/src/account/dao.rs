use crate::error::AppError;
use aws_sdk_dynamodb::{
    client::fluent_builders::UpdateItem,
    error::UpdateItemError,
    model::{AttributeValue, ReturnValue},
    Client,
    SdkError::{self, ServiceError},
};
use bigdecimal::{num_bigint::Sign, BigDecimal};
use std::{collections::HashMap, ops::Neg, str::FromStr};

use super::Account;

pub struct AccountDao {
    ddb_client: Client,
}

// For Client API see https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/client/index.html
// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/SQLtoNoSQL.UpdateData.html
// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.UpdateExpressions.html

impl AccountDao {
    pub fn new(ddb_client: Client) -> Self {
        Self { ddb_client }
    }

    pub async fn adjust_account(
        &self,
        account_id: String,
        amount: BigDecimal,
    ) -> Result<BigDecimal, AppError> {
        let update = if amount.sign() != Sign::Minus {
            self.update_to_change_balance(account_id, amount)
        } else {
            let min_balance = amount.to_owned().neg();
            self.update_with_min_balance_condition(account_id, amount, min_balance)
        };

        let output = update
            .send()
            .await
            .map_err(|err| map_condition_failure_to(err, "insufficient funds"))?;
        // TODO - trap error when account unknown.

        let attrs = output.attributes.ok_or_else(AppError::not_found)?;

        let balance = unpack_balance(attrs)?;
        Ok(balance)
    }

    fn update_to_change_balance(&self, account_id: String, amount: BigDecimal) -> UpdateItem {
        self.ddb_client
            .update_item()
            .table_name("Accounts")
            .key("accountId", AttributeValue::S(account_id))
            .update_expression("SET balance = balance + :amount")
            .expression_attribute_values(":amount", AttributeValue::N(amount.to_string()))
            .return_values(ReturnValue::UpdatedNew)
    }

    fn update_with_min_balance_condition(
        &self,
        account_id: String,
        amount: BigDecimal,
        min_balance: BigDecimal,
    ) -> UpdateItem {
        self.update_to_change_balance(account_id, amount)
            .condition_expression("balance >= :min_bal")
            .expression_attribute_values(":min_bal", AttributeValue::N(min_balance.to_string()))
    }

    pub async fn create_account(&self, account: Account) -> Result<(), AppError> {
        let put = self
            .ddb_client
            .put_item()
            .table_name("Accounts")
            .item("accountId", AttributeValue::S(account.account_id))
            .item("balance", AttributeValue::N(account.balance.to_string()));

        put.send().await?;
        Ok(())
    }

    pub async fn read_account(&self, account_id: String) -> Result<Account, AppError> {
        let get = self
            .ddb_client
            .get_item()
            .table_name("Accounts")
            .key("accountId", AttributeValue::S(account_id))
            .consistent_read(false);

        let attrs = get.send().await?
            .item.ok_or_else(AppError::not_found)?;

        let account = unpack_account(attrs)?;
        Ok(account)
    }
}

fn map_condition_failure_to(upd_err: SdkError<UpdateItemError>, message: &str) -> AppError {
    if is_condition_failure(&upd_err) {
        AppError::unprocessable(message)
    } else {
        AppError::Internal(Box::new(upd_err))
    }
}

fn is_condition_failure(upd_err: &SdkError<UpdateItemError>) -> bool {
    matches!(upd_err, ServiceError{err, raw: _} if err.is_conditional_check_failed_exception())
}

fn unpack_balance(attrs: HashMap<String, AttributeValue>) -> Result<BigDecimal, AppError> {
    let balance = decimal_attr(&attrs, "balance")?;
    Ok(balance.normalized())
}

fn unpack_account(attrs: HashMap<String, AttributeValue>) -> Result<Account, AppError> {
    let account_id = str_attr(&attrs, "accountId")?.to_string();
    let balance = decimal_attr(&attrs, "balance")?.normalized();
    Ok(Account {
        account_id,
        balance,
    })
}

fn str_attr(attrs: &HashMap<String, AttributeValue>, attr_name: &str) -> Result<String, AppError> {
    let av = attrs
        .get(attr_name)
        .ok_or_else(|| app_err(format!("{} not returned by dynamodb", attr_name)))?;
    let val = av
        .as_s()
        .or_else(|_av| Err(app_err(format!("{} not returned by dynamodb", attr_name))))?;
    Ok(val.to_owned())
}

fn decimal_attr(
    attrs: &HashMap<String, AttributeValue>,
    attr_name: &str,
) -> Result<BigDecimal, AppError> {
    let av = attrs
        .get(attr_name)
        .ok_or_else(|| app_err(format!("{} not returned by dynamodb", attr_name)))?;
    let val = av
        .as_n()
        .or_else(|_av| Err(app_err(format!("{} not returned by dynamodb", attr_name))))?;
    let val = BigDecimal::from_str(val)?;
    Ok(val)
}

fn app_err(message: String) -> AppError {
    AppError::internal_s(message)
}

#[cfg(test)]
mod test {

    use std::{sync::Once, process::{Command, Stdio, Child, ChildStdout, ChildStdin}, str::FromStr, io::{Write, BufRead, BufReader}};
    use aws_sdk_dynamodb::{Client, Config, Credentials, Endpoint, Region};
    use super::{AccountDao, Account};
    use bigdecimal::BigDecimal;
    use http::Uri;

    #[tokio::test]
    async fn should_create_new_account() {
        // Given
        let account_id = "NEWACC001".to_string();
        let amount = BigDecimal::from_str("10.10").expect("failed to parse number");
        let account = Account{account_id: account_id.clone(), balance: amount.clone()};

        let dao = AccountDao::new(get_dynamodb_client());

        // When
        dao.create_account(account).await.expect("could not create account");

        // Then
        let current_account = dao.read_account(account_id.clone()).await.expect("could not read account");
        assert_eq!(current_account.account_id, account_id);
        assert_eq!(current_account.balance, amount);
    }

    // These tests use DynamoDB in a docker container.
    // The container is started and populated using script db/account-dao-test-setup.sh
    // The script shuts down the container when the input stream closes, 
    // which happens when the this test process exits.
    // It's all terribly clunky, and should be moved into its own module.

    fn get_dynamodb_client() -> Client {
        unsafe {
            DB_INITIALISER.call_once(|| {
                DB_CLIENT = Some(setup_dynamodb_client());
            });
            match &DB_CLIENT {
                Some(client) => client.clone(),
                None => panic!("db client not initialised")
            }
        }
    }

    static mut DB_CLIENT: Option<Client> = Option::None;
    static DB_INITIALISER: Once = Once::new();

    fn setup_dynamodb_client() -> Client {
        let mut dynamodb_process: Child = start_db_setup_script();

        let script_stdout = dynamodb_process.stdout.take().expect("Failed to open script stdout");
        let dynamodb_url = read_output_until_db_ready(script_stdout);

        // When tests finish the input stream is closed, this causes the DB to shutdown.
        let script_stdin = dynamodb_process.stdin.take().expect("Failed to open script stdin");
        establish_input_stream(script_stdin);

        let dynamodb_uri = dynamodb_url.parse().expect("Could not parse URL");
        create_dynamodb_client(dynamodb_uri)
    }

    fn create_dynamodb_client(dynamodb_uri: Uri) -> Client {
        let endpoint = Endpoint::immutable(dynamodb_uri);
        let region = Region::new("eu-west-2");
        let creds = Credentials::new(
            "local_access_id",
            "local_access_key",
            None,
            None,
            "local_provider",
        );
        let config = Config::builder()
            .credentials_provider(creds)
            .region(region)
            .endpoint_resolver(endpoint)
            .build();
        Client::from_conf(config)
    }

    fn start_db_setup_script() -> Child {
        Command::new("bash")
            .arg("../db/account-dao-test-setup.sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn account-dao-test-setup.sh")
    }

    fn read_output_until_db_ready(script_stdout: ChildStdout) -> String {
        let mut reader = BufReader::new(script_stdout);
        let mut line = String::new();

        loop {
            match reader.read_line(&mut line) {
                Ok(n_bytes) => {
                    if n_bytes == 0 {
                        panic!("Failed to find READY marker in script output");
                    }
                    let mut word_iter = line.split_whitespace();
                    if matches!(word_iter.next(), Some(word) if word == "READY") {
                        match word_iter.next() {
                            Some(dynamo_url) => {
                                return String::from(dynamo_url);
                            },
                            None => panic!("READY marker not followed by DB URL")
                        };
                    }
                },
                Err(error) => panic!("Failed to read READY marker: {:?}", error)
            }
        }
    }

    fn establish_input_stream(mut script_stdin: ChildStdin) {
        script_stdin.write("Tests beginning\n".as_bytes()).expect("Failed to write script stdin");
        script_stdin.flush().expect("Failed to flush buffer");
    }
}
