use std::{
    collections::HashMap,
    path::Path as StdPath,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::{
    api::shared::{process_portfolio_job, PortfolioJobConfig},
    error::ApiResult,
    main_lib::AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use reqwest::StatusCode as HttpStatusCode;
use semver::Version;
use serde::Deserialize;
use tokio::fs;
use whaleit_core::{
    portfolio::{snapshot::SnapshotRecalcMode, valuation::ValuationRecalcMode},
    quotes::MarketSyncMode,
    settings::{Settings, SettingsServiceTrait, SettingsUpdate},
};

async fn get_settings(State(state): State<Arc<AppState>>) -> ApiResult<Json<Settings>> {
    let s = state.settings_service.get_settings().await?;
    Ok(Json(s))
}

async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SettingsUpdate>,
) -> ApiResult<Json<Settings>> {
    let previous_base_currency = state.base_currency.read().unwrap().clone();
    let previous_timezone = state.timezone.read().unwrap().clone();
    state.settings_service.update_settings(&payload).await?;
    let updated_settings = state.settings_service.get_settings().await?;
    *state.timezone.write().unwrap() = updated_settings.timezone.clone();
    state.health_service.clear_cache().await;

    let base_currency_changed = updated_settings.base_currency != previous_base_currency;
    let timezone_changed = updated_settings.timezone != previous_timezone;

    if base_currency_changed {
        *state.base_currency.write().unwrap() = updated_settings.base_currency.clone();

        let state_for_job = state.clone();
        tokio::spawn(async move {
            let job_config = PortfolioJobConfig {
                account_ids: None,
                market_sync_mode: MarketSyncMode::BackfillHistory {
                    asset_ids: None,
                    days: whaleit_core::quotes::DEFAULT_HISTORY_DAYS,
                },
                snapshot_mode: SnapshotRecalcMode::Full,
                valuation_mode: ValuationRecalcMode::Full,
                since_date: None,
            };

            if let Err(err) = process_portfolio_job(state_for_job, job_config).await {
                tracing::warn!("Base currency change recalculation failed: {}", err);
            }
        });
    } else if timezone_changed {
        let state_for_job = state.clone();
        tokio::spawn(async move {
            let job_config = PortfolioJobConfig {
                account_ids: None,
                market_sync_mode: MarketSyncMode::None,
                snapshot_mode: SnapshotRecalcMode::Full,
                valuation_mode: ValuationRecalcMode::Full,
                since_date: None,
            };

            if let Err(err) = process_portfolio_job(state_for_job, job_config).await {
                tracing::warn!("Timezone change recalculation failed: {}", err);
            }
        });
    }

    Ok(Json(updated_settings))
}

async fn is_auto_update_check_enabled(State(state): State<Arc<AppState>>) -> ApiResult<Json<bool>> {
    let enabled = state
        .settings_service
        .is_auto_update_check_enabled()
        .await
        .unwrap_or(true);
    Ok(Json(enabled))
}

const WEB_RUNTIME_TARGET: &str = "web-docker";

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfoResponse {
    version: String,
    database_url: String,
    logs_dir: String,
}

async fn get_app_info(State(state): State<Arc<AppState>>) -> ApiResult<Json<AppInfoResponse>> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let database_url = state.database_url.clone();
    let logs_dir = std::env::var("WF_LOGS_DIR").unwrap_or_else(|_| {
        StdPath::new(&state.data_root)
            .join("logs")
            .to_str()
            .unwrap_or("")
            .to_string()
    });

    Ok(Json(AppInfoResponse {
        version,
        database_url,
        logs_dir,
    }))
}

#[derive(Deserialize)]
struct UpdatePlatformInfo {
    url: Option<String>,
}

