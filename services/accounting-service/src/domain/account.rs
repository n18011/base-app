use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 勘定科目の種別（5要素）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 資産
    Asset,
    /// 負債
    Liability,
    /// 純資産
    Equity,
    /// 収入
    Revenue,
    /// 支出
    Expense,
}

impl AccountType {
    /// 借方（Debit）で増加する科目か
    pub fn is_debit_increase(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }

    /// 貸方（Credit）で増加する科目か
    pub fn is_credit_increase(&self) -> bool {
        !self.is_debit_increase()
    }
}

/// 教会会計向け勘定科目カテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountCategory {
    // 資産
    Cash,
    BankDeposit,
    FixedDeposit,
    AccountsReceivable,

    // 負債
    AccountsPayable,
    DepositsReceived,
    Borrowings,

    // 純資産
    Capital,
    RetainedSurplus,

    // 収入（献金関連）
    TitheOffering,
    ThankOffering,
    SpecialOffering,
    BuildingOffering,
    InterestIncome,
    OtherRevenue,

    // 支出
    PersonnelExpense,
    UtilityExpense,
    CommunicationExpense,
    SuppliesExpense,
    WorshipExpense,
    EducationExpense,
    MissionExpense,
    MaintenanceExpense,
    OtherExpense,
}

impl AccountCategory {
    /// このカテゴリが属する勘定科目種別を返す
    pub fn account_type(&self) -> AccountType {
        match self {
            AccountCategory::Cash
            | AccountCategory::BankDeposit
            | AccountCategory::FixedDeposit
            | AccountCategory::AccountsReceivable => AccountType::Asset,

            AccountCategory::AccountsPayable
            | AccountCategory::DepositsReceived
            | AccountCategory::Borrowings => AccountType::Liability,

            AccountCategory::Capital | AccountCategory::RetainedSurplus => AccountType::Equity,

            AccountCategory::TitheOffering
            | AccountCategory::ThankOffering
            | AccountCategory::SpecialOffering
            | AccountCategory::BuildingOffering
            | AccountCategory::InterestIncome
            | AccountCategory::OtherRevenue => AccountType::Revenue,

            AccountCategory::PersonnelExpense
            | AccountCategory::UtilityExpense
            | AccountCategory::CommunicationExpense
            | AccountCategory::SuppliesExpense
            | AccountCategory::WorshipExpense
            | AccountCategory::EducationExpense
            | AccountCategory::MissionExpense
            | AccountCategory::MaintenanceExpense
            | AccountCategory::OtherExpense => AccountType::Expense,
        }
    }
}

/// 勘定科目エンティティ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub category: AccountCategory,
    pub description: Option<String>,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(
        code: String,
        name: String,
        category: AccountCategory,
        description: Option<String>,
        display_order: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            code,
            name,
            account_type: category.account_type(),
            category,
            description,
            is_active: true,
            display_order,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 勘定科目作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 3, max = 10, message = "科目コードは3〜10文字で入力してください"))]
    #[validate(regex(
        path = *CODE_REGEX,
        message = "科目コードは英数字とハイフンのみ使用できます"
    ))]
    pub code: String,

    #[validate(length(min = 1, max = 100, message = "科目名は1〜100文字で入力してください"))]
    pub name: String,

    pub category: AccountCategory,

    #[validate(length(max = 500, message = "説明は500文字以内で入力してください"))]
    pub description: Option<String>,

    pub display_order: Option<i32>,
}

lazy_static::lazy_static! {
    static ref CODE_REGEX: regex::Regex = regex::Regex::new(r"^[A-Za-z0-9\-]+$").unwrap();
}

/// 勘定科目更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateAccountRequest {
    #[validate(length(min = 1, max = 100, message = "科目名は1〜100文字で入力してください"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "説明は500文字以内で入力してください"))]
    pub description: Option<String>,

    pub display_order: Option<i32>,

    pub is_active: Option<bool>,
}

/// 勘定科目レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub category: AccountCategory,
    pub description: Option<String>,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            code: account.code,
            name: account.name,
            account_type: account.account_type,
            category: account.category,
            description: account.description,
            is_active: account.is_active,
            display_order: account.display_order,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_debit_credit() {
        assert!(AccountType::Asset.is_debit_increase());
        assert!(AccountType::Expense.is_debit_increase());
        assert!(!AccountType::Liability.is_debit_increase());
        assert!(!AccountType::Equity.is_debit_increase());
        assert!(!AccountType::Revenue.is_debit_increase());

        assert!(AccountType::Liability.is_credit_increase());
        assert!(AccountType::Equity.is_credit_increase());
        assert!(AccountType::Revenue.is_credit_increase());
    }

    #[test]
    fn test_account_category_type_mapping() {
        assert_eq!(AccountCategory::Cash.account_type(), AccountType::Asset);
        assert_eq!(
            AccountCategory::AccountsPayable.account_type(),
            AccountType::Liability
        );
        assert_eq!(AccountCategory::Capital.account_type(), AccountType::Equity);
        assert_eq!(
            AccountCategory::TitheOffering.account_type(),
            AccountType::Revenue
        );
        assert_eq!(
            AccountCategory::PersonnelExpense.account_type(),
            AccountType::Expense
        );
    }

    #[test]
    fn test_account_new() {
        let account = Account::new(
            "101".to_string(),
            "現金".to_string(),
            AccountCategory::Cash,
            Some("手許現金".to_string()),
            1,
        );

        assert_eq!(account.code, "101");
        assert_eq!(account.name, "現金");
        assert_eq!(account.account_type, AccountType::Asset);
        assert_eq!(account.category, AccountCategory::Cash);
        assert!(account.is_active);
    }
}
