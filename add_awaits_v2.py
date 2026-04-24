#!/usr/bin/env python3
"""
Add .await to async method calls in Rust service implementations.
Handles multi-line calls by processing the full file content.
Only targets calls on self.xxx_repository, self.xxx_service, self.xxx_store, self.xxx_repo.
"""
import re, sys, os

# Async method patterns - calls that need .await
# We need to match:
# self.xxx_repo.method(...)?  ->  self.xxx_repo.method(...).await?
# self.xxx_repo.method(...);  ->  self.xxx_repo.method(...).await;
# self.xxx_repo.method(...).unwrap_or_default()  ->  self.xxx_repo.method(...).await.unwrap_or_default()
# self.xxx_repo.method(...).ok()  ->  self.xxx_repo.method(...).await.ok()
# self.xxx_repo.method(...).map_err(...)  ->  self.xxx_repo.method(...).await.map_err(...)
# etc.
# And self.xxx.method() as return value (no ? or ;)

def make_self_pattern():
    """Build regex pattern for self.xxx_repo/service/store calls."""
    # Match self.xxx_repository, self.xxx_service, self.xxx_store, self.xxx_repo
    # Also match just self.repository, self.service, self.store
    return r'self\.(?:\w+_(?:repository|service|store|repo)|(?:repository|service|store|repo))'

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    SELF = make_self_pattern()
    
    # We need to handle:
    # 1. Single-line: self.xxx.method(args)?  or  self.xxx.method(args);
    # 2. Multi-line: self.xxx\n    .method(\n        args\n    )?;
    # 3. Chained: self.xxx.method(args).unwrap_or_default()
    # 4. Return: self.xxx.method(args) as last expression
    
    # Strategy: use DOTALL regex to match across lines
    
    # Pattern 1: self.xxx.method(...)?
    # Match the call (which may span multiple lines) ending with )?
    # We look for: self.xxx.<word>(<anything>)?
    pattern1 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\?)',
        re.DOTALL
    )
    content = pattern1.sub(r'\1.await\2', content)
    
    # Pattern 1b: multi-line - self.xxx\n    .method(\n    args\n)?;
    # The closing )? might be at the end of the parenthesized expression
    pattern1b = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))\s*(\?)',
        re.DOTALL
    )
    content = pattern1b.sub(r'\1.await\2', content)
    
    # Pattern 2: self.xxx.method(...).unwrap_or_default()
    pattern2 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.unwrap_or_default\(\))',
        re.DOTALL
    )
    content = pattern2.sub(r'\1.await\2', content)
    
    # Pattern 3: self.xxx.method(...).ok()
    pattern3 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.ok\(\))',
        re.DOTALL
    )
    content = pattern3.sub(r'\1.await\2', content)
    
    # Pattern 3b: self.xxx.method(...).ok().flatten()
    pattern3b = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.ok\(\)\.flatten\(\))',
        re.DOTALL
    )
    content = pattern3b.sub(r'\1.await\2', content)
    
    # Pattern 4: self.xxx.method(...).map_err(|...| ...) or .map_err(...)
    pattern4 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.map_err\()',
        re.DOTALL
    )
    content = pattern4.sub(r'\1.await\2', content)
    
    # Pattern 5: self.xxx.method(...).map(|...| ...) or .map(...)
    pattern5 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.map\()',
        re.DOTALL
    )
    content = pattern5.sub(r'\1.await\2', content)
    
    # Pattern 6: self.xxx.method(...).or_else(...)
    pattern6 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.or_else\()',
        re.DOTALL
    )
    content = pattern6.sub(r'\1.await\2', content)
    
    # Pattern 7: self.xxx.method(...).and_then(...)
    pattern7 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\.and_then\()',
        re.DOTALL
    )
    content = pattern7.sub(r'\1.await\2', content)
    
    # Pattern 8: self.xxx.method(...); (statement)
    pattern8 = re.compile(
        r'(' + SELF + r'\.\w+\([^)]*\))(\s*;)',
        re.DOTALL
    )
    content = pattern8.sub(r'\1.await\2', content)
    
    # Pattern 9: self.xxx.method(...) as return value (no trailing ? ;)
    # This is tricky - we need to find self.xxx.method(...) at the end of a code block
    # Match: self.xxx.method(...)\n followed by } or whitespace+}
    # Actually let's match self.xxx.method(...) followed by whitespace then newline
    # But only if not already followed by .await
    # Skip this for now - handle manually
    
    # Now handle multi-line calls where the method chain starts on one line
    # and the closing )? is on another line
    # Example:
    #   self.activity_repository
    #       .check_existing_duplicates(std::slice::from_ref(key))?;
    # After first pass, the .check_existing_duplicates(...) part is single-line
    # so pattern1 should have caught it
    
    # Handle: self.xxx\n    .method(...)?  (method on next line after self.xxx)
    # This pattern was already handled if .method(args)? is on one line
    
    # Handle multi-line parens:
    # self.xxx.method(
    #     arg1,
    #     arg2,
    # )?
    # For this, [^)]* won't match across newlines... wait, DOTALL makes . match \n
    # But [^)] doesn't match ) but DOES match \n with DOTALL? No - [^)] is a character class,
    # it matches any char except ). DOTALL doesn't affect character classes.
    # So [^)]* won't span lines if there are ) in between... actually it should be fine
    # because [^)] just means "not )" which includes newlines.
    
    # Wait, actually [^)] DOES match newlines. DOTALL only affects . (dot).
    # So [^)]* already matches across lines. Good.
    
    # But we have a problem: nested parens! If the method args contain
    # parens like .method(foo(bar))?, then [^)]* will stop at the first ).
    # We need a different approach for nested parens.
    
    # Let's handle the specific nested case: .method(Some(...))?
    # We can't easily handle arbitrary nesting with regex.
    # Instead, let's process line by line for the remaining cases.
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False


