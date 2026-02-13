use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::{
    Account, AccountCategory, AccountType, CreateAccountRequest, UpdateAccountRequest,
};
use crate::repository::{AccountRepository, RepositoryError, RepositoryResult};

/// PostgreSQL 勘定科目リポジトリ
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// SQLx の行を表す中間型（domain 層と SQLx の結合を回避）
#[derive(Debug, sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    code: String,
    name: String,
    account_type: String,
    category: String,
    description: Option<String>,
    is_active: bool,
    display_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AccountRow> for Account {
    type Error = RepositoryError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        let account_type =
            AccountType::from_str(&row.account_type).map_err(RepositoryError::DatabaseError)?;
        let category =
            AccountCategory::from_str(&row.category).map_err(RepositoryError::DatabaseError)?;

        Ok(Account {
            id: row.id,
            code: row.code,
            name: row.name,
            account_type,
            category,
            description: row.description,
            is_active: row.is_active,
            display_order: row.display_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn map_sqlx_error(err: sqlx::Error) -> RepositoryError {
    match &err {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23505") {
                let detail = db_err.message().to_string();
                RepositoryError::DuplicateCode(detail)
            } else {
                RepositoryError::DatabaseError(err.to_string())
            }
        }
        _ => RepositoryError::DatabaseError(err.to_string()),
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn create(&self, request: CreateAccountRequest) -> RepositoryResult<Account> {
        let id = Uuid::new_v4();
        let account_type = request.category.account_type();
        let display_order = request.display_order.unwrap_or(0);

        let row = sqlx::query_as::<_, AccountRow>(
            r#"
            INSERT INTO accounts (id, code, name, account_type, category, description, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.code)
        .bind(&request.name)
        .bind(account_type.to_string())
        .bind(request.category.to_string())
        .bind(&request.description)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Account::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Account>> {
        let row = sqlx::query_as::<_, AccountRow>(
            "SELECT id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at FROM accounts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Account::try_from).transpose()
    }

    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<Account>> {
        let row = sqlx::query_as::<_, AccountRow>(
            "SELECT id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at FROM accounts WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Account::try_from).transpose()
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Account>> {
        let rows = sqlx::query_as::<_, AccountRow>(
            "SELECT id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at FROM accounts ORDER BY display_order",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Account::try_from).collect()
    }

    async fn find_by_type(&self, account_type: AccountType) -> RepositoryResult<Vec<Account>> {
        let rows = sqlx::query_as::<_, AccountRow>(
            "SELECT id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at FROM accounts WHERE account_type = $1 ORDER BY display_order",
        )
        .bind(account_type.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Account::try_from).collect()
    }

    async fn update(&self, id: Uuid, request: UpdateAccountRequest) -> RepositoryResult<Account> {
        let row = sqlx::query_as::<_, AccountRow>(
            r#"
            UPDATE accounts
            SET name         = COALESCE($2, name),
                description  = COALESCE($3, description),
                display_order = COALESCE($4, display_order),
                is_active    = COALESCE($5, is_active),
                updated_at   = NOW()
            WHERE id = $1
            RETURNING id, code, name, account_type, category, description, is_active, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.display_order)
        .bind(request.is_active)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => Account::try_from(row),
            None => Err(RepositoryError::NotFound(id)),
        }
    }

    async fn soft_delete(&self, id: Uuid) -> RepositoryResult<()> {
        let result = sqlx::query(
            "UPDATE accounts SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(id));
        }

        Ok(())
    }

    async fn exists_by_code(&self, code: &str) -> RepositoryResult<bool> {
        let row = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM accounts WHERE code = $1)")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(row)
    }
}
