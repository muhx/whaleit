import { useMemo } from "react";
import { format, parseISO } from "date-fns";
import { useQuery } from "@tanstack/react-query";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@whaleit/ui/components/ui/sheet";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@whaleit/ui/components/ui/alert-dialog";
import { Button } from "@whaleit/ui/components/ui/button";
import { PrivacyAmount } from "@whaleit/ui";
import { ArrowDownLeft, ArrowLeftRight, ArrowUpRight } from "lucide-react";
import { searchTransactions } from "@/adapters";
import { useDeleteTransaction } from "@/hooks/use-transactions";
import { cn } from "@/lib/utils";
import type { Account } from "@/lib/types";
import type {
  Transaction,
  TransactionDirection,
  TransactionSearchResult,
} from "@/lib/types/transaction";

const DIRECTION_ICONS: Record<
  TransactionDirection,
  { icon: React.ComponentType<{ className?: string }>; className: string }
> = {
  INCOME: { icon: ArrowDownLeft, className: "text-success" },
  EXPENSE: { icon: ArrowUpRight, className: "text-foreground" },
  TRANSFER: { icon: ArrowLeftRight, className: "text-muted-foreground" },
};

interface TransactionDetailSheetProps {
  transaction: Transaction | null;
  baseCurrency: string;
  accounts: Account[];
  onClose: () => void;
  onEdit: (transaction: Transaction) => void;
}

// D-05: Sibling fetch only happens when the detail sheet is opened on a TRANSFER
// row. Per-account list queries return only one leg.
function usePairedSibling(transaction: Transaction | null) {
  const enabled =
    transaction != null &&
    transaction.direction === "TRANSFER" &&
    transaction.transferGroupId != null;

  return useQuery<Transaction | null>({
    queryKey: [
      "transactions",
      "pair-sibling",
      transaction?.transferGroupId ?? null,
      transaction?.id ?? null,
    ],
    queryFn: async () => {
      const result: TransactionSearchResult = await searchTransactions(
        0,
        2,
        { showTransfers: true },
        "",
        undefined,
      );
      // Filter on the client — server filter for transferGroupId not exposed here.
      // For pair lookup the data is small (max 2 rows per pair).
      const sibling = result.items.find(
        (t) => t.transferGroupId === transaction!.transferGroupId && t.id !== transaction!.id,
      );
      return sibling ?? null;
    },
    enabled,
    staleTime: 60_000,
  });
}

export function TransactionDetailSheet({
  transaction,
  baseCurrency,
  accounts,
  onClose,
  onEdit,
}: TransactionDetailSheetProps) {
  const deleteMutation = useDeleteTransaction();
  const { data: sibling } = usePairedSibling(transaction);

  const accountById = useMemo(() => {
    const m = new Map<string, Account>();
    for (const a of accounts) m.set(a.id, a);
    return m;
  }, [accounts]);

  if (!transaction) {
    return (
      <Sheet open={false} onOpenChange={(open) => !open && onClose()}>
        <SheetContent />
      </Sheet>
    );
  }

  const account = accountById.get(transaction.accountId);
  const { icon: Icon, className: iconClass } = DIRECTION_ICONS[transaction.direction];

  const sign =
    transaction.direction === "INCOME" ? "+" : transaction.direction === "EXPENSE" ? "−" : "";
  const amountClass =
    transaction.direction === "INCOME"
      ? "text-success"
      : transaction.direction === "TRANSFER"
        ? "text-muted-foreground"
        : "text-foreground";

  const isTransfer = transaction.direction === "TRANSFER";
  const siblingAccount = sibling ? accountById.get(sibling.accountId) : null;

  const isOpen = transaction != null;

  async function handleDelete() {
    if (!transaction) return;
    await deleteMutation.mutateAsync(transaction.id);
    onClose();
  }

  return (
    <Sheet open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <SheetContent side="right" className="flex w-full flex-col p-0 sm:max-w-md">
        <SheetHeader className="border-b px-6 py-4">
          <SheetTitle className="flex items-center gap-2">
            <Icon className={cn("size-5", iconClass)} aria-hidden="true" />
            {transaction.payee ?? "Transfer"}
          </SheetTitle>
          <SheetDescription>
            {format(parseISO(transaction.transactionDate + "T00:00:00"), "MMMM d, yyyy")}
          </SheetDescription>
        </SheetHeader>

        <div className="flex-1 space-y-4 overflow-auto px-6 py-4">
          {/* Hero amount */}
          <div className="py-2 text-center">
            <div className={cn("text-3xl font-semibold tabular-nums", amountClass)}>
              {sign}
              <PrivacyAmount value={transaction.amount} currency={transaction.currency} />
            </div>
            {transaction.currency !== baseCurrency && transaction.fxRate != null && (
              <div className="text-muted-foreground mt-1 text-sm tabular-nums">
                ~
                <PrivacyAmount
                  value={transaction.amount * transaction.fxRate}
                  currency={baseCurrency}
                />
              </div>
            )}
          </div>

          {/* Transfer pair detail (D-03 / D-05) */}
          {isTransfer && (
            <div className="bg-muted/20 space-y-2 rounded-md border p-3">
              <div className="text-muted-foreground text-xs font-semibold uppercase">
                Transfer pair
              </div>
              <div className="flex items-center justify-between text-sm">
                <span>This leg ({account?.name ?? "Unknown"})</span>
                <span className="tabular-nums">
                  {sign}
                  <PrivacyAmount value={transaction.amount} currency={transaction.currency} />
                </span>
              </div>
              {sibling ? (
                <div className="flex items-center justify-between text-sm">
                  <span>Other leg ({siblingAccount?.name ?? "Unknown"})</span>
                  <span className="tabular-nums">
                    {sibling.direction === "INCOME" ? "+" : "−"}
                    <PrivacyAmount value={sibling.amount} currency={sibling.currency} />
                  </span>
                </div>
              ) : (
                <div className="text-muted-foreground text-xs">Pair link broken or solo leg.</div>
              )}
            </div>
          )}

          {/* Splits */}
          {transaction.hasSplits && transaction.splits.length > 0 && (
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs font-semibold uppercase">
                Split categories
              </div>
              {transaction.splits.map((split) => (
                <div key={split.id} className="flex items-center justify-between text-sm">
                  <span>{split.notes ?? split.categoryId}</span>
                  <span className="tabular-nums">
                    <PrivacyAmount value={split.amount} currency={transaction.currency} />
                  </span>
                </div>
              ))}
            </div>
          )}

          {/* Notes */}
          {transaction.notes && (
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs font-semibold uppercase">Notes</div>
              <p className="text-sm">{transaction.notes}</p>
            </div>
          )}

          {/* Account */}
          <div className="space-y-1">
            <div className="text-muted-foreground text-xs font-semibold uppercase">Account</div>
            <p className="text-sm">{account?.name ?? "Unknown"}</p>
          </div>
        </div>

        <div className="flex items-center gap-2 border-t px-6 py-4">
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="destructive" size="sm">
                Delete transaction
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Delete this transaction?</AlertDialogTitle>
                <AlertDialogDescription>This can&apos;t be undone.</AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction onClick={handleDelete}>Delete transaction</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
          <Button size="sm" className="ml-auto" onClick={() => onEdit(transaction)}>
            Edit transaction
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
