use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use crate::error::AppError;
use super::AccountDao;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub(super) account_id: String,
    pub(super) balance: BigDecimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Adjustment {
    amount: BigDecimal,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    balance: BigDecimal,
}

pub struct AccountService {
    account_dao: AccountDao,
}

// Not a lot of business logic here - could probably have just put the DAO code here.

impl AccountService {

    pub fn new(account_dao: AccountDao) -> Self {
        Self { account_dao }
    }

    pub async fn adjust_balance(&self, account_id: String, adjustment: Adjustment) -> Result<Balance, AppError> {
        let balance = self.account_dao.adjust_account(account_id, adjustment.amount).await?;
        Ok(Balance{ balance })
    }

    pub async fn create_account(&self, account: Account) -> Result<(), AppError> {
        self.account_dao.create_account(account).await?;
        Ok(())
    }

    pub async fn read_account(&self, account_id: String) -> Result<Account, AppError> {
        let account = self.account_dao.read_account(account_id).await?;
        Ok(account)
    }
}
