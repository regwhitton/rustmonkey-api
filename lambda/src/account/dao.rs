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
