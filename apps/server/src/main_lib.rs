use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{
    ai_environment::ServerAiEnvironment, auth::AuthManager, config::Config,
    domain_events::WebDomainEventSink, email::EmailService, events::EventBus,
    secrets::build_secret_store,
};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};
use whaleit_ai::{AiProviderService, AiProviderServiceTrait, ChatConfig, ChatService};
use whaleit_connect::{
    BrokerSyncService, BrokerSyncServiceTrait, CoreImportRunRepositoryAdapter,
    ImportRunRepositoryTrait, TokenLifecycleState,
};
use whaleit_core::addons::{AddonService, AddonServiceTrait};
use whaleit_core::{
    accounts::AccountService,
    activities::{ActivityService as CoreActivityService, ActivityServiceTrait},
    assets::{
        AlternativeAssetRepositoryTrait, AlternativeAssetService, AlternativeAssetServiceTrait,
        AssetClassificationService, AssetService, AssetServiceTrait,
    },
    events::DomainEventSink,
    fx::{FxService, FxServiceTrait},
    goals::{GoalService, GoalServiceTrait},
    health::{HealthService, HealthServiceTrait},
    limits::{ContributionLimitService, ContributionLimitServiceTrait},
    portfolio::allocation::{AllocationService, AllocationServiceTrait},
    portfolio::income::{IncomeService, IncomeServiceTrait},
    portfolio::{
        holdings::{
            holdings_valuation_service::HoldingsValuationService, HoldingsService,
            HoldingsServiceTrait,
        },
        net_worth::{NetWorthService, NetWorthServiceTrait},
        snapshot::{SnapshotService, SnapshotServiceTrait},
        valuation::{ValuationService, ValuationServiceTrait},
    },
    quotes::{QuoteService, QuoteServiceTrait},
    secrets::SecretStore,
    settings::{SettingsRepositoryTrait, SettingsService, SettingsServiceTrait},
    taxonomies::{TaxonomyService, TaxonomyServiceTrait},
    users::UserRepositoryTrait,
};
use whaleit_device_sync::{engine::DeviceSyncRuntimeState, DeviceEnrollService};
use whaleit_storage_postgres::{
    accounts::PgAccountRepository,
    activities::PgActivityRepository,
    ai_chat::PgAiChatRepository,
    assets::{PgAlternativeAssetRepository, PgAssetRepository},
    custom_provider::PgCustomProviderRepository,
    db::{self},
    fx::PgFxRepository,
    goals::PgGoalRepository,
    health::PgHealthDismissalRepository,
    limits::PgContributionLimitRepository,
    market_data::{PgMarketDataRepository, PgQuoteSyncStateRepository},
    portfolio::{PgSnapshotRepository, PgValuationRepository},
    settings::PgSettingsRepository,
    sync::{
        PgAppSyncRepository, PgBrokerSyncStateRepository, PgImportRunRepository,
        PgPlatformRepository,
    },
    taxonomies::PgTaxonomyRepository,
    users::PgUserRepository,
    AppSyncRepository, SnapshotRepository,
};

pub struct AppState {
    #[allow(dead_code)]
    pub domain_event_sink: Arc<dyn DomainEventSink>,
    pub account_service: Arc<AccountService>,
    pub settings_service: Arc<SettingsService>,
    pub holdings_service: Arc<dyn HoldingsServiceTrait + Send + Sync>,
    pub valuation_service: Arc<dyn ValuationServiceTrait + Send + Sync>,
    pub allocation_service: Arc<dyn AllocationServiceTrait + Send + Sync>,
    pub quote_service: Arc<dyn QuoteServiceTrait + Send + Sync>,
    pub base_currency: Arc<RwLock<String>>,
    pub timezone: Arc<RwLock<String>>,
    pub snapshot_service: Arc<dyn SnapshotServiceTrait + Send + Sync>,
    pub snapshot_repository: Arc<SnapshotRepository>,
    pub performance_service:
        Arc<dyn whaleit_core::portfolio::performance::PerformanceServiceTrait + Send + Sync>,
    pub income_service: Arc<dyn IncomeServiceTrait + Send + Sync>,
    pub goal_service: Arc<dyn GoalServiceTrait + Send + Sync>,
    pub limits_service: Arc<dyn ContributionLimitServiceTrait + Send + Sync>,
    pub fx_service: Arc<dyn FxServiceTrait + Send + Sync>,
    pub activity_service: Arc<dyn ActivityServiceTrait + Send + Sync>,
    pub asset_service: Arc<dyn AssetServiceTrait + Send + Sync>,
    pub taxonomy_service: Arc<dyn TaxonomyServiceTrait + Send + Sync>,
    pub net_worth_service: Arc<dyn NetWorthServiceTrait + Send + Sync>,
    pub alternative_asset_service: Arc<dyn AlternativeAssetServiceTrait + Send + Sync>,
    pub addon_service: Arc<dyn AddonServiceTrait + Send + Sync>,
    pub connect_sync_service: Arc<dyn BrokerSyncServiceTrait + Send + Sync>,
    pub ai_provider_service: Arc<dyn AiProviderServiceTrait + Send + Sync>,
    pub ai_chat_service: Arc<ChatService<ServerAiEnvironment>>,
    pub data_root: String,
    pub database_url: String,
    pub instance_id: String,
    pub secret_store: Arc<dyn SecretStore>,
    pub event_bus: EventBus,
    pub auth: Option<Arc<AuthManager>>,
    pub user_repo: Option<Arc<dyn UserRepositoryTrait>>,
    pub email: Option<Arc<EmailService>>,
    pub device_enroll_service: Arc<DeviceEnrollService>,
    pub app_sync_repository: Arc<AppSyncRepository>,
    pub device_sync_runtime: Arc<DeviceSyncRuntimeState>,
    pub health_service: Arc<dyn HealthServiceTrait + Send + Sync>,
    pub token_lifecycle: Arc<TokenLifecycleState>,
    pub custom_provider_service: Arc<whaleit_core::custom_provider::CustomProviderService>,
}

