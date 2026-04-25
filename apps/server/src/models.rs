use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use whaleit_core::accounts as core_accounts;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    pub is_archived: bool,
    pub tracking_mode: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub institution: Option<String>,
    #[schema(value_type = Option<String>)]
    pub opening_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub current_balance: Option<Decimal>,
    pub balance_updated_at: Option<NaiveDateTime>,
    #[schema(value_type = Option<String>)]
    pub credit_limit: Option<Decimal>,
    pub statement_cycle_day: Option<i16>,
    #[schema(value_type = Option<String>)]
    pub statement_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub minimum_payment: Option<Decimal>,
    pub statement_due_date: Option<NaiveDate>,
    pub reward_points_balance: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub cashback_balance: Option<Decimal>,
}

impl From<core_accounts::Account> for Account {
    fn from(a: core_accounts::Account) -> Self {
        let tracking_mode = match a.tracking_mode {
            core_accounts::TrackingMode::Transactions => "TRANSACTIONS",
            core_accounts::TrackingMode::Holdings => "HOLDINGS",
            core_accounts::TrackingMode::NotSet => "NOT_SET",
        }
        .to_string();
        Self {
            id: a.id,
            name: a.name,
            account_type: a.account_type,
            group: a.group,
            currency: a.currency,
            is_default: a.is_default,
            is_active: a.is_active,
            is_archived: a.is_archived,
            tracking_mode,
            created_at: a.created_at,
            updated_at: a.updated_at,
            platform_id: a.platform_id,
            account_number: a.account_number,
            meta: a.meta,
            provider: a.provider,
            provider_account_id: a.provider_account_id,
            institution: a.institution,
            opening_balance: a.opening_balance,
            current_balance: a.current_balance,
            balance_updated_at: a.balance_updated_at,
            credit_limit: a.credit_limit,
            statement_cycle_day: a.statement_cycle_day,
            statement_balance: a.statement_balance,
            minimum_payment: a.minimum_payment,
            statement_due_date: a.statement_due_date,
            reward_points_balance: a.reward_points_balance,
            cashback_balance: a.cashback_balance,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default = "default_tracking_mode")]
    pub tracking_mode: String,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub institution: Option<String>,
    #[schema(value_type = Option<String>)]
    pub opening_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub current_balance: Option<Decimal>,
    pub balance_updated_at: Option<NaiveDateTime>,
    #[schema(value_type = Option<String>)]
    pub credit_limit: Option<Decimal>,
    pub statement_cycle_day: Option<i16>,
    #[schema(value_type = Option<String>)]
    pub statement_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub minimum_payment: Option<Decimal>,
    pub statement_due_date: Option<NaiveDate>,
    pub reward_points_balance: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub cashback_balance: Option<Decimal>,
}

fn default_tracking_mode() -> String {
    "NOT_SET".to_string()
}

fn parse_tracking_mode(s: &str) -> core_accounts::TrackingMode {
    match s {
        "TRANSACTIONS" => core_accounts::TrackingMode::Transactions,
        "HOLDINGS" => core_accounts::TrackingMode::Holdings,
        _ => core_accounts::TrackingMode::NotSet,
    }
}

impl From<NewAccount> for core_accounts::NewAccount {
    fn from(a: NewAccount) -> Self {
        Self {
            id: a.id,
            name: a.name,
            account_type: a.account_type,
            group: a.group,
            currency: a.currency,
            is_default: a.is_default,
            is_active: a.is_active,
            is_archived: a.is_archived,
            tracking_mode: parse_tracking_mode(&a.tracking_mode),
            platform_id: a.platform_id,
            account_number: a.account_number,
            meta: a.meta,
            provider: a.provider,
            provider_account_id: a.provider_account_id,
            institution: a.institution,
            opening_balance: a.opening_balance,
            current_balance: a.current_balance,
            balance_updated_at: None, // D-12: server-only field, client value discarded
            credit_limit: a.credit_limit,
            statement_cycle_day: a.statement_cycle_day,
            statement_balance: a.statement_balance,
            minimum_payment: a.minimum_payment,
            statement_due_date: a.statement_due_date,
            reward_points_balance: a.reward_points_balance,
            cashback_balance: a.cashback_balance,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdate {
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub is_archived: Option<bool>,
    pub tracking_mode: Option<String>,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub institution: Option<String>,
    #[schema(value_type = Option<String>)]
    pub opening_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub current_balance: Option<Decimal>,
    pub balance_updated_at: Option<NaiveDateTime>,
    #[schema(value_type = Option<String>)]
    pub credit_limit: Option<Decimal>,
    pub statement_cycle_day: Option<i16>,
    #[schema(value_type = Option<String>)]
    pub statement_balance: Option<Decimal>,
    #[schema(value_type = Option<String>)]
    pub minimum_payment: Option<Decimal>,
    pub statement_due_date: Option<NaiveDate>,
    pub reward_points_balance: Option<i32>,
    #[schema(value_type = Option<String>)]
    pub cashback_balance: Option<Decimal>,
}

impl From<AccountUpdate> for core_accounts::AccountUpdate {
    fn from(a: AccountUpdate) -> Self {
        Self {
            id: a.id,
            name: a.name,
            account_type: a.account_type,
            group: a.group,
            is_default: a.is_default,
            is_active: a.is_active,
            is_archived: a.is_archived,
            tracking_mode: a.tracking_mode.map(|s| parse_tracking_mode(&s)),
            platform_id: a.platform_id,
            account_number: a.account_number,
            meta: a.meta,
            provider: a.provider,
            provider_account_id: a.provider_account_id,
            institution: a.institution,
            opening_balance: a.opening_balance,
            current_balance: a.current_balance,
            balance_updated_at: None, // D-12: server-only field, client value discarded
            credit_limit: a.credit_limit,
            statement_cycle_day: a.statement_cycle_day,
            statement_balance: a.statement_balance,
            minimum_payment: a.minimum_payment,
            statement_due_date: a.statement_due_date,
            reward_points_balance: a.reward_points_balance,
            cashback_balance: a.cashback_balance,
        }
    }
}
