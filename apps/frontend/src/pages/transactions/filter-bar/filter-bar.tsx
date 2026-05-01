import { useRef } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import { Input } from "@whaleit/ui/components/ui/input";
import { cn } from "@/lib/utils";
import { Icons } from "@whaleit/ui/components/ui/icons";
import type { Account } from "@/lib/types";
import type { TransactionFilters } from "@/lib/types/transaction";
import { AccountFilter } from "./account-filter";
import { DateRangeFilter } from "./date-range-filter";
import { CategoryFilter } from "./category-filter";
import { AmountFilter } from "./amount-filter";

interface FilterBarProps {
  filters: TransactionFilters;
  onChange: (filters: TransactionFilters) => void;
  searchKeyword: string;
  onSearchChange: (kw: string) => void;
  accounts: Account[];
}

export function FilterBar({
  filters,
  onChange,
  searchKeyword,
  onSearchChange,
  accounts,
}: FilterBarProps) {
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function handleSearch(value: string) {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      onSearchChange(value);
    }, 250);
  }

  const hasActiveFilters =
    (filters.accountIds?.length ?? 0) > 0 ||
    (filters.categoryIds?.length ?? 0) > 0 ||
    filters.dateFrom != null ||
    filters.dateTo != null ||
    filters.amountMin != null ||
    filters.amountMax != null;

  function clearAll() {
    onChange({
      accountIds: [],
      categoryIds: [],
      directions: [],
      dateFrom: undefined,
      dateTo: undefined,
      amountMin: undefined,
      amountMax: undefined,
      showTransfers: true,
    });
    onSearchChange("");
  }

  return (
    <div className="bg-background/80 sticky top-0 z-10 border-b px-4 py-2 backdrop-blur">
      <div className="flex flex-wrap items-center gap-2">
        {/* Search */}
        <div className="relative min-w-40 flex-1">
          <Icons.Search className="text-muted-foreground absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2" />
          <Input
            type="search"
            placeholder="Search transactions..."
            defaultValue={searchKeyword}
            onChange={(e) => handleSearch(e.target.value)}
            className="h-8 rounded-full pl-8 text-xs"
          />
        </div>

        {/* Filter chips */}
        <AccountFilter
          accounts={accounts}
          selected={filters.accountIds ?? []}
          onChange={(ids) => onChange({ ...filters, accountIds: ids })}
        />
        <DateRangeFilter
          dateFrom={filters.dateFrom}
          dateTo={filters.dateTo}
          onChange={(from, to) => onChange({ ...filters, dateFrom: from, dateTo: to })}
        />
        <CategoryFilter
          selected={filters.categoryIds ?? []}
          onChange={(ids) => onChange({ ...filters, categoryIds: ids })}
        />
        <AmountFilter
          amountMin={filters.amountMin}
          amountMax={filters.amountMax}
          onChange={(min, max) => onChange({ ...filters, amountMin: min, amountMax: max })}
        />

        {/* Transfers toggle */}
        <Button
          variant="outline"
          size="sm"
          data-state={filters.showTransfers !== false ? "on" : "off"}
          className={cn(
            "h-8 rounded-full px-3 text-xs",
            filters.showTransfers !== false && "border-primary text-primary",
          )}
          onClick={() =>
            onChange({ ...filters, showTransfers: !(filters.showTransfers !== false) })
          }
        >
          Transfers
        </Button>

        {/* Clear all */}
        {hasActiveFilters && (
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground h-8 rounded-full px-3 text-xs"
            onClick={clearAll}
          >
            Clear filters
          </Button>
        )}
      </div>
    </div>
  );
}
