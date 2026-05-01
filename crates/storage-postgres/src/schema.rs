// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Text,
        name -> Text,
        account_type -> Text,
        group -> Nullable<Text>,
        currency -> Text,
        is_default -> Bool,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        platform_id -> Nullable<Text>,
        account_number -> Nullable<Text>,
        meta -> Nullable<Text>,
        provider -> Nullable<Text>,
        provider_account_id -> Nullable<Text>,
        is_archived -> Bool,
        tracking_mode -> Text,
        institution -> Nullable<Text>,
        opening_balance -> Nullable<Numeric>,
        current_balance -> Nullable<Numeric>,
        balance_updated_at -> Nullable<Timestamp>,
        credit_limit -> Nullable<Numeric>,
        statement_cycle_day -> Nullable<Int2>,
        statement_balance -> Nullable<Numeric>,
        minimum_payment -> Nullable<Numeric>,
        statement_due_date -> Nullable<Date>,
        reward_points_balance -> Nullable<Int4>,
        cashback_balance -> Nullable<Numeric>,
    }
}

diesel::table! {
    activities (id) {
        id -> Text,
        account_id -> Text,
        asset_id -> Nullable<Text>,
        activity_type -> Text,
        activity_type_override -> Nullable<Text>,
        source_type -> Nullable<Text>,
        subtype -> Nullable<Text>,
        status -> Text,
        activity_date -> Text,
        settlement_date -> Nullable<Text>,
        quantity -> Nullable<Text>,
        unit_price -> Nullable<Text>,
        amount -> Nullable<Text>,
        fee -> Nullable<Text>,
        currency -> Text,
        fx_rate -> Nullable<Text>,
        notes -> Nullable<Text>,
        metadata -> Nullable<Text>,
        source_system -> Nullable<Text>,
        source_record_id -> Nullable<Text>,
        source_group_id -> Nullable<Text>,
        idempotency_key -> Nullable<Text>,
        import_run_id -> Nullable<Text>,
        is_user_modified -> Bool,
        needs_review -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    ai_messages (id) {
        id -> Text,
        thread_id -> Text,
        role -> Text,
        content_json -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    ai_thread_tags (id) {
        id -> Text,
        thread_id -> Text,
        tag -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    ai_threads (id) {
        id -> Text,
        title -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        config_snapshot -> Nullable<Text>,
        is_pinned -> Bool,
    }
}

diesel::table! {
    api_keys (id) {
        id -> Text,
        user_id -> Text,
        key_prefix -> Text,
        key_hash -> Text,
        name -> Text,
        last_used_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        is_active -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    app_settings (setting_key) {
        setting_key -> Text,
        setting_value -> Text,
    }
}

diesel::table! {
    asset_taxonomy_assignments (id) {
        id -> Text,
        asset_id -> Text,
        taxonomy_id -> Text,
        category_id -> Text,
        weight -> Int4,
        source -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    assets (id) {
        id -> Text,
        kind -> Text,
        name -> Nullable<Text>,
        display_code -> Nullable<Text>,
        notes -> Nullable<Text>,
        metadata -> Nullable<Text>,
        is_active -> Bool,
        quote_mode -> Text,
        quote_ccy -> Text,
        instrument_type -> Nullable<Text>,
        instrument_symbol -> Nullable<Text>,
        instrument_exchange_mic -> Nullable<Text>,
        instrument_key -> Nullable<Text>,
        provider_config -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    brokers_sync_state (account_id, provider) {
        account_id -> Text,
        provider -> Text,
        checkpoint_json -> Nullable<Text>,
        last_attempted_at -> Nullable<Timestamp>,
        last_successful_at -> Nullable<Timestamp>,
        last_error -> Nullable<Text>,
        last_run_id -> Nullable<Text>,
        sync_status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    contribution_limits (id) {
        id -> Text,
        group_name -> Text,
        contribution_year -> Int4,
        limit_amount -> Float8,
        account_ids -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        start_date -> Nullable<Text>,
        end_date -> Nullable<Text>,
    }
}

diesel::table! {
    daily_account_valuation (id) {
        id -> Text,
        account_id -> Text,
        valuation_date -> Date,
        account_currency -> Text,
        base_currency -> Text,
        fx_rate_to_base -> Text,
        cash_balance -> Text,
        investment_market_value -> Text,
        total_value -> Text,
        cost_basis -> Text,
        net_contribution -> Text,
        calculated_at -> Timestamp,
    }
}

diesel::table! {
    goals (id) {
        id -> Text,
        title -> Text,
        description -> Nullable<Text>,
        target_amount -> Float8,
        is_achieved -> Bool,
    }
}

diesel::table! {
    goals_allocation (id) {
        id -> Text,
        percent_allocation -> Int4,
        goal_id -> Text,
        account_id -> Text,
    }
}

diesel::table! {
    health_issue_dismissals (issue_id) {
        issue_id -> Text,
        dismissed_at -> Timestamp,
        data_hash -> Text,
    }
}

diesel::table! {
    holdings_snapshots (id) {
        id -> Text,
        account_id -> Text,
        snapshot_date -> Date,
        currency -> Text,
        positions -> Text,
        cash_balances -> Text,
        cost_basis -> Text,
        net_contribution -> Text,
        calculated_at -> Timestamp,
        net_contribution_base -> Text,
        cash_total_account_currency -> Text,
        cash_total_base_currency -> Text,
        source -> Text,
    }
}

diesel::table! {
    import_account_templates (id) {
        id -> Text,
        account_id -> Text,
        context_kind -> Text,
        source_system -> Text,
        template_id -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    import_runs (id) {
        id -> Text,
        account_id -> Text,
        source_system -> Text,
        run_type -> Text,
        mode -> Text,
        status -> Text,
        started_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
        review_mode -> Text,
        applied_at -> Nullable<Timestamp>,
        checkpoint_in -> Nullable<Text>,
        checkpoint_out -> Nullable<Text>,
        summary -> Nullable<Text>,
        warnings -> Nullable<Text>,
        error -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    import_templates (id) {
        id -> Text,
        name -> Text,
        scope -> Text,
        kind -> Text,
        source_system -> Text,
        config_version -> Int4,
        config -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    market_data_custom_providers (id) {
        id -> Text,
        code -> Text,
        name -> Text,
        description -> Text,
        enabled -> Bool,
        priority -> Int4,
        config -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    market_data_providers (id) {
        id -> Text,
        name -> Text,
        description -> Text,
        url -> Nullable<Text>,
        priority -> Int4,
        enabled -> Bool,
        logo_filename -> Nullable<Text>,
        last_synced_at -> Nullable<Timestamp>,
        last_sync_status -> Nullable<Text>,
        last_sync_error -> Nullable<Text>,
        provider_type -> Text,
        config -> Nullable<Text>,
    }
}

diesel::table! {
    payee_category_memory (account_id, normalized_merchant) {
        account_id -> Text,
        normalized_merchant -> Text,
        category_id -> Text,
        last_seen_at -> Timestamp,
        seen_count -> Int4,
    }
}

diesel::table! {
    platforms (id) {
        id -> Text,
        name -> Nullable<Text>,
        url -> Text,
        external_id -> Nullable<Text>,
        kind -> Text,
        website_url -> Nullable<Text>,
        logo_url -> Nullable<Text>,
    }
}

diesel::table! {
    quote_sync_state (asset_id) {
        asset_id -> Text,
        position_closed_date -> Nullable<Text>,
        last_synced_at -> Nullable<Timestamp>,
        data_source -> Text,
        sync_priority -> Int4,
        error_count -> Int4,
        last_error -> Nullable<Text>,
        profile_enriched_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    quotes (id) {
        id -> Text,
        asset_id -> Text,
        day -> Text,
        source -> Text,
        open -> Nullable<Text>,
        high -> Nullable<Text>,
        low -> Nullable<Text>,
        close -> Text,
        adjclose -> Nullable<Text>,
        volume -> Nullable<Text>,
        currency -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    sync_applied_events (event_id) {
        event_id -> Text,
        seq -> Int8,
        entity -> Text,
        entity_id -> Text,
        applied_at -> Timestamp,
    }
}

diesel::table! {
    sync_cursor (id) {
        id -> Int4,
        cursor -> Int8,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sync_device_config (device_id) {
        device_id -> Text,
        key_version -> Nullable<Int4>,
        trust_state -> Text,
        last_bootstrap_at -> Nullable<Timestamp>,
        min_snapshot_created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sync_engine_state (id) {
        id -> Int4,
        lock_version -> Int8,
        last_push_at -> Nullable<Timestamp>,
        last_pull_at -> Nullable<Timestamp>,
        last_error -> Nullable<Text>,
        consecutive_failures -> Int4,
        next_retry_at -> Nullable<Timestamp>,
        last_cycle_status -> Nullable<Text>,
        last_cycle_duration_ms -> Nullable<Int8>,
    }
}

diesel::table! {
    sync_entity_metadata (entity, entity_id) {
        entity -> Text,
        entity_id -> Text,
        last_event_id -> Text,
        last_client_timestamp -> Timestamp,
        last_seq -> Int8,
    }
}

diesel::table! {
    sync_outbox (event_id) {
        event_id -> Text,
        entity -> Text,
        entity_id -> Text,
        op -> Text,
        client_timestamp -> Timestamp,
        payload -> Text,
        payload_key_version -> Int4,
        sent -> Bool,
        status -> Text,
        retry_count -> Int4,
        next_retry_at -> Nullable<Timestamp>,
        last_error -> Nullable<Text>,
        last_error_code -> Nullable<Text>,
        device_id -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sync_table_state (table_name) {
        table_name -> Text,
        enabled -> Bool,
        last_snapshot_restore_at -> Nullable<Timestamp>,
        last_incremental_apply_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    taxonomies (id) {
        id -> Text,
        name -> Text,
        color -> Text,
        description -> Nullable<Text>,
        is_system -> Bool,
        is_single_select -> Bool,
        sort_order -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    taxonomy_categories (id, taxonomy_id) {
        id -> Text,
        taxonomy_id -> Text,
        parent_id -> Nullable<Text>,
        name -> Text,
        key -> Text,
        color -> Text,
        description -> Nullable<Text>,
        sort_order -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transaction_csv_templates (id) {
        id -> Text,
        name -> Text,
        mapping -> Jsonb,
        header_signature -> Array<Nullable<Text>>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transaction_splits (id) {
        id -> Text,
        transaction_id -> Text,
        category_id -> Text,
        amount -> Numeric,
        notes -> Nullable<Text>,
        sort_order -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Text,
        account_id -> Text,
        direction -> Text,
        amount -> Numeric,
        currency -> Text,
        transaction_date -> Date,
        payee -> Nullable<Text>,
        notes -> Nullable<Text>,
        category_id -> Nullable<Text>,
        has_splits -> Bool,
        fx_rate -> Nullable<Numeric>,
        fx_rate_source -> Nullable<Text>,
        transfer_group_id -> Nullable<Text>,
        counterparty_account_id -> Nullable<Text>,
        transfer_leg_role -> Nullable<Text>,
        idempotency_key -> Nullable<Text>,
        import_run_id -> Nullable<Text>,
        source -> Text,
        external_ref -> Nullable<Text>,
        is_system_generated -> Bool,
        is_user_modified -> Bool,
        category_source -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password_hash -> Text,
        display_name -> Nullable<Text>,
        email_verified -> Bool,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    verification_tokens (id) {
        id -> Text,
        user_id -> Text,
        token_hash -> Text,
        token_type -> Text,
        expires_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(accounts -> platforms (platform_id));
diesel::joinable!(activities -> accounts (account_id));
diesel::joinable!(activities -> assets (asset_id));
diesel::joinable!(activities -> import_runs (import_run_id));
diesel::joinable!(ai_messages -> ai_threads (thread_id));
diesel::joinable!(ai_thread_tags -> ai_threads (thread_id));
diesel::joinable!(api_keys -> users (user_id));
diesel::joinable!(asset_taxonomy_assignments -> assets (asset_id));
diesel::joinable!(asset_taxonomy_assignments -> taxonomies (taxonomy_id));
diesel::joinable!(brokers_sync_state -> accounts (account_id));
diesel::joinable!(brokers_sync_state -> import_runs (last_run_id));
diesel::joinable!(goals_allocation -> accounts (account_id));
diesel::joinable!(goals_allocation -> goals (goal_id));
diesel::joinable!(import_account_templates -> accounts (account_id));
diesel::joinable!(import_account_templates -> import_templates (template_id));
diesel::joinable!(import_runs -> accounts (account_id));
diesel::joinable!(payee_category_memory -> accounts (account_id));
diesel::joinable!(quote_sync_state -> assets (asset_id));
diesel::joinable!(quotes -> assets (asset_id));
diesel::joinable!(taxonomy_categories -> taxonomies (taxonomy_id));
diesel::joinable!(transaction_splits -> transactions (transaction_id));
diesel::joinable!(verification_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    activities,
    ai_messages,
    ai_thread_tags,
    ai_threads,
    api_keys,
    app_settings,
    asset_taxonomy_assignments,
    assets,
    brokers_sync_state,
    contribution_limits,
    daily_account_valuation,
    goals,
    goals_allocation,
    health_issue_dismissals,
    holdings_snapshots,
    import_account_templates,
    import_runs,
    import_templates,
    market_data_custom_providers,
    market_data_providers,
    payee_category_memory,
    platforms,
    quote_sync_state,
    quotes,
    sync_applied_events,
    sync_cursor,
    sync_device_config,
    sync_engine_state,
    sync_entity_metadata,
    sync_outbox,
    sync_table_state,
    taxonomies,
    taxonomy_categories,
    transaction_csv_templates,
    transaction_splits,
    transactions,
    users,
    verification_tokens,
);
