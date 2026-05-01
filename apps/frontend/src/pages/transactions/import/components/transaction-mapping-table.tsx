// Transaction Mapping Table (Phase 4, plan 04-08).
//
// Forked from apps/frontend/src/pages/activity/import/components/mapping-table.tsx.
// Drops all asset/symbol/account-resolution columns.
// Transaction field set: date (required), amount OR debit+credit (required),
// payee (required), category, notes, currency, external_id (optional).
// Reads/writes CsvFieldMapping via dispatch to the import context.

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@whaleit/ui/components/ui/select";
import { Badge } from "@whaleit/ui/components/ui/badge";
import { cn } from "@/lib/utils";
import type { CsvFieldMapping } from "@/lib/types/transaction";

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const NONE_VALUE = "__none__";

export const DATE_FORMAT_OPTIONS = [
  { value: "auto", label: "Auto-detect" },
  { value: "YYYY-MM-DD", label: "YYYY-MM-DD" },
  { value: "MM/DD/YYYY", label: "MM/DD/YYYY" },
  { value: "DD/MM/YYYY", label: "DD/MM/YYYY" },
  { value: "DD.MM.YYYY", label: "DD.MM.YYYY" },
  { value: "MM-DD-YYYY", label: "MM-DD-YYYY" },
];

const DECIMAL_SEP_OPTIONS = [
  { value: ".", label: "Period ( . )" },
  { value: ",", label: "Comma ( , )" },
];

const THOUSANDS_SEP_OPTIONS = [
  { value: NONE_VALUE, label: "None" },
  { value: ",", label: "Comma ( , )" },
  { value: ".", label: "Period ( . )" },
  { value: " ", label: "Space (   )" },
];

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

interface TransactionMappingTableProps {
  headers: string[];
  mapping: CsvFieldMapping;
  onMappingChange: (updated: CsvFieldMapping) => void;
  /** If true, renders "Amount" as two separate Debit / Credit selectors */
  useSplitAmountColumns?: boolean;
  onToggleSplitAmount?: (split: boolean) => void;
}

// ─────────────────────────────────────────────────────────────────────────────
// Column select helper
// ─────────────────────────────────────────────────────────────────────────────