def process_file_line_by_line(filepath):
    """Process file line by line to handle multi-line calls and nested parens."""
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    SELF = make_self_pattern()
    modified = False
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this line starts an async call that continues on subsequent lines
        # Pattern: self.xxx.method(  (line ends with open paren, no close paren)
        # or: self.xxx\n  (next line has .method(...)
        
        # Check for multi-line call starting with self.xxx on this line
        if re.search(SELF + r'\.\w+\(', line) and ')' not in line.split('(')[1:]:
            # This is a call that opens on this line but doesn't close
            # Find the closing paren
            depth = line.count('(') - line.count(')')
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count('(') - lines[j].count(')')
                j += 1
            
            # j is now the line AFTER the closing paren
            if j <= len(lines) and depth == 0:
                closing_line = lines[j-1]
                # Check what follows the closing paren on the last line
                # Find the position of the last ) that closes our call
                after_close = closing_line.rstrip()
                
                if after_close.endswith(')?'):
                    lines[j-1] = after_close[:-2] + ').await?\n'
                    modified = True
                elif after_close.endswith(');'):
                    lines[j-1] = after_close[:-1] + '.await;\n'
                    modified = True
                elif after_close.endswith(').unwrap_or_default()'):
                    lines[j-1] = after_close.replace(').unwrap_or_default()', ').await.unwrap_or_default()', 1) + '\n'
                    modified = True
                elif after_close.endswith(').ok()'):
                    lines[j-1] = after_close.replace(').ok()', ').await.ok()', 1) + '\n'
                    modified = True
                elif after_close.endswith(').ok().flatten()'):
                    lines[j-1] = after_close.replace(').ok().flatten()', ').await.ok().flatten()', 1) + '\n'
                    modified = True
        
        # Check for pattern: self.xxx on this line, .method(args)? on next line
        # e.g., self.activity_repository\n    .get_activity(activity_id)?;
        elif re.search(r'^\s*' + SELF + r'\s*$', line):
            # Next line should have .method(args)? pattern
            if i + 1 < len(lines):
                next_line = lines[i + 1]
                if re.search(r'\.\w+\([^)]*\)\?', next_line):
                    lines[i+1] = re.sub(r'(\.\w+\([^)]*\))(\?)', r'\1.await\2', next_line)
                    modified = True
                elif re.search(r'\.\w+\([^)]*\);', next_line):
                    lines[i+1] = re.sub(r'(\.\w+\([^)]*\))(;)', r'\1.await\2', next_line)
                    modified = True
        
        # Check for pattern: self.xxx\n    .method(\n    args\n    )?;
        # First line: self.xxx
        # Second line: .method(
        # ... args ...
        # Last line: )?;
        elif re.search(r'^\s*' + SELF + r'\s*$', line):
            pass  # handled above
        
        i += 1
    
    if modified:
        with open(filepath, 'w') as f:
            f.writelines(lines)
        return True
    return False


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
    
    for f in files:
        if os.path.exists(f):
            changed1 = process_file(f)
            changed2 = process_file_line_by_line(f)
            status = []
            if changed1: status.append("regex")
            if changed2: status.append("line-by-line")
            if not status: status.append("unchanged")
            print(f"  {'+'.join(status)}: {f}")
