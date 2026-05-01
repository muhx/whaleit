import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Page, PageContent, PageHeader } from "@whaleit/ui";
import { Button } from "@whaleit/ui/components/ui/button";
import { useTransactionSearch } from "@/hooks/use-transactions";
import { useAccounts } from "@/hooks/use-accounts";
import { useTaxonomy } from "@/hooks/use-taxonomies";
import { useSettingsContext } from "@/lib/settings-provider";
import { FilterBar } from "./filter-bar/filter-bar";
import { TransactionList } from "./transaction-list";
import { DuplicateBanner } from "./duplicate-banner";
import { TransactionDetailSheet } from "./transaction-detail-sheet";
import type { Transaction, TransactionFilters } from "@/lib/types/transaction";

const TRANSACTION_TAXONOMY_ID = "sys_taxonomy_transaction_categories";

export default function TransactionsPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { settings } = useSettingsContext();
  const baseCurrency = settings?.baseCurrency ?? "USD";

  const initialAccountId = searchParams.get("accountId");
  const [filters, setFilters] = useState<TransactionFilters>({
    accountIds: initialAccountId ? [initialAccountId] : [],
    dateFrom: undefined,
    dateTo: undefined,
    categoryIds: [],
    directions: [],
    showTransfers: true,
    source: undefined,
  });
  const [searchKw, setSearchKw] = useState("");
  const [page] = useState(0);
  const [selected, setSelected] = useState<Transaction | null>(null);

  const { data: searchResult, isLoading } = useTransactionSearch(filters, page, 50, searchKw);
  const { accounts = [] } = useAccounts();
  const { data: taxonomy } = useTaxonomy(TRANSACTION_TAXONOMY_ID);
  const categories = taxonomy?.categories ?? [];

  const showRunningBalance = (filters.accountIds?.length ?? 0) === 1;

  // pendingDuplicateCount: wired by plan 04-09 — ships as 0 here
  const pendingDupes = 0;

  return (
    <Page>
      <PageHeader
        title="Transactions"
        actions={
          <>
            <Button variant="outline" size="sm" onClick={() => navigate("/transactions/import")}>
              Import
            </Button>
            <Button size="sm" onClick={() => navigate("/transactions?new=1")}>
              New transaction
            </Button>
          </>
        }
      />
      <PageContent className="p-0">
        <FilterBar
          filters={filters}
          onChange={setFilters}
          searchKeyword={searchKw}
          onSearchChange={setSearchKw}
          accounts={accounts}
        />
        <DuplicateBanner
          pendingCount={pendingDupes}
          onReview={() => navigate("/transactions/duplicates")}
        />
        <TransactionList
          transactions={searchResult?.items ?? []}
          accounts={accounts}
          categories={categories}
          baseCurrency={baseCurrency}
          showRunningBalance={showRunningBalance}
          isLoading={isLoading}
          onRowClick={setSelected}
        />
        <TransactionDetailSheet
          transaction={selected}
          baseCurrency={baseCurrency}
          accounts={accounts}
          onClose={() => setSelected(null)}
          onEdit={(txn) => {
            setSelected(null);
            navigate(`/transactions?edit=${txn.id}`);
          }}
        />
      </PageContent>
    </Page>
  );
}