#[derive(Deserialize)]
struct UpdateCheckResponseRaw {
    version: String,
    notes: Option<String>,
    pub_date: Option<String>,
    platforms: HashMap<String, UpdatePlatformInfo>,
    changelog_url: Option<String>,
    screenshots: Option<Vec<String>>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckResponse {
    update_available: bool,
    latest_version: String,
    notes: Option<String>,
    pub_date: Option<String>,
    download_url: Option<String>,
    changelog_url: Option<String>,
    screenshots: Option<Vec<String>>,
}

static UPDATE_CACHE: std::sync::LazyLock<Mutex<Option<(Instant, UpdateCheckResponse)>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
const UPDATE_CACHE_TTL: Duration = Duration::from_secs(60 * 60);

fn normalize_target(target: Option<String>) -> String {
    match target
        .or_else(|| Some(WEB_RUNTIME_TARGET.to_string()))
        .unwrap_or_else(|| WEB_RUNTIME_TARGET.to_string())
        .to_lowercase()
        .as_str()
    {
        "macos" | "darwin" => "darwin".to_string(),
        "windows" | "win32" => "windows".to_string(),
        "linux" => "linux".to_string(),
        "web-docker" => WEB_RUNTIME_TARGET.to_string(),
        other => other.to_string(),
    }
}

fn normalize_arch(arch: Option<String>) -> String {
    match arch
        .or_else(|| Some(std::env::consts::ARCH.to_string()))
        .unwrap_or_else(|| "x86_64".to_string())
        .to_lowercase()
        .as_str()
    {
        "arm64" | "aarch64" => "aarch64".to_string(),
        "x86_64" | "x64" | "amd64" => "x86_64".to_string(),
        other => other.to_string(),
    }
}

#[derive(Deserialize)]
struct CheckUpdateQuery {
    #[serde(default)]
    force: bool,
}

async fn check_update(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<CheckUpdateQuery>,
) -> ApiResult<Json<UpdateCheckResponse>> {
    if !query.force {
        let cache = UPDATE_CACHE.lock().await;
        if let Some((cached_at, ref response)) = *cache {
            if cached_at.elapsed() < UPDATE_CACHE_TTL {
                return Ok(Json(response.clone()));
            }
        }
    }

    let current_version_str = env!("CARGO_PKG_VERSION").to_string();
    let target = normalize_target(None);
    let arch = normalize_arch(None);
    let request_url = format!(
        "https://whaleit.app/releases/{}/{}/{}",
        target, arch, current_version_str
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header("X-Instance-Id", state.instance_id.clone())
        .header("X-Client-Runtime", WEB_RUNTIME_TARGET)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query update endpoint: {e}"))?;

    let result = if response.status() == HttpStatusCode::NOT_FOUND {
        UpdateCheckResponse {
            update_available: false,
            latest_version: current_version_str,
            notes: None,
            pub_date: None,
            download_url: None,
            changelog_url: None,
            screenshots: None,
        }
    } else {
        let payload: UpdateCheckResponseRaw = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse update response: {e}"))?;

        let current_version =
            Version::parse(&current_version_str).unwrap_or_else(|_| Version::new(0, 0, 0));
        let latest_version =
            Version::parse(&payload.version).unwrap_or_else(|_| current_version.clone());
        let update_available = latest_version > current_version;

        let platform_key = format!("{}-{}", target, arch);
        let download_url = payload
            .platforms
            .get(&platform_key)
            .and_then(|p| p.url.clone());

        UpdateCheckResponse {
            update_available,
            latest_version: payload.version,
            notes: payload.notes,
            pub_date: payload.pub_date,
            download_url,
            changelog_url: payload.changelog_url,
            screenshots: payload.screenshots,
        }
    };

    {
        let mut cache = UPDATE_CACHE.lock().await;
        *cache = Some((Instant::now(), result.clone()));
    }

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct BackupBody {
    #[serde(default)]
    backup_dir: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupDatabaseResponse {
    filename: String,
    data_b64: String,
}

async fn backup_database_route(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<BackupDatabaseResponse>> {
    let database_url = state.database_url.clone();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("whaleit_backup_{}.sql", timestamp);
    let backup_path = StdPath::new(&state.data_root).join(&filename);

    let backup_path_str = backup_path.to_str().unwrap_or(&filename).to_string();
    let output = tokio::process::Command::new("pg_dump")
        .arg(&database_url)
        .arg("--no-owner")
        .arg("--no-acl")
        .arg("--clean")
        .arg("--if-exists")
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("pg_dump failed: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "pg_dump failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    fs::write(&backup_path_str, &output.stdout)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write backup file: {}", e))?;

    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    let data_b64 = BASE64.encode(&output.stdout);
    Ok(Json(BackupDatabaseResponse { filename, data_b64 }))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupToPathBody {
    backup_dir: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupToPathResponse {
    path: String,
}

async fn backup_database_to_path_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BackupToPathBody>,
) -> ApiResult<Json<BackupToPathResponse>> {
    let database_url = state.database_url.clone();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("whaleit_backup_{}.sql", timestamp);
    let backup_path = StdPath::new(&body.backup_dir).join(&filename);
    let backup_path_str = backup_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid backup path"))?
        .to_string();

    let output = tokio::process::Command::new("pg_dump")
        .arg(&database_url)
        .arg("--no-owner")
        .arg("--no-acl")
        .arg("--clean")
        .arg("--if-exists")
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("pg_dump failed: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "pg_dump failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    fs::write(&backup_path_str, &output.stdout)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write backup file: {}", e))?;
    Ok(Json(BackupToPathResponse { path: backup_path_str }))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreBody {
    backup_file_path: String,
}

async fn restore_database_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RestoreBody>,
) -> ApiResult<StatusCode> {
    let database_url = state.database_url.clone();
    let sql_content = fs::read_to_string(&body.backup_file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read backup file: {}", e))?;

    let output = tokio::process::Command::new("psql")
        .arg(&database_url)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("psql failed to start: {}", e))?
        .wait_with_output()
        .await
        .map_err(|e| anyhow::anyhow!("psql failed: {}", e))?;

    drop(sql_content);

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "psql restore failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/settings", get(get_settings).put(update_settings))
        .route(
            "/settings/auto-update-enabled",
            get(is_auto_update_check_enabled),
        )
        .route("/app/info", get(get_app_info))
        .route("/app/check-update", get(check_update))
        .route("/utilities/database/backup", post(backup_database_route))
        .route(
            "/utilities/database/backup-to-path",
            post(backup_database_to_path_route),
        )
        .route("/utilities/database/restore", post(restore_database_route))
}