function ColumnSelect({
  label,
  required,
  value,
  headers,
  onChange,
}: {
  label: string;
  required?: boolean;
  value: string | null | undefined;
  headers: string[];
  onChange: (v: string | null) => void;
}) {
  const current = value || NONE_VALUE;

  return (
    <div className="flex items-center gap-3">
      <div className="w-44 shrink-0">
        <span className="text-sm font-medium">
          {label}
          {required && (
            <span className="text-destructive ml-1" aria-hidden="true">
              *
            </span>
          )}
        </span>
      </div>
      <Select value={current} onValueChange={(v) => onChange(v === NONE_VALUE ? null : v)}>
        <SelectTrigger className="h-8 min-w-0 flex-1">
          <SelectValue placeholder="Select column…" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={NONE_VALUE}>
            <span className="text-muted-foreground italic">— not mapped —</span>
          </SelectItem>
          {headers.map((h) => (
            <SelectItem key={h} value={h}>
              {h}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {required && !value && (
        <Badge variant="outline" className="border-destructive text-destructive shrink-0 text-xs">
          Required
        </Badge>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Component
// ─────────────────────────────────────────────────────────────────────────────

export function TransactionMappingTable({
  headers,
  mapping,
  onMappingChange,
  useSplitAmountColumns = false,
  onToggleSplitAmount,
}: TransactionMappingTableProps) {
  const set = <K extends keyof CsvFieldMapping>(key: K, value: CsvFieldMapping[K]) => {
    onMappingChange({ ...mapping, [key]: value });
  };

  // Check required fields to highlight incomplete state
  const dateOk = !!mapping.dateColumn;
  const amountOk = useSplitAmountColumns
    ? !!(mapping.debitColumn || mapping.creditColumn)
    : !!mapping.amountColumn;
  const payeeOk = !!mapping.payeeColumn;
  const requiredsMet = dateOk && amountOk && payeeOk;

  return (
    <div className="space-y-6">
      {/* Format options */}
      <section>
        <h3 className="text-muted-foreground mb-3 text-xs font-semibold uppercase tracking-wider">
          Format Options
        </h3>
        <div className="space-y-3">
          <div className="flex items-center gap-3">
            <div className="w-44 shrink-0">
              <span className="text-sm font-medium">Date format</span>
            </div>
            <Select
              value={mapping.dateFormat || "auto"}
              onValueChange={(v) => set("dateFormat", v)}
            >
              <SelectTrigger className="h-8 min-w-0 flex-1">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {DATE_FORMAT_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-3">
            <div className="w-44 shrink-0">
              <span className="text-sm font-medium">Decimal separator</span>
            </div>
            <Select
              value={mapping.decimalSeparator || "."}
              onValueChange={(v) => set("decimalSeparator", v)}
            >
              <SelectTrigger className="h-8 min-w-0 flex-1">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {DECIMAL_SEP_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-3">
            <div className="w-44 shrink-0">
              <span className="text-sm font-medium">Thousands separator</span>
            </div>
            <Select
              value={mapping.thousandsSeparator ?? NONE_VALUE}
              onValueChange={(v) => set("thousandsSeparator", v === NONE_VALUE ? null : v)}
            >
              <SelectTrigger className="h-8 min-w-0 flex-1">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {THOUSANDS_SEP_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </section>

      {/* Required columns */}
      <section>
        <div className="mb-3 flex items-center justify-between">
          <h3 className="text-muted-foreground text-xs font-semibold uppercase tracking-wider">
            Column Mapping
          </h3>
          {!requiredsMet && (
            <span className="text-destructive text-xs">
              * Required fields must be mapped to continue
            </span>
          )}
        </div>
        <div className="space-y-3">
          <ColumnSelect
            label="Date"
            required
            value={mapping.dateColumn}
            headers={headers}
            onChange={(v) => set("dateColumn", v ?? "")}
          />

          {/* Amount: single column OR split debit/credit */}
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <div className="w-44 shrink-0">
                <span className="text-sm font-medium">
                  Amount
                  <span className="text-destructive ml-1" aria-hidden="true">
                    *
                  </span>
                </span>
              </div>
              {onToggleSplitAmount && (
                <button
                  type="button"
                  onClick={() => onToggleSplitAmount(!useSplitAmountColumns)}
                  className={cn(
                    "text-xs underline-offset-2 hover:underline",
                    useSplitAmountColumns ? "text-primary" : "text-muted-foreground",
                  )}
                >
                  {useSplitAmountColumns
                    ? "Use single amount column"
                    : "Use separate debit + credit columns"}
                </button>
              )}
            </div>

            {!useSplitAmountColumns ? (
              <div className="pl-[11.5rem]">
                <Select
                  value={mapping.amountColumn ?? NONE_VALUE}
                  onValueChange={(v) => set("amountColumn", v === NONE_VALUE ? null : v)}
                >
                  <SelectTrigger className="h-8 w-full">
                    <SelectValue placeholder="Select column…" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value={NONE_VALUE}>
                      <span className="text-muted-foreground italic">— not mapped —</span>
                    </SelectItem>
                    {headers.map((h) => (
                      <SelectItem key={h} value={h}>
                        {h}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ) : (
              <div className="space-y-2 pl-[11.5rem]">
                <ColumnSelect
                  label="Debit column"
                  value={mapping.debitColumn}
                  headers={headers}
                  onChange={(v) => set("debitColumn", v)}
                />
                <ColumnSelect
                  label="Credit column"
                  value={mapping.creditColumn}
                  headers={headers}
                  onChange={(v) => set("creditColumn", v)}
                />
              </div>
            )}
            {!amountOk && (
              <div className="pl-[11.5rem]">
                <Badge variant="outline" className="border-destructive text-destructive text-xs">
                  Required
                </Badge>
              </div>
            )}
          </div>

          <ColumnSelect
            label="Payee"
            required
            value={mapping.payeeColumn}
            headers={headers}
            onChange={(v) => set("payeeColumn", v ?? "")}
          />
        </div>
      </section>

      {/* Optional columns */}
      <section>
        <h3 className="text-muted-foreground mb-3 text-xs font-semibold uppercase tracking-wider">
          Optional Columns
        </h3>
        <div className="space-y-3">
          <ColumnSelect
            label="Category"
            value={mapping.categoryColumn}
            headers={headers}
            onChange={(v) => set("categoryColumn", v)}
          />
          <ColumnSelect
            label="Notes"
            value={mapping.notesColumn}
            headers={headers}
            onChange={(v) => set("notesColumn", v)}
          />
          <ColumnSelect
            label="Currency"
            value={mapping.currencyColumn}
            headers={headers}
            onChange={(v) => set("currencyColumn", v)}
          />
          <ColumnSelect
            label="External ID"
            value={mapping.externalIdColumn}
            headers={headers}
            onChange={(v) => set("externalIdColumn", v)}
          />
        </div>
      </section>
    </div>
  );
}
