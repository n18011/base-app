use accounting_service::domain::{
    AccountCategory, AccountType, CreateAccountRequest, UpdateAccountRequest,
};
use accounting_service::repository::{AccountRepository, PostgresAccountRepository, RepositoryError};
use sqlx::PgPool;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

fn create_test_request(code: &str, name: &str, category: AccountCategory) -> CreateAccountRequest {
    CreateAccountRequest {
        code: code.to_string(),
        name: name.to_string(),
        category,
        description: Some(format!("{name}の説明")),
        display_order: Some(1),
    }
}

fn default_request() -> CreateAccountRequest {
    create_test_request("101", "現金", AccountCategory::Cash)
}

// 1. 正常作成、全フィールド確認
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_create_account(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let request = default_request();

    let account = repo.create(request).await.unwrap();

    assert_eq!(account.code, "101");
    assert_eq!(account.name, "現金");
    assert_eq!(account.account_type, AccountType::Asset);
    assert_eq!(account.category, AccountCategory::Cash);
    assert_eq!(account.description, Some("現金の説明".to_string()));
    assert!(account.is_active);
    assert_eq!(account.display_order, 1);
}

// 2. 重複コード → DuplicateCode エラー
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_create_duplicate_code_fails(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let request = default_request();

    let _ = repo.create(request.clone()).await.unwrap();
    let result = repo.create(request).await;

    assert!(matches!(result, Err(RepositoryError::DuplicateCode(_))));
}

// 3. ID 検索
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_id(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let created = repo.create(default_request()).await.unwrap();

    let found = repo.find_by_id(created.id).await.unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.code, "101");
}

// 4. 存在しない ID → None
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_id_not_found(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();

    assert!(found.is_none());
}

// 5. コード検索
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_code(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let _ = repo.create(default_request()).await.unwrap();

    let found = repo.find_by_code("101").await.unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().code, "101");
}

// 6. 存在しないコード → None
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_code_not_found(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let found = repo.find_by_code("999").await.unwrap();

    assert!(found.is_none());
}

// 7. 全件取得、display_order 順
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_all(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let req1 = CreateAccountRequest {
        code: "401".to_string(),
        name: "什一献金".to_string(),
        category: AccountCategory::TitheOffering,
        description: None,
        display_order: Some(10),
    };
    let req2 = CreateAccountRequest {
        code: "101".to_string(),
        name: "現金".to_string(),
        category: AccountCategory::Cash,
        description: None,
        display_order: Some(1),
    };

    let _ = repo.create(req1).await.unwrap();
    let _ = repo.create(req2).await.unwrap();

    let all = repo.find_all().await.unwrap();

    assert_eq!(all.len(), 2);
    assert_eq!(all[0].display_order, 1);
    assert_eq!(all[1].display_order, 10);
}

// 8. 種別フィルタ
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_type(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let asset_req = create_test_request("101", "現金", AccountCategory::Cash);
    let revenue_req = create_test_request("401", "什一献金", AccountCategory::TitheOffering);

    let _ = repo.create(asset_req).await.unwrap();
    let _ = repo.create(revenue_req).await.unwrap();

    let assets = repo.find_by_type(AccountType::Asset).await.unwrap();
    let revenues = repo.find_by_type(AccountType::Revenue).await.unwrap();

    assert_eq!(assets.len(), 1);
    assert_eq!(revenues.len(), 1);
    assert_eq!(assets[0].code, "101");
    assert_eq!(revenues[0].code, "401");
}

// 9. 名前・説明更新、updated_at 更新確認
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_update_account(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let created = repo.create(default_request()).await.unwrap();

    let update_request = UpdateAccountRequest {
        name: Some("小口現金".to_string()),
        description: Some("小口経費用".to_string()),
        display_order: None,
        is_active: None,
    };

    let updated = repo.update(created.id, update_request).await.unwrap();

    assert_eq!(updated.name, "小口現金");
    assert_eq!(updated.description, Some("小口経費用".to_string()));
    assert_eq!(updated.code, "101");
    assert!(updated.updated_at >= created.updated_at);
}

// 10. 存在しない ID → NotFound エラー
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_update_not_found(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let update_request = UpdateAccountRequest {
        name: Some("テスト".to_string()),
        description: None,
        display_order: None,
        is_active: None,
    };

    let result = repo.update(Uuid::new_v4(), update_request).await;

    assert!(matches!(result, Err(RepositoryError::NotFound(_))));
}

// 11. 論理削除 (is_active=false)
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_soft_delete(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);
    let created = repo.create(default_request()).await.unwrap();

    let result = repo.soft_delete(created.id).await;
    assert!(result.is_ok());

    let found = repo.find_by_id(created.id).await.unwrap().unwrap();
    assert!(!found.is_active);
}

// 12. 存在しない ID → NotFound エラー
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_soft_delete_not_found(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    let result = repo.soft_delete(Uuid::new_v4()).await;

    assert!(matches!(result, Err(RepositoryError::NotFound(_))));
}

// 13. 存在チェック
#[sqlx::test(migrator = "MIGRATOR")]
async fn test_exists_by_code(pool: PgPool) {
    let repo = PostgresAccountRepository::new(pool);

    assert!(!repo.exists_by_code("101").await.unwrap());

    let _ = repo.create(default_request()).await.unwrap();

    assert!(repo.exists_by_code("101").await.unwrap());
}