pub fn init_tracing() {
    let log_format = std::env::var("WF_LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry().with(filter);

    if log_format.eq_ignore_ascii_case("json") {
        registry
            .with(fmt::layer().json().with_current_span(false))
            .init();
    } else {
        registry
            .with(fmt::layer().with_target(true).with_line_number(true))
            .init();
    }
}

pub async fn build_state(config: &Config) -> anyhow::Result<Arc<AppState>> {
    let database_url = config
        .database_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required"))?;
    build_state_postgres(config, database_url).await
}

async fn build_state_postgres(
    config: &Config,
    database_url: &str,
) -> anyhow::Result<Arc<AppState>> {
    db::init(database_url).await?;
    tracing::info!("PostgreSQL database connected");

    let data_root_path = std::path::Path::new("./data");
    std::fs::create_dir_all(&data_root_path)
        .map_err(|e| anyhow::anyhow!("Failed to create data root directory: {}", e))?;

    let resolved_secret_path = std::env::var("WF_SECRET_FILE")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_root_path.join("secrets.json"));
    let file_store = build_secret_store(
        resolved_secret_path.clone(),
        Some(config.secrets_encryption_key),
        Some(&config.raw_secret_key),
    )
    .map_err(anyhow::Error::new)?;
    let secret_store: Arc<dyn SecretStore> = Arc::new(file_store);
    std::env::set_var(
        "WF_SECRET_FILE",
        resolved_secret_path.to_string_lossy().to_string(),
    );

    db::run_migrations(database_url).await?;

    let pool = db::create_pool(database_url, config.pg_pool_size)?;

    let domain_event_sink = Arc::new(WebDomainEventSink::new());

    let fx_repo = Arc::new(PgFxRepository::new(pool.clone()));
    let fx_service = Arc::new(FxService::new(fx_repo).with_event_sink(domain_event_sink.clone()));
    fx_service.initialize().await?;

    let settings_repo = Arc::new(PgSettingsRepository::new(pool.clone()));
    let settings_service = Arc::new(SettingsService::new(
        settings_repo.clone(),
        fx_service.clone(),
    ));
    let settings = settings_service.get_settings().await?;
    let base_currency = Arc::new(RwLock::new(settings.base_currency));
    let timezone = Arc::new(RwLock::new(settings.timezone.clone()));

    let account_repo = Arc::new(PgAccountRepository::new(pool.clone()));

    let asset_repository = Arc::new(PgAssetRepository::new(pool.clone()));
    let market_data_repository = Arc::new(PgMarketDataRepository::new(pool.clone()));
    let activity_repository = Arc::new(PgActivityRepository::new(pool.clone()));
    let snapshot_repository = Arc::new(PgSnapshotRepository::new(pool.clone()));
    let app_sync_repository = Arc::new(PgAppSyncRepository::new(pool.clone()));
    let quote_sync_state_repository = Arc::new(PgQuoteSyncStateRepository::new(pool.clone()));

    let account_service = Arc::new(AccountService::new(
        account_repo.clone(),
        fx_service.clone(),
        base_currency.clone(),
        domain_event_sink.clone(),
        asset_repository.clone(),
        quote_sync_state_repository.clone(),
    ));
    let custom_provider_repository = Arc::new(PgCustomProviderRepository::new(pool.clone()));
    let quote_service: Arc<dyn QuoteServiceTrait + Send + Sync> = Arc::new(
        QuoteService::new_with_custom_provider(
            market_data_repository.clone(),
            quote_sync_state_repository.clone(),
            market_data_repository.clone(),
            asset_repository.clone(),
            activity_repository.clone(),
            secret_store.clone(),
            Some(custom_provider_repository.clone()),
        )
        .await?,
    );
    let custom_provider_service =
        Arc::new(whaleit_core::custom_provider::CustomProviderService::new(
            custom_provider_repository.clone(),
            secret_store.clone(),
        ));

    let taxonomy_repository = Arc::new(PgTaxonomyRepository::new(pool.clone()));
    let taxonomy_service = Arc::new(TaxonomyService::new(taxonomy_repository));

    let asset_service = Arc::new(
        AssetService::with_taxonomy_service(
            asset_repository.clone(),
            quote_service.clone(),
            taxonomy_service.clone(),
        )?
        .with_event_sink(domain_event_sink.clone()),
    );
    let snapshot_service = Arc::new(
        SnapshotService::new_with_timezone(
            base_currency.clone(),
            timezone.clone(),
            account_repo.clone(),
            activity_repository.clone(),
            snapshot_repository.clone(),
            asset_repository.clone(),
            fx_service.clone(),
        )
        .with_event_sink(domain_event_sink.clone()),
    );

    let valuation_repository = Arc::new(PgValuationRepository::new(pool.clone()));
    let valuation_service = Arc::new(ValuationService::new(
        base_currency.clone(),
        valuation_repository.clone(),
        snapshot_service.clone(),
        quote_service.clone(),
        fx_service.clone(),
    ));

    let net_worth_service: Arc<dyn NetWorthServiceTrait + Send + Sync> =
        Arc::new(NetWorthService::new(
            base_currency.clone(),
            account_repo.clone(),
            asset_repository.clone(),
            snapshot_repository.clone(),
            quote_service.clone(),
            valuation_repository.clone(),
            fx_service.clone(),
        ));

    let holdings_valuation_service = Arc::new(HoldingsValuationService::new_with_timezone(
        fx_service.clone(),
        quote_service.clone(),
        timezone.clone(),
    ));
    let classification_service =
        Arc::new(AssetClassificationService::new(taxonomy_service.clone()));
    let holdings_service = Arc::new(HoldingsService::new_with_timezone(
        asset_service.clone(),
        snapshot_service.clone(),
        holdings_valuation_service.clone(),
        classification_service.clone(),
        timezone.clone(),
    ));

    let allocation_service: Arc<dyn AllocationServiceTrait + Send + Sync> = Arc::new(
        AllocationService::new(holdings_service.clone(), taxonomy_service.clone()),
    );

    let performance_service = Arc::new(
        whaleit_core::portfolio::performance::PerformanceService::new_with_timezone(
            valuation_service.clone(),
            quote_service.clone(),
            timezone.clone(),
        ),
    );

    let income_service = Arc::new(IncomeService::new_with_timezone(
        fx_service.clone(),
        activity_repository.clone(),
        base_currency.clone(),
        timezone.clone(),
    ));

    let goal_repository = Arc::new(PgGoalRepository::new(pool.clone()));
    let goal_service = Arc::new(GoalService::new(goal_repository));

    let limits_repository = Arc::new(PgContributionLimitRepository::new(pool.clone()));
    let limits_service: Arc<dyn ContributionLimitServiceTrait + Send + Sync> =
        Arc::new(ContributionLimitService::new_with_timezone(
            fx_service.clone(),
            limits_repository.clone(),
            activity_repository.clone(),
            timezone.clone(),
        ));

    let import_run_repository: Arc<dyn ImportRunRepositoryTrait> =
        Arc::new(PgImportRunRepository::new(pool.clone()));
    let core_import_run_repository = Arc::new(CoreImportRunRepositoryAdapter::new(
        import_run_repository.clone(),
    ));
    let broker_sync_state_repository = Arc::new(PgBrokerSyncStateRepository::new(pool.clone()));

    let activity_service: Arc<dyn ActivityServiceTrait + Send + Sync> = Arc::new(
        CoreActivityService::with_import_run_repository(
            activity_repository.clone(),
            account_service.clone(),
            asset_service.clone(),
            fx_service.clone(),
            quote_service.clone(),
            core_import_run_repository,
        )
        .with_event_sink(domain_event_sink.clone()),
    );

    let alternative_asset_repository: Arc<dyn AlternativeAssetRepositoryTrait + Send + Sync> =
        Arc::new(PgAlternativeAssetRepository::new(pool.clone()));

    let alternative_asset_service: Arc<dyn AlternativeAssetServiceTrait + Send + Sync> = Arc::new(
        AlternativeAssetService::new(
            alternative_asset_repository.clone(),
            asset_repository.clone(),
            quote_service.clone(),
        )
        .with_event_sink(domain_event_sink.clone()),
    );

    let platform_repository = Arc::new(PgPlatformRepository::new(pool.clone()));
    let connect_sync_service: Arc<dyn BrokerSyncServiceTrait + Send + Sync> = Arc::new(
        BrokerSyncService::new(
            account_service.clone(),
            asset_service.clone(),
            activity_service.clone(),
            activity_repository.clone(),
            platform_repository,
            broker_sync_state_repository,
            import_run_repository,
            snapshot_repository.clone(),
        )
        .with_event_sink(domain_event_sink.clone())
        .with_snapshot_service(snapshot_service.clone())
        .with_quote_store(market_data_repository.clone()),
    );

    let data_root = data_root_path.to_string_lossy().to_string();

    let ai_catalog_json = include_str!("../../../crates/ai/src/ai_providers.json");
    let ai_provider_service: Arc<dyn AiProviderServiceTrait + Send + Sync> =
        Arc::new(AiProviderService::new(
            settings_repo.clone() as Arc<dyn SettingsRepositoryTrait>,
            secret_store.clone(),
            ai_catalog_json,
        )?);

    let health_dismissal_repository = Arc::new(PgHealthDismissalRepository::new(pool.clone()));
    let health_service: Arc<dyn HealthServiceTrait + Send + Sync> =
        Arc::new(HealthService::new(health_dismissal_repository));

    let ai_chat_repository = Arc::new(PgAiChatRepository::new(pool.clone()));

    let ai_environment = Arc::new(ServerAiEnvironment::new(
        base_currency.clone(),
        account_service.clone(),
        activity_service.clone(),
        holdings_service.clone(),
        valuation_service.clone(),
        goal_service.clone(),
        settings_service.clone(),
        secret_store.clone(),
        ai_chat_repository,
        quote_service.clone(),
        allocation_service.clone(),
        performance_service.clone(),
        income_service.clone(),
        health_service.clone(),
    ));
    let ai_chat_service = Arc::new(ChatService::new(ai_environment, ChatConfig::default()));

    let cloud_api_url = crate::features::cloud_api_base_url().unwrap_or_default();
    let device_display_name = "WhaleIt Server".to_string();
    let app_version = Some(env!("CARGO_PKG_VERSION").to_string());
    let device_enroll_service = Arc::new(DeviceEnrollService::new(
        secret_store.clone(),
        &cloud_api_url,
        device_display_name,
        app_version,
    ));

    let event_bus = EventBus::new(256);
    let device_sync_runtime = Arc::new(DeviceSyncRuntimeState::new());
    let token_lifecycle = Arc::new(TokenLifecycleState::new());

    domain_event_sink.start_worker(
        asset_service.clone(),
        connect_sync_service.clone(),
        event_bus.clone(),
        health_service.clone(),
        snapshot_service.clone(),
        quote_service.clone(),
        valuation_service.clone(),
        account_service.clone(),
        fx_service.clone(),
        timezone.clone(),
        secret_store.clone(),
        token_lifecycle.clone(),
    );

    let addon_service: Arc<dyn AddonServiceTrait + Send + Sync> = Arc::new(AddonService::new(
        &config.addons_root,
        &settings.instance_id,
    ));

    let auth_manager = config
        .auth
        .as_ref()
        .map(AuthManager::new)
        .transpose()?
        .map(Arc::new);

    let user_repo: Option<Arc<dyn UserRepositoryTrait>> = Some(Arc::new(
        PgUserRepository::new(pool.clone()),
    ));
    let email_service = user_repo.as_ref().map(|_| {
        Arc::new(EmailService::new())
    });

    Ok(Arc::new(AppState {
        domain_event_sink,
        account_service,
        settings_service,
        holdings_service,
        valuation_service,
        allocation_service,
        quote_service,
        base_currency,
        timezone,
        snapshot_service,
        snapshot_repository,
        performance_service,
        income_service,
        goal_service,
        limits_service,
        fx_service: fx_service.clone(),
        activity_service,
        asset_service,
        taxonomy_service,
        net_worth_service,
        alternative_asset_service,
        addon_service,
        connect_sync_service,
        ai_provider_service,
        ai_chat_service,
        data_root: data_root.clone(),
        database_url: database_url.to_string(),
        instance_id: settings.instance_id,
        secret_store,
        event_bus,
        auth: auth_manager,
        user_repo,
        email: email_service,
        device_enroll_service,
        app_sync_repository,
        device_sync_runtime,
        health_service,
        token_lifecycle,
        custom_provider_service,
    }))
}
