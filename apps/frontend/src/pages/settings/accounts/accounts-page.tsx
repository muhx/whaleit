import { getPlatforms } from "@/features/connect";
import { useAccounts } from "@/hooks/use-accounts";
import { defaultGroupForAccountType } from "@/lib/constants";
import { QueryKeys } from "@/lib/query-keys";
import type { Account, Platform } from "@/lib/types";
import { useQuery } from "@tanstack/react-query";
import {
  Button,
  EmptyPlaceholder,
  Icons,
  Separator,
  Skeleton,
  Switch,
  ToggleGroup,
  ToggleGroupItem,
} from "@whaleit/ui";
import { Input } from "@whaleit/ui/components/ui/input";
import { useMemo, useState } from "react";
import { SettingsHeader } from "../settings-header";
import { AccountEditModal } from "./components/account-edit-modal";
import { AccountItem } from "./components/account-item";
import { useAccountMutations } from "./components/use-account-mutations";

const GROUP_ORDER = [
  "Banking",
  "Credit Cards",
  "Loans",
  "Investments",
  "Cash",
  "Crypto",
  "Uncategorized",
] as const;

type FilterType = "all" | "active" | "archived" | "hidden";

const SettingsAccountsPage = () => {
  const { accounts, isLoading } = useAccounts({ filterActive: false, includeArchived: true });

  const { data: platforms } = useQuery<Platform[], Error>({
    queryKey: [QueryKeys.PLATFORMS],
    queryFn: getPlatforms,
  });

  // Create a map of platform ID to platform for quick lookup
  const platformMap = useMemo(() => {
    if (!platforms) return new Map<string, Platform>();
    return new Map(platforms.map((p) => [p.id, p]));
  }, [platforms]);

  const [visibleModal, setVisibleModal] = useState(false);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [filter, setFilter] = useState<FilterType>("all");
  const [showArchived, setShowArchived] = useState(false);

  const handleAddAccount = () => {
    setSelectedAccount(null);
    setVisibleModal(true);
  };

  const { deleteAccountMutation, updateAccountMutation } = useAccountMutations({});

  const handleEditAccount = (account: Account) => {
    setSelectedAccount(account);
    setVisibleModal(true);
  };

  const handleDeleteAccount = (account: Account) => {
    deleteAccountMutation.mutate(account.id);
  };

  const handleArchiveAccount = (account: Account, archive: boolean) => {
    updateAccountMutation.mutate({
      ...account,
      isArchived: archive,
    });
  };

  const handleHideAccount = (account: Account, hide: boolean) => {
    updateAccountMutation.mutate({
      ...account,
      isActive: !hide,
    });
  };

  // Filter and search accounts
  const filteredAccounts = useMemo(() => {
    let result = accounts;

    // Apply search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (account) =>
          account.name.toLowerCase().includes(query) ||
          account.group?.toLowerCase().includes(query) ||
          account.currency.toLowerCase().includes(query),
      );
    }

    // Apply status filter
    switch (filter) {
      case "active":
        result = result.filter((a) => a.isActive && !a.isArchived);
        break;
      case "archived":
        result = result.filter((a) => a.isArchived);
        break;
      case "hidden":
        result = result.filter((a) => !a.isActive && !a.isArchived);
        break;
      default:
        break;
    }

    return result;
  }, [accounts, searchQuery, filter]);

  // Apply Show-archived gate when "all" filter is selected. The other filter
  // values explicitly opt-in to archived/hidden, so the Switch only narrows
  // the default "all" view (D-19 amended).
  const visibleAccounts = useMemo(() => {
    if (filter !== "all" || showArchived) return filteredAccounts;
    return filteredAccounts.filter((a) => !a.isArchived);
  }, [filteredAccounts, filter, showArchived]);

  // Split accounts into active and hidden/archived sections
  const { activeAccounts, inactiveAccounts } = useMemo(() => {
    const active = visibleAccounts.filter((a) => a.isActive && !a.isArchived);
    // Sort inactive: hidden first, then archived
    const inactive = visibleAccounts
      .filter((a) => !a.isActive || a.isArchived)
      .sort((a, b) => {
        // Hidden (not archived) comes before archived
        if (a.isArchived && !b.isArchived) return 1;
        if (!a.isArchived && b.isArchived) return -1;
        return 0;
      });
    return { activeAccounts: active, inactiveAccounts: inactive };
  }, [visibleAccounts]);

  // Group active accounts by account.group ?? defaultGroupForAccountType,
  // ordered per the canonical group order with custom groups appended alpha.
  const activeGroups = useMemo(() => {
    const buckets = new Map<string, Account[]>();
    for (const acc of activeAccounts) {
      const key = acc.group ?? defaultGroupForAccountType(acc.accountType) ?? "Uncategorized";
      const list = buckets.get(key) ?? [];
      list.push(acc);
      buckets.set(key, list);
    }
    const ordered = GROUP_ORDER.filter((g) => buckets.has(g)).map((g) => ({
      name: g,
      accounts: buckets.get(g) ?? [],
    }));
    const custom = [...buckets.keys()]
      .filter((g) => !GROUP_ORDER.includes(g as (typeof GROUP_ORDER)[number]))
      .sort()
      .map((g) => ({ name: g, accounts: buckets.get(g) ?? [] }));
    return [...ordered, ...custom];
  }, [activeAccounts]);

  // Counts for section headers
  const counts = useMemo(
    () => ({
      active: accounts.filter((a) => a.isActive && !a.isArchived).length,
      hidden: accounts.filter((a) => !a.isActive && !a.isArchived).length,
      archived: accounts.filter((a) => a.isArchived).length,
    }),
    [accounts],
  );

  if (isLoading) {
    return (
      <div>
        <Skeleton className="h-12" />
        <Skeleton className="h-12" />
      </div>
    );
  }

  const renderAccountItem = (account: Account) => (
    <AccountItem
      key={account.id}
      account={account}
      platform={account.platformId ? platformMap.get(account.platformId) : undefined}
      onEdit={handleEditAccount}
      onDelete={handleDeleteAccount}
      onArchive={handleArchiveAccount}
      onHide={handleHideAccount}
    />
  );

  const showSections = filter === "all";

  return (
    <>
      <div className="space-y-6">
        <SettingsHeader heading="Accounts" text=" Manage your investment and saving accounts.">
          {/* Mobile: icon button; Desktop: full button */}
          <>
            <Button
              size="icon"
              className="sm:hidden"
              onClick={() => handleAddAccount()}
              aria-label="Add account"
            >
              <Icons.Plus className="h-4 w-4" />
            </Button>
            <Button size="sm" className="hidden sm:inline-flex" onClick={() => handleAddAccount()}>
              <Icons.Plus className="mr-2 h-4 w-4" />
              Add account
            </Button>
          </>
        </SettingsHeader>
        <Separator />

        {/* Search and Filter Bar */}
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="relative flex-1 sm:max-w-sm">
            <Icons.Search className="text-muted-foreground absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2" />
            <Input
              type="text"
              placeholder="Search accounts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="!h-9 pl-9 pr-9 text-sm"
            />
            {searchQuery && (
              <button
                type="button"
                onClick={() => setSearchQuery("")}
                className="text-muted-foreground hover:text-foreground absolute right-3 top-1/2 -translate-y-1/2"
                aria-label="Clear search"
              >
                <Icons.Close className="h-4 w-4" />
              </button>
            )}
          </div>

          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <Switch
                id="show-archived"
                checked={showArchived}
                onCheckedChange={setShowArchived}
                aria-describedby="show-archived-desc"
              />
              <label htmlFor="show-archived" className="text-sm">
                Show archived
              </label>
              <span id="show-archived-desc" className="sr-only">
                Reveal accounts you&apos;ve set aside
              </span>
            </div>

            <ToggleGroup
              type="single"
              value={filter}
              onValueChange={(value) => value && setFilter(value as FilterType)}
              className="bg-muted h-9 rounded-md p-1"
            >
              <ToggleGroupItem
                value="all"
                className="data-[state=on]:bg-background h-7 rounded px-3 text-xs"
              >
                All
              </ToggleGroupItem>
              <ToggleGroupItem
                value="active"
                className="data-[state=on]:bg-background h-7 rounded px-3 text-xs"
              >
                Active
              </ToggleGroupItem>
              <ToggleGroupItem
                value="hidden"
                className="data-[state=on]:bg-background h-7 rounded px-3 text-xs"
              >
                Hidden
              </ToggleGroupItem>
              <ToggleGroupItem
                value="archived"
                className="data-[state=on]:bg-background h-7 rounded px-3 text-xs"
              >
                Archived
              </ToggleGroupItem>
            </ToggleGroup>
          </div>
        </div>

        {/* Account Lists */}
        <div className="space-y-8">
          {accounts.length === 0 ? (
            <EmptyPlaceholder>
              <EmptyPlaceholder.Icon name="Wallet" />
              <EmptyPlaceholder.Title>No account added!</EmptyPlaceholder.Title>
              <EmptyPlaceholder.Description>
                You don&apos;t have any account yet. Start adding your investment accounts.
              </EmptyPlaceholder.Description>
              <Button onClick={() => handleAddAccount()}>
                <Icons.Plus className="mr-2 h-4 w-4" />
                Add an account
              </Button>
            </EmptyPlaceholder>
          ) : visibleAccounts.length === 0 ? (
            <div className="text-muted-foreground py-8 text-center">
              No accounts match your search.
            </div>
          ) : showSections ? (
            <>
              {/* Active Accounts grouped by account.group ?? defaultGroupForAccountType */}
              {activeGroups.length > 0 && (
                <div className="space-y-6">
                  {activeGroups.map((g) => (
                    <div key={g.name} className="space-y-3">
                      <div className="flex items-center gap-2">
                        <h3 className="text-muted-foreground text-sm font-medium">{g.name}</h3>
                        <span className="bg-success/20 text-success rounded-full px-2 py-0.5 text-xs font-medium">
                          {g.accounts.length} {g.accounts.length === 1 ? "account" : "accounts"}
                        </span>
                      </div>
                      <div className="divide-border bg-card divide-y rounded-md border">
                        {g.accounts.map(renderAccountItem)}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {/* Hidden & Archived Section */}
              {inactiveAccounts.length > 0 && (
                <div className="space-y-3">
                  <div className="flex items-center gap-2">
                    <h3 className="text-muted-foreground text-sm font-medium">Hidden & Archived</h3>
                    <span className="bg-muted text-muted-foreground rounded-full px-2 py-0.5 text-xs font-medium">
                      {counts.hidden + counts.archived}
                    </span>
                  </div>
                  <div className="divide-border bg-card divide-y rounded-md border">
                    {inactiveAccounts.map(renderAccountItem)}
                  </div>
                </div>
              )}
            </>
          ) : (
            /* Flat list when filtered */
            <div className="divide-border bg-card divide-y rounded-md border">
              {visibleAccounts.map(renderAccountItem)}
            </div>
          )}
        </div>
      </div>
      <AccountEditModal
        account={selectedAccount || undefined}
        open={visibleModal}
        onClose={() => setVisibleModal(false)}
      />
    </>
  );
};

export default SettingsAccountsPage;
