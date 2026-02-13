use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
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

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Revenue => "revenue",
            AccountType::Expense => "expense",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccountType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(AccountType::Asset),
            "liability" => Ok(AccountType::Liability),
            "equity" => Ok(AccountType::Equity),
            "revenue" => Ok(AccountType::Revenue),
            "expense" => Ok(AccountType::Expense),
            other => Err(format!("Invalid account type: {}", other)),
        }
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

impl fmt::Display for AccountCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AccountCategory::Cash => "cash",
            AccountCategory::BankDeposit => "bank_deposit",
            AccountCategory::FixedDeposit => "fixed_deposit",
            AccountCategory::AccountsReceivable => "accounts_receivable",
            AccountCategory::AccountsPayable => "accounts_payable",
            AccountCategory::DepositsReceived => "deposits_received",
            AccountCategory::Borrowings => "borrowings",
            AccountCategory::Capital => "capital",
            AccountCategory::RetainedSurplus => "retained_surplus",
            AccountCategory::TitheOffering => "tithe_offering",
            AccountCategory::ThankOffering => "thank_offering",
            AccountCategory::SpecialOffering => "special_offering",
            AccountCategory::BuildingOffering => "building_offering",
            AccountCategory::InterestIncome => "interest_income",
            AccountCategory::OtherRevenue => "other_revenue",
            AccountCategory::PersonnelExpense => "personnel_expense",
            AccountCategory::UtilityExpense => "utility_expense",
            AccountCategory::CommunicationExpense => "communication_expense",
            AccountCategory::SuppliesExpense => "supplies_expense",
            AccountCategory::WorshipExpense => "worship_expense",
            AccountCategory::EducationExpense => "education_expense",
            AccountCategory::MissionExpense => "mission_expense",
            AccountCategory::MaintenanceExpense => "maintenance_expense",
            AccountCategory::OtherExpense => "other_expense",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccountCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cash" => Ok(AccountCategory::Cash),
            "bank_deposit" => Ok(AccountCategory::BankDeposit),
            "fixed_deposit" => Ok(AccountCategory::FixedDeposit),
            "accounts_receivable" => Ok(AccountCategory::AccountsReceivable),
            "accounts_payable" => Ok(AccountCategory::AccountsPayable),
            "deposits_received" => Ok(AccountCategory::DepositsReceived),
            "borrowings" => Ok(AccountCategory::Borrowings),
            "capital" => Ok(AccountCategory::Capital),
            "retained_surplus" => Ok(AccountCategory::RetainedSurplus),
            "tithe_offering" => Ok(AccountCategory::TitheOffering),
            "thank_offering" => Ok(AccountCategory::ThankOffering),
            "special_offering" => Ok(AccountCategory::SpecialOffering),
            "building_offering" => Ok(AccountCategory::BuildingOffering),
            "interest_income" => Ok(AccountCategory::InterestIncome),
            "other_revenue" => Ok(AccountCategory::OtherRevenue),
            "personnel_expense" => Ok(AccountCategory::PersonnelExpense),
            "utility_expense" => Ok(AccountCategory::UtilityExpense),
            "communication_expense" => Ok(AccountCategory::CommunicationExpense),
            "supplies_expense" => Ok(AccountCategory::SuppliesExpense),
            "worship_expense" => Ok(AccountCategory::WorshipExpense),
            "education_expense" => Ok(AccountCategory::EducationExpense),
            "mission_expense" => Ok(AccountCategory::MissionExpense),
            "maintenance_expense" => Ok(AccountCategory::MaintenanceExpense),
            "other_expense" => Ok(AccountCategory::OtherExpense),
            other => Err(format!("Invalid account category: {}", other)),
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
    fn test_account_type_display_and_from_str() {
        let types = vec![
            (AccountType::Asset, "asset"),
            (AccountType::Liability, "liability"),
            (AccountType::Equity, "equity"),
            (AccountType::Revenue, "revenue"),
            (AccountType::Expense, "expense"),
        ];
        for (variant, expected) in types {
            assert_eq!(variant.to_string(), expected);
            assert_eq!(AccountType::from_str(expected).unwrap(), variant);
        }
        assert!(AccountType::from_str("invalid").is_err());
    }

    #[test]
    fn test_account_category_display_and_from_str() {
        let categories = vec![
            (AccountCategory::Cash, "cash"),
            (AccountCategory::BankDeposit, "bank_deposit"),
            (AccountCategory::TitheOffering, "tithe_offering"),
            (AccountCategory::PersonnelExpense, "personnel_expense"),
            (AccountCategory::OtherExpense, "other_expense"),
        ];
        for (variant, expected) in categories {
            assert_eq!(variant.to_string(), expected);
            assert_eq!(AccountCategory::from_str(expected).unwrap(), variant);
        }
        assert!(AccountCategory::from_str("invalid").is_err());
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
