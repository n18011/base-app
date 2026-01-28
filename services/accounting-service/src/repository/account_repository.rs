use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{Account, AccountType, CreateAccountRequest, UpdateAccountRequest};

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Account not found: {0}")]
    NotFound(Uuid),

    #[error("Account code already exists: {0}")]
    DuplicateCode(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// 勘定科目リポジトリインターフェース
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// 勘定科目を作成
    async fn create(&self, request: CreateAccountRequest) -> RepositoryResult<Account>;

    /// IDで勘定科目を取得
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<Account>>;

    /// 科目コードで勘定科目を取得
    async fn find_by_code(&self, code: &str) -> RepositoryResult<Option<Account>>;

    /// 全勘定科目を取得
    async fn find_all(&self) -> RepositoryResult<Vec<Account>>;

    /// 科目種別で勘定科目を取得
    async fn find_by_type(&self, account_type: AccountType) -> RepositoryResult<Vec<Account>>;

    /// 勘定科目を更新
    async fn update(&self, id: Uuid, request: UpdateAccountRequest) -> RepositoryResult<Account>;

    /// 勘定科目を論理削除（is_active = false）
    async fn soft_delete(&self, id: Uuid) -> RepositoryResult<()>;

    /// 科目コードの重複チェック
    async fn exists_by_code(&self, code: &str) -> RepositoryResult<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AccountCategory;
    use crate::repository::InMemoryAccountRepository;

    fn create_test_request() -> CreateAccountRequest {
        CreateAccountRequest {
            code: "101".to_string(),
            name: "現金".to_string(),
            category: AccountCategory::Cash,
            description: Some("手許現金".to_string()),
            display_order: Some(1),
        }
    }

    #[tokio::test]
    async fn test_create_account() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();

        let result = repo.create(request).await;

        assert!(result.is_ok());
        let account = result.unwrap();
        assert_eq!(account.code, "101");
        assert_eq!(account.name, "現金");
        assert_eq!(account.account_type, AccountType::Asset);
    }

    #[tokio::test]
    async fn test_create_duplicate_code_fails() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();

        let _ = repo.create(request.clone()).await.unwrap();
        let result = repo.create(request).await;

        assert!(matches!(result, Err(RepositoryError::DuplicateCode(_))));
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();
        let created = repo.create(request).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = InMemoryAccountRepository::new();
        let random_id = Uuid::new_v4();

        let found = repo.find_by_id(random_id).await.unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_code() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();
        let _ = repo.create(request).await.unwrap();

        let found = repo.find_by_code("101").await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().code, "101");
    }

    #[tokio::test]
    async fn test_find_all() {
        let repo = InMemoryAccountRepository::new();

        let request1 = CreateAccountRequest {
            code: "101".to_string(),
            name: "現金".to_string(),
            category: AccountCategory::Cash,
            description: None,
            display_order: Some(1),
        };
        let request2 = CreateAccountRequest {
            code: "401".to_string(),
            name: "什一献金".to_string(),
            category: AccountCategory::TitheOffering,
            description: None,
            display_order: Some(10),
        };

        let _ = repo.create(request1).await.unwrap();
        let _ = repo.create(request2).await.unwrap();

        let all = repo.find_all().await.unwrap();

        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_type() {
        let repo = InMemoryAccountRepository::new();

        let asset_request = CreateAccountRequest {
            code: "101".to_string(),
            name: "現金".to_string(),
            category: AccountCategory::Cash,
            description: None,
            display_order: Some(1),
        };
        let revenue_request = CreateAccountRequest {
            code: "401".to_string(),
            name: "什一献金".to_string(),
            category: AccountCategory::TitheOffering,
            description: None,
            display_order: Some(10),
        };

        let _ = repo.create(asset_request).await.unwrap();
        let _ = repo.create(revenue_request).await.unwrap();

        let assets = repo.find_by_type(AccountType::Asset).await.unwrap();
        let revenues = repo.find_by_type(AccountType::Revenue).await.unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(revenues.len(), 1);
        assert_eq!(assets[0].code, "101");
        assert_eq!(revenues[0].code, "401");
    }

    #[tokio::test]
    async fn test_update_account() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();
        let created = repo.create(request).await.unwrap();

        let update_request = UpdateAccountRequest {
            name: Some("小口現金".to_string()),
            description: Some("小口経費用".to_string()),
            display_order: None,
            is_active: None,
        };

        let updated = repo.update(created.id, update_request).await.unwrap();

        assert_eq!(updated.name, "小口現金");
        assert_eq!(updated.description, Some("小口経費用".to_string()));
        assert_eq!(updated.code, "101"); // codeは変更されない
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let repo = InMemoryAccountRepository::new();
        let random_id = Uuid::new_v4();

        let update_request = UpdateAccountRequest {
            name: Some("テスト".to_string()),
            description: None,
            display_order: None,
            is_active: None,
        };

        let result = repo.update(random_id, update_request).await;

        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_soft_delete() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();
        let created = repo.create(request).await.unwrap();

        let result = repo.soft_delete(created.id).await;
        assert!(result.is_ok());

        let found = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert!(!found.is_active);
    }

    #[tokio::test]
    async fn test_exists_by_code() {
        let repo = InMemoryAccountRepository::new();
        let request = create_test_request();

        assert!(!repo.exists_by_code("101").await.unwrap());

        let _ = repo.create(request).await.unwrap();

        assert!(repo.exists_by_code("101").await.unwrap());
    }
}
