import { useMemo } from "react";
import { Link } from "react-router-dom";
import { format, parseISO } from "date-fns";
import { Card, CardContent, CardHeader, CardTitle } from "@whaleit/ui";
import { Skeleton } from "@whaleit/ui/components/ui/skeleton";
import { useAccountRecentTransactions, useRunningBalance } from "@/hooks/use-transactions";
import { useTaxonomy } from "@/hooks/use-taxonomies";
import { TransactionRow } from "./transaction-row";
import type { Transaction } from "@/lib/types/transaction";

const TRANSACTION_TAXONOMY_ID = "sys_taxonomy_transaction_categories";

interface RecentTransactionsProps {
  accountId: string;
  baseCurrency: string;
  limit?: number;
}

interface DateGroup {
  date: string;
  items: Transaction[];
}

function groupByDate(transactions: Transaction[]): DateGroup[] {
  const map = new Map<string, Transaction[]>();
  for (const txn of transactions) {
    const key = txn.transactionDate;
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(txn);
  }
  const sortedKeys = [...map.keys()].sort((a, b) => b.localeCompare(a));
  return sortedKeys.map((date) => ({
    date,
    items: (map.get(date) ?? []).sort((a, b) => b.createdAt.localeCompare(a.createdAt)),
  }));
}

export function RecentTransactions({
  accountId,
  baseCurrency,
  limit = 10,
}: RecentTransactionsProps) {
  // D-05: per-account leg query — server returns only the leg attached to accountId.
  const { data: txns = [], isLoading } = useAccountRecentTransactions(accountId, limit);
  const { data: balances = [] } = useRunningBalance(accountId);
  const { data: taxonomy } = useTaxonomy(TRANSACTION_TAXONOMY_ID);
  const categories = taxonomy?.categories ?? [];

  const balanceById = useMemo(() => {
    const m = new Map<string, number>();
    for (const b of balances) m.set(b.id, b.runningBalance);
    return m;
  }, [balances]);

  const categoryById = useMemo(() => {
    const m = new Map<string, { name: string; color: string }>();
    for (const c of categories) m.set(c.id, { name: c.name, color: c.color });
    return m;
  }, [categories]);

  const groups = useMemo(() => groupByDate(txns), [txns]);

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-md">Recent transactions</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="flex items-center gap-3 px-4 py-3">
              <Skeleton className="size-4 rounded-full" />
              <div className="flex-1 space-y-1">
                <Skeleton className="h-4 w-40" />
                <Skeleton className="h-3 w-28" />
              </div>
              <Skeleton className="h-4 w-16" />
            </div>
          ))}
        </CardContent>
      </Card>
    );
  }

  if (txns.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-md">Recent transactions</CardTitle>
        </CardHeader>
        <CardContent className="py-8 text-center">
          <h4 className="text-base font-semibold">No transactions yet</h4>
          <p className="text-muted-foreground mt-1 text-sm">
            Add your first transaction or import a CSV to start tracking this account.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0">
        <CardTitle className="text-md">Recent transactions</CardTitle>
        <Link
          to={`/transactions?accountId=${accountId}`}
          className="text-primary text-xs hover:underline"
        >
          View all →
        </Link>
      </CardHeader>
      <CardContent className="p-0">
        {groups.map(({ date, items }) => (
          <div key={date}>
            <div className="bg-muted/30 text-muted-foreground px-4 py-2 text-xs font-semibold">
              {format(parseISO(date + "T00:00:00"), "MMM d, yyyy")}
            </div>
            {items.map((t) => {
              const cat = t.categoryId ? categoryById.get(t.categoryId) : undefined;
              return (
                <TransactionRow
                  key={t.id}
                  transaction={t}
                  variant="account-suppressed"
                  showRunningBalance
                  runningBalance={balanceById.get(t.id) ?? null}
                  baseCurrency={baseCurrency}
                  categoryName={cat?.name}
                  categoryColor={cat?.color}
                />
              );
            })}
          </div>
        ))}
        <div className="border-t px-4 py-2 text-center">
          <Link
            to={`/transactions?accountId=${accountId}`}
            className="text-primary text-sm hover:underline"
          >
            View all transactions in this account →
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
