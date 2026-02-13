CREATE TABLE IF NOT EXISTS accounts (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    code            VARCHAR(10)     NOT NULL UNIQUE,
    name            VARCHAR(100)    NOT NULL,
    account_type    VARCHAR(20)     NOT NULL,
    category        VARCHAR(30)     NOT NULL,
    description     TEXT,
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    display_order   INTEGER         NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_accounts_account_type ON accounts (account_type);
CREATE INDEX IF NOT EXISTS idx_accounts_type_order ON accounts (account_type, display_order);

ALTER TABLE accounts ADD CONSTRAINT chk_account_type
    CHECK (account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense'));

ALTER TABLE accounts ADD CONSTRAINT chk_category
    CHECK (category IN (
        'cash', 'bank_deposit', 'fixed_deposit', 'accounts_receivable',
        'accounts_payable', 'deposits_received', 'borrowings',
        'capital', 'retained_surplus',
        'tithe_offering', 'thank_offering', 'special_offering', 'building_offering',
        'interest_income', 'other_revenue',
        'personnel_expense', 'utility_expense', 'communication_expense', 'supplies_expense',
        'worship_expense', 'education_expense', 'mission_expense', 'maintenance_expense',
        'other_expense'
    ));
