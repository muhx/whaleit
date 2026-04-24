#!/usr/bin/env python3
"""
Add .await to async method calls using proper paren-depth tracking.
Handles multi-line calls and nested parentheses correctly.
"""
import re, os

# All async method names (from converted traits)
ASYNC_METHODS = {
    'get_by_id', 'list', 'get_activity', 'get_activities',
    'get_activities_by_account_id', 'get_activities_by_account_ids',
    'get_activities_by_asset', 'count_activities', 'list_activities',
    'get_trading_activities', 'get_income_activities',
    'search_activities', 'get_first_activity_date',
    'get_import_mapping', 'list_import_templates', 'get_import_template',
    'get_broker_sync_profile', 'check_existing_duplicates',
    'get_contribution_activities',
    'get_assets', 'get_asset_by_id', 'get_by_symbol',
    'get_active_assets', 'find_by_ids', 'list_by_asset_ids',
    'find_by_instrument_key', 'search_by_symbol',
    'get_alternative_holdings',
    'get_latest_rate', 'list_rates', 'get_historical_exchange_rates',
    'get_latest_exchange_rate', 'get_latest_exchange_rates',
    'get_exchange_rate_for_date',
    'get_goals', 'load_goals', 'load_goals_allocations',
    'get_goals_by_account',
    'is_dismissed', 'list_dismissed',
    'get_contribution_limit', 'get_contribution_limits',
    'find_by_account',
    'get_latest', 'get_by_date', 'list_range',
    'get_snapshots_by_account', 'get_snapshots_by_account_and_date',
    'get_historical_snapshots',
    'get_latest_valuations', 'get_historical_valuations',
    'get_latest_quote', 'get_latest_quotes', 'get_latest_quotes_pair',
    'get_latest_quotes_batch', 'get_quotes_range',
    'get_historical_quotes', 'get_all_historical_quotes',
    'get_quotes_in_range', 'get_quote_bounds_for_assets',
    'latest', 'range', 'get_latest_quote_before',
    'get_state', 'list_states', 'get_all',
    'get_by_asset_id', 'get_by_asset_ids',
    'get_provider_sync_stats', 'get_with_errors',
    'get_settings', 'get',
    'get_taxonomies', 'get_taxonomy', 'get_categories',
    'get_category_by_id', 'get_all_taxonomies_with_categories',
    'get_taxonomy_with_categories',
    'get_asset_assignments', 'get_category_assignments',
    'export_taxonomy_json',
    'create_import_run', 'update_import_run', 'get_import_runs',
    'initialize', 'get_historical_rates',
    'convert_currency', 'convert_currency_for_date',
    'validate_persisted_symbol_metadata', 'existing_quote_ccy_by_symbol',
    'deactivate_orphaned_investments',
}

# Build regex pattern for matching start of async call
# Pattern: self.xxx.method_name(
SELF_PATTERN = r'self\.(\w+(?:_repository|_service|_store|_repo|_repo|repository|service|store|repo))\.(\w+)\('

def find_call_span(lines, start_line, start_col):
    """Find the closing paren of a call starting at (start_line, start_col).
    Returns (end_line, end_col) of the closing paren.
    Uses paren depth tracking to handle nested parens correctly.
    """
    depth = 0
    for i in range(start_line, len(lines)):
        line = lines[i]
        j = start_col if i == start_line else 0
        while j < len(line):
            c = line[j]
            if c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
                if depth == 0:
                    return (i, j)
            # Skip string literals
            if c == '"':
                j += 1
                while j < len(line) and line[j] != '"':
                    if line[j] == '\\':
                        j += 1
                    j += 1
            # Skip char literals
            if c == "'":
                j += 1
                while j < len(line) and line[j] != "'":
                    j += 1
            j += 1
    return None

