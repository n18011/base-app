use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::domain::{Account, AccountType, CreateAccountRequest, UpdateAccountRequest};
use crate::repository::{AccountRepository, RepositoryError, RepositoryResult};

/// インメモリ勘定科目リポジトリ（テスト用）
pub struct InMemoryAccountRepository {
    accounts: RwLock<HashMap<Uuid, Account>>,
}

impl InMemoryAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn create(&self, request: CreateAccountRequest) -> RepositoryResult<Account> {
        let mut accounts = self
            .accounts
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // 重複チェック
        if accounts.values().any(|a| a.code == request.code) {
            return Err(RepositoryError::DuplicateCode(request.code));
        }

        let account = Account::new(
            request.code,
            request.name,
            request.category,
            request.description,
            request.display_order.unwrap_or(0),
        );

        accounts.insert(account.id, account.clone());

        Ok(account)
    }

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Account>> {
        let accounts = self
            .accounts
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(accounts.get(&id).cloned())
    }

    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<Account>> {
        let accounts = self
            .accounts
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(accounts.values().find(|a| a.code == code).cloned())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Account>> {
        let accounts = self
            .accounts
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut result: Vec<Account> = accounts.values().cloned().collect();
        result.sort_by(|a, b| a.display_order.cmp(&b.display_order));

        Ok(result)
    }

    async fn find_by_type(&self, account_type: AccountType) -> RepositoryResult<Vec<Account>> {
        let accounts = self
            .accounts
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut result: Vec<Account> = accounts
            .values()
            .filter(|a| a.account_type == account_type)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.display_order.cmp(&b.display_order));

        Ok(result)
    }

    async fn update(&self, id: Uuid, request: UpdateAccountRequest) -> RepositoryResult<Account> {
        let mut accounts = self
            .accounts
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let account = accounts
            .get_mut(&id)
            .ok_or(RepositoryError::NotFound(id))?;

        if let Some(name) = request.name {
            account.name = name;
        }
        if let Some(description) = request.description {
            account.description = Some(description);
        }
        if let Some(display_order) = request.display_order {
            account.display_order = display_order;
        }
        if let Some(is_active) = request.is_active {
            account.is_active = is_active;
        }

        account.updated_at = Utc::now();

        Ok(account.clone())
    }

    async fn soft_delete(&self, id: Uuid) -> RepositoryResult<()> {
        let mut accounts = self
            .accounts
            .write()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let account = accounts
            .get_mut(&id)
            .ok_or(RepositoryError::NotFound(id))?;

        account.is_active = false;
        account.updated_at = Utc::now();

        Ok(())
    }

    async fn exists_by_code(&self, code: &str) -> RepositoryResult<bool> {
        let accounts = self
            .accounts
            .read()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(accounts.values().any(|a| a.code == code))
    }
}
