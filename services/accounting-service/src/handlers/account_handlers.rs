use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::domain::{
    AccountResponse, AccountType, CreateAccountRequest, UpdateAccountRequest,
};
use crate::repository::{AccountRepository, RepositoryError};

pub type DynAccountRepository = Arc<dyn AccountRepository>;

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub account_type: Option<AccountType>,
}

/// エラーレスポンス
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ErrorResponse {
    fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }
}

fn map_repo_error(err: RepositoryError) -> (StatusCode, Json<ErrorResponse>) {
    match err {
        RepositoryError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                format!("Account not found: {}", id),
                "NOT_FOUND",
            )),
        ),
        RepositoryError::DuplicateCode(code) => (
            StatusCode::CONFLICT,
            Json(ErrorResponse::new(
                format!("Account code already exists: {}", code),
                "DUPLICATE_CODE",
            )),
        ),
        RepositoryError::ValidationError(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(msg, "VALIDATION_ERROR")),
        ),
        RepositoryError::DatabaseError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(msg, "DATABASE_ERROR")),
        ),
    }
}

/// POST /api/accounts - 勘定科目作成
pub async fn create_account(
    State(repo): State<DynAccountRepository>,
    Json(request): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    // バリデーション
    if let Err(errors) = request.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                format!("Validation failed: {}", errors),
                "VALIDATION_ERROR",
            )),
        )
            .into_response();
    }

    match repo.create(request).await {
        Ok(account) => (StatusCode::CREATED, Json(AccountResponse::from(account))).into_response(),
        Err(err) => map_repo_error(err).into_response(),
    }
}

/// GET /api/accounts - 勘定科目一覧取得
pub async fn list_accounts(
    State(repo): State<DynAccountRepository>,
    Query(query): Query<ListAccountsQuery>,
) -> impl IntoResponse {
    let result = if let Some(account_type) = query.account_type {
        repo.find_by_type(account_type).await
    } else {
        repo.find_all().await
    };

    match result {
        Ok(accounts) => {
            let responses: Vec<AccountResponse> =
                accounts.into_iter().map(AccountResponse::from).collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(err) => map_repo_error(err).into_response(),
    }
}

/// GET /api/accounts/:id - 勘定科目詳細取得
pub async fn get_account(
    State(repo): State<DynAccountRepository>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo.find_by_id(id).await {
        Ok(Some(account)) => (StatusCode::OK, Json(AccountResponse::from(account))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                format!("Account not found: {}", id),
                "NOT_FOUND",
            )),
        )
            .into_response(),
        Err(err) => map_repo_error(err).into_response(),
    }
}

/// PUT /api/accounts/:id - 勘定科目更新
pub async fn update_account(
    State(repo): State<DynAccountRepository>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateAccountRequest>,
) -> impl IntoResponse {
    // バリデーション
    if let Err(errors) = request.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                format!("Validation failed: {}", errors),
                "VALIDATION_ERROR",
            )),
        )
            .into_response();
    }

    match repo.update(id, request).await {
        Ok(account) => (StatusCode::OK, Json(AccountResponse::from(account))).into_response(),
        Err(err) => map_repo_error(err).into_response(),
    }
}

/// DELETE /api/accounts/:id - 勘定科目論理削除
pub async fn delete_account(
    State(repo): State<DynAccountRepository>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo.soft_delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_repo_error(err).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AccountCategory;
    use crate::repository::InMemoryAccountRepository;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{delete, get, post, put},
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let repo: DynAccountRepository = Arc::new(InMemoryAccountRepository::new());

        Router::new()
            .route("/api/accounts", post(create_account).get(list_accounts))
            .route(
                "/api/accounts/:id",
                get(get_account).put(update_account).delete(delete_account),
            )
            .with_state(repo)
    }

    #[tokio::test]
    async fn test_create_account_success() {
        let app = create_test_app();

        let request_body = serde_json::json!({
            "code": "101",
            "name": "現金",
            "category": "cash",
            "description": "手許現金"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/accounts")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let account: AccountResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(account.code, "101");
        assert_eq!(account.name, "現金");
        assert_eq!(account.account_type, AccountType::Asset);
    }

    #[tokio::test]
    async fn test_create_account_validation_error() {
        let app = create_test_app();

        let request_body = serde_json::json!({
            "code": "x",  // 3文字未満
            "name": "",   // 空
            "category": "cash"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/accounts")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let repo = Arc::new(InMemoryAccountRepository::new());

        // テストデータ作成
        let _ = repo
            .create(CreateAccountRequest {
                code: "101".to_string(),
                name: "現金".to_string(),
                category: AccountCategory::Cash,
                description: None,
                display_order: Some(1),
            })
            .await
            .unwrap();

        let app = Router::new()
            .route("/api/accounts", get(list_accounts))
            .with_state(repo as DynAccountRepository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/accounts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let accounts: Vec<AccountResponse> = serde_json::from_slice(&body).unwrap();

        assert_eq!(accounts.len(), 1);
    }

    #[tokio::test]
    async fn test_list_accounts_by_type() {
        let repo = Arc::new(InMemoryAccountRepository::new());

        // 資産と収入を作成
        let _ = repo
            .create(CreateAccountRequest {
                code: "101".to_string(),
                name: "現金".to_string(),
                category: AccountCategory::Cash,
                description: None,
                display_order: Some(1),
            })
            .await
            .unwrap();
        let _ = repo
            .create(CreateAccountRequest {
                code: "401".to_string(),
                name: "什一献金".to_string(),
                category: AccountCategory::TitheOffering,
                description: None,
                display_order: Some(10),
            })
            .await
            .unwrap();

        let app = Router::new()
            .route("/api/accounts", get(list_accounts))
            .with_state(repo as DynAccountRepository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/accounts?account_type=asset")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let accounts: Vec<AccountResponse> = serde_json::from_slice(&body).unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].code, "101");
    }

    #[tokio::test]
    async fn test_get_account_not_found() {
        let app = create_test_app();
        let random_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/accounts/{}", random_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_account() {
        let repo = Arc::new(InMemoryAccountRepository::new());

        let created = repo
            .create(CreateAccountRequest {
                code: "101".to_string(),
                name: "現金".to_string(),
                category: AccountCategory::Cash,
                description: None,
                display_order: Some(1),
            })
            .await
            .unwrap();

        let app = Router::new()
            .route("/api/accounts/:id", put(update_account))
            .with_state(repo as DynAccountRepository);

        let update_body = serde_json::json!({
            "name": "小口現金",
            "description": "小口経費用"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/accounts/{}", created.id))
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let updated: AccountResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(updated.name, "小口現金");
    }

    #[tokio::test]
    async fn test_delete_account() {
        let repo = Arc::new(InMemoryAccountRepository::new());

        let created = repo
            .create(CreateAccountRequest {
                code: "101".to_string(),
                name: "現金".to_string(),
                category: AccountCategory::Cash,
                description: None,
                display_order: Some(1),
            })
            .await
            .unwrap();

        let app = Router::new()
            .route("/api/accounts/:id", delete(delete_account))
            .with_state(repo.clone() as DynAccountRepository);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/accounts/{}", created.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // 論理削除確認
        let account = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert!(!account.is_active);
    }
}