def process_file(filepath):
    """Process a file to add .await to async method calls."""
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    changes = []
    
    for i, line in enumerate(lines):
        # Find all async method calls on this line
        for match in re.finditer(SELF_PATTERN, line):
            obj_name = match.group(1)
            method_name = match.group(2)
            
            if method_name not in ASYNC_METHODS:
                continue
            
            # Check if this line already has .await after the call
            # We'll check after finding the full call span
            
            start_col = match.end() - 1  # Position of the opening (
            span = find_call_span(lines, i, start_col)
            if span is None:
                continue
            
            end_line, end_col = span
            
            # Check if .await already exists right after the closing paren
            rest_of_end_line = lines[end_line][end_col+1:]
            if rest_of_end_line.lstrip().startswith('.await'):
                continue
            
            # Determine what follows the closing paren
            if end_line == i:
                # Single-line call
                after = lines[i][end_col+1:]
            else:
                after = lines[end_line][end_col+1:]
            
            # Insert .await based on what follows
            # Patterns: )? -> ).await?, ); -> ).await;, ).xxx -> ).await.xxx
            # ) at end of expression -> ).await
            
            if after.lstrip().startswith('?'):
                # )? -> ).await?
                insert_pos = end_col + 1
                line_content = lines[end_line]
                lines[end_line] = line_content[:insert_pos] + '.await' + line_content[insert_pos:]
                changes.append(f"L{end_line+1}: added .await before ?")
            elif after.lstrip().startswith(';'):
                insert_pos = end_col + 1
                line_content = lines[end_line]
                lines[end_line] = line_content[:insert_pos] + '.await' + line_content[insert_pos:]
                changes.append(f"L{end_line+1}: added .await before ;")
            elif after.lstrip().startswith('.'):
                # Chained method like .unwrap_or_default(), .ok(), etc.
                dot_pos = end_col + 1 + (len(after) - len(after.lstrip()))
                line_content = lines[end_line]
                # Insert .await right after the closing paren
                insert_pos = end_col + 1
                lines[end_line] = line_content[:insert_pos] + '.await' + line_content[insert_pos:]
                changes.append(f"L{end_line+1}: added .await before .{after.lstrip()[:20]}")
            elif after.strip() == '' or after.strip() == '\n':
                # End of expression (return value)
                insert_pos = end_col + 1
                line_content = lines[end_line]
                lines[end_line] = line_content[:insert_pos] + '.await' + line_content[insert_pos:]
                changes.append(f"L{end_line+1}: added .await (return value)")
    
    if changes:
        with open(filepath, 'w') as f:
            f.writelines(lines)
        return changes
    return []

if __name__ == '__main__':
    files = [
        'crates/core/src/accounts/accounts_service.rs',
        'crates/core/src/activities/activities_service.rs',
        'crates/core/src/assets/assets_service.rs',
        'crates/core/src/assets/alternative_assets_service.rs',
        'crates/core/src/assets/classification_service.rs',
        'crates/core/src/fx/fx_service.rs',
        'crates/core/src/goals/goals_service.rs',
        'crates/core/src/health/checks/classification.rs',
        'crates/core/src/health/checks/quote_sync.rs',
        'crates/core/src/health/fixes/classification_migration.rs',
        'crates/core/src/health/service.rs',
        'crates/core/src/limits/limits_service.rs',
        'crates/core/src/portfolio/allocation/allocation_service.rs',
        'crates/core/src/portfolio/holdings/holdings_valuation_service.rs',
        'crates/core/src/portfolio/income/income_service.rs',
        'crates/core/src/portfolio/net_worth/net_worth_service.rs',
        'crates/core/src/portfolio/snapshot/holdings_calculator.rs',
        'crates/core/src/portfolio/snapshot/snapshot_service.rs',
        'crates/core/src/portfolio/valuation/valuation_service.rs',
        'crates/core/src/quotes/import.rs',
        'crates/core/src/quotes/service.rs',
        'crates/core/src/quotes/sync.rs',
        'crates/core/src/settings/settings_service.rs',
        'crates/core/src/taxonomies/taxonomy_service.rs',
    ]
    
    total = 0
    for f in files:
        if os.path.exists(f):
            changes = process_file(f)
            if changes:
                print(f"\n{f} ({len(changes)} fixes):")
                for c in changes:
                    print(f"  {c}")
                total += len(changes)
            else:
                print(f"  unchanged: {f}")
    
    print(f"\nTotal: {total} .await insertions")
