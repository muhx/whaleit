import {
  ArrowDownLeft,
  ArrowLeftRight,
  ArrowUpRight,
  MessageSquare,
  SplitSquareHorizontal,
  Tag,
} from "lucide-react";
import { PrivacyAmount } from "@whaleit/ui";
import { cn } from "@/lib/utils";
import type { Transaction, TransactionDirection } from "@/lib/types/transaction";

// Duplicate confidence bucket → row tint (UI-SPEC §6 D-09)
function getDuplicateTint(confidence: number | undefined): string | null {
  if (confidence == null) return null;
  if (confidence >= 95) return "bg-destructive/10";
  if (confidence >= 70) return "bg-warning/10";
  if (confidence >= 50) return "bg-muted/50";
  return null;
}

const DIRECTION_ICONS: Record<
  TransactionDirection,
  { icon: React.ComponentType<{ className?: string }>; className: string }
> = {
  INCOME: { icon: ArrowDownLeft, className: "text-success" },
  EXPENSE: { icon: ArrowUpRight, className: "text-muted-foreground" },
  TRANSFER: { icon: ArrowLeftRight, className: "text-muted-foreground" },
};

function CategoryChip({ name, color }: { name: string | undefined; color: string | undefined }) {
  if (!name) return null;
  const bg = color ? `bg-[${color}]/10` : "bg-muted";
  const text = color ? `text-[${color}]` : "text-muted-foreground";
  return (
    <span
      className={cn(
        "inline-flex items-center gap-0.5 rounded-sm px-1 py-0.5 text-[11px] font-medium",
        bg,
        text,
      )}
    >
      <Tag className="size-2.5" aria-hidden="true" />
      {name}
    </span>
  );
}

function SplitBadge({ count }: { count: number }) {
  return (
    <span className="bg-muted text-muted-foreground inline-flex items-center gap-0.5 rounded-sm px-1 py-0.5 text-[11px] font-medium">
      <SplitSquareHorizontal className="size-2.5" aria-hidden="true" />
      Split · {count} categories
    </span>
  );
}

export interface TransactionRowProps {
  transaction: Transaction;
  variant?: "default" | "account-suppressed";
  showRunningBalance?: boolean;
  runningBalance?: number | null;
  baseCurrency: string;
  accountName?: string;
  categoryName?: string;
  categoryColor?: string;
  duplicateConfidence?: number;
  onClick?: () => void;
}

export function TransactionRow({
  transaction,
  variant = "default",
  showRunningBalance = false,
  runningBalance,
  baseCurrency,
  accountName,
  categoryName,
  categoryColor,
  duplicateConfidence,
  onClick,
}: TransactionRowProps) {
  const { icon: Icon, className: iconClass } = DIRECTION_ICONS[transaction.direction];
  const showFx = transaction.currency !== baseCurrency && transaction.fxRate != null;
  const baseEquivalent = showFx ? transaction.amount * (transaction.fxRate ?? 1) : null;
  const sign =
    transaction.direction === "INCOME" ? "+" : transaction.direction === "EXPENSE" ? "−" : "";
  const amountClass =
    transaction.direction === "INCOME"
      ? "text-success"
      : transaction.direction === "TRANSFER"
        ? "text-muted-foreground"
        : "text-foreground";

  const duplicateTint = getDuplicateTint(duplicateConfidence);
  const isPairedTransfer =
    transaction.transferGroupId !== null && transaction.direction === "TRANSFER";

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "hover:bg-card/50 flex w-full items-center gap-3 px-4 py-3 text-left transition-colors duration-150",
        duplicateTint,
      )}
      data-testid="transaction-row"
      aria-label={`Transaction: ${transaction.payee ?? "Transfer"} ${sign}${transaction.amount} ${transaction.currency}`}
    >
      <Icon className={cn("size-4 shrink-0", iconClass)} aria-hidden="true" />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="truncate text-base font-semibold">
            {transaction.payee ?? "Transfer"}
          </span>
          {transaction.notes && (
            <MessageSquare
              className="text-muted-foreground/60 size-3 shrink-0"
              aria-label="has note"
            />
          )}
        </div>
        <div className="text-muted-foreground flex flex-wrap items-center gap-1.5 text-xs">
          {transaction.hasSplits ? (
            <SplitBadge count={transaction.splits.length} />
          ) : (
            <CategoryChip name={categoryName} color={categoryColor} />
          )}
          {variant !== "account-suppressed" && accountName && (
            <>
              <span aria-hidden="true">·</span>
              <span>{accountName}</span>
            </>
          )}
          <span aria-hidden="true">·</span>
          <span>{transaction.currency}</span>
        </div>
      </div>
      <div className="shrink-0 text-right">
        <div className={cn("text-base font-semibold tabular-nums", amountClass)}>
          {sign}
          <PrivacyAmount value={transaction.amount} currency={transaction.currency} />
        </div>
        {showFx && (
          <div className="text-muted-foreground text-xs tabular-nums">
            ~<PrivacyAmount value={baseEquivalent ?? 0} currency={baseCurrency} />
          </div>
        )}
        <div className="flex items-center justify-end gap-1">
          {showRunningBalance && runningBalance != null && (
            <div className="text-muted-foreground text-xs tabular-nums">
              Bal <PrivacyAmount value={runningBalance} currency={transaction.currency} />
            </div>
          )}
          {isPairedTransfer && (
            <span aria-label="transfer pair" className="text-muted-foreground ml-1.5 text-xs">
              &#x2194;
            </span>
          )}
        </div>
      </div>
    </button>
  );
}
