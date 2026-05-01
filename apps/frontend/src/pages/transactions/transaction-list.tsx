import { useMemo } from "react";
import { format, parseISO } from "date-fns";
import { Skeleton } from "@whaleit/ui/components/ui/skeleton";
import { TransactionRow } from "./transaction-row";
import type { Account } from "@/lib/types";
import type { TaxonomyCategory } from "@/lib/types";
import type { Transaction } from "@/lib/types/transaction";

function formatGroupHeader(date: string, count: number, netAmount: number, currency: string) {
  const formatted = format(parseISO(date + "T00:00:00"), "MMM d, yyyy");
  const sign = netAmount >= 0 ? "+" : "−";
  const abs = Math.abs(netAmount).toFixed(2);
  return `${formatted} · ${count} ${count === 1 ? "transaction" : "transactions"} · ${sign}${currency} ${abs}`;
}

interface TransactionGroup {
  date: string;
  items: Transaction[];
  netAmount: number;
  currency: string;
}

function groupByDate(transactions: Transaction[]): TransactionGroup[] {
  const map = new Map<string, Transaction[]>();
  for (const txn of transactions) {
    const key = txn.transactionDate;
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(txn);
  }

  // Sort groups DESC by date
  const sortedKeys = [...map.keys()].sort((a, b) => b.localeCompare(a));

  return sortedKeys.map((date) => {
    const items = (map.get(date) ?? []).sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    const netAmount = items.reduce((sum, t) => {
      if (t.direction === "INCOME") return sum + t.amount;
      if (t.direction === "EXPENSE") return sum - t.amount;
      return sum;
    }, 0);
    const currency = items[0]?.currency ?? "USD";
    return { date, items, netAmount, currency };
  });
}

interface TransactionListProps {
  transactions: Transaction[];
  runningBalances?: Record<string, number>;
  accounts: Account[];
  categories: TaxonomyCategory[];
  baseCurrency: string;
  showRunningBalance: boolean;
  isLoading?: boolean;
  onRowClick: (txn: Transaction) => void;
}

export function TransactionList({
  transactions,
  runningBalances,
  accounts,
  categories,
  baseCurrency,
  showRunningBalance,
  isLoading,
  onRowClick,
}: TransactionListProps) {
  const groups = useMemo(() => groupByDate(transactions), [transactions]);

  const accountById = useMemo(() => {
    const m = new Map<string, Account>();
    for (const a of accounts) m.set(a.id, a);
    return m;
  }, [accounts]);

  const categoryById = useMemo(() => {
    const m = new Map<string, TaxonomyCategory>();
    for (const c of categories) m.set(c.id, c);
    return m;
  }, [categories]);

  if (isLoading) {
    return (
      <div className="divide-y" data-testid="transaction-list-skeleton">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="flex items-center gap-3 px-4 py-3">
            <Skeleton className="size-4 rounded-full" />
            <div className="flex-1 space-y-1">
              <Skeleton className="h-4 w-40" />
              <Skeleton className="h-3 w-28" />
            </div>
            <Skeleton className="h-4 w-16" />
          </div>
        ))}
      </div>
    );
  }

  if (groups.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center px-4 py-16 text-center">
        <p className="text-base font-semibold">No transactions found</p>
        <p className="text-muted-foreground mt-1 text-sm">
          Try adjusting your filters or add your first transaction.
        </p>
      </div>
    );
  }

  return (
    <div className="divide-y" data-testid="transaction-list">
      {groups.map(({ date, items, netAmount, currency }) => (
        <div key={date}>
          <div className="bg-muted/30 text-muted-foreground sticky top-0 z-10 px-4 py-2 text-sm font-semibold">
            {formatGroupHeader(date, items.length, netAmount, currency)}
          </div>
          {items.map((txn) => {
            const account = accountById.get(txn.accountId);
            const category = txn.categoryId ? categoryById.get(txn.categoryId) : undefined;
            return (
              <TransactionRow
                key={txn.id}
                transaction={txn}
                baseCurrency={baseCurrency}
                showRunningBalance={showRunningBalance}
                runningBalance={runningBalances?.[txn.id] ?? null}
                accountName={account?.name}
                categoryName={category?.name}
                categoryColor={category?.color}
                onClick={() => onRowClick(txn)}
              />
            );
          })}
        </div>
      ))}
    </div>
  );
}
