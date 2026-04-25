import { useState } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@whaleit/ui/components/ui/dialog";
import { toast } from "@whaleit/ui/components/ui/use-toast";
import { MoneyInput, PrivacyAmount } from "@whaleit/ui";

import type { Account } from "@/lib/types";
import { useAccountMutations } from "./use-account-mutations";

interface Props {
  account: Account;
  open: boolean;
  onClose: () => void;
}

/**
 * "Update balance" modal — writes `current_balance` and triggers the server-side
 * auto-bump of `balance_updated_at` (Plan 03-04, D-12). Note field is omitted
 * because the Phase 3 service does not persist notes (UI-SPEC §5 dead-UI rule).
 */
export function UpdateBalanceModal({ account, open, onClose }: Props) {
  const [newBalance, setNewBalance] = useState<number | undefined>(account.currentBalance);
  const { updateAccountMutation } = useAccountMutations({});

  const unchanged = newBalance === undefined || newBalance === account.currentBalance;

  const handleSave = () => {
    if (newBalance === undefined) return;
    updateAccountMutation.mutate(
      {
        id: account.id,
        name: account.name,
        accountType: account.accountType,
        group: account.group,
        currency: account.currency,
        isDefault: account.isDefault,
        isActive: account.isActive,
        isArchived: account.isArchived,
        trackingMode: account.trackingMode,
        // Carry every Phase 3 field through unchanged so the server's auto-bump
        // (Plan 03-04) sees current_balance changed alone.
        institution: account.institution,
        openingBalance: account.openingBalance,
        currentBalance: newBalance,
        creditLimit: account.creditLimit,
        statementCycleDay: account.statementCycleDay,
        statementBalance: account.statementBalance,
        minimumPayment: account.minimumPayment,
        statementDueDate: account.statementDueDate,
        rewardPointsBalance: account.rewardPointsBalance,
        cashbackBalance: account.cashbackBalance,
      },
      {
        onSuccess: () => {
          toast({ title: "Balance updated just now", variant: "success" });
          onClose();
        },
        onError: () => {
          toast({
            title: "Balance didn't save. Check your connection and try again.",
            variant: "destructive",
          });
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="sm:max-w-[450px]">
        <DialogHeader>
          <DialogTitle>Update balance</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <p className="text-muted-foreground text-xs">
            Current balance{" "}
            <PrivacyAmount value={account.currentBalance ?? 0} currency={account.currency} />
            {account.balanceUpdatedAt
              ? ` as of ${new Date(account.balanceUpdatedAt).toLocaleDateString()}`
              : " (not yet recorded)"}
          </p>
          <MoneyInput
            value={newBalance}
            onValueChange={(v) => setNewBalance(v ?? undefined)}
            autoFocus
          />
          <p className="text-muted-foreground text-xs">
            Phase 4 will reconcile this with transactions once imported. For now this is a manual
            snapshot.
          </p>
        </div>
        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={unchanged || updateAccountMutation.isPending}>
            Save balance
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
