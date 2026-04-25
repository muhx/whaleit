import { Dialog, DialogContent } from "@whaleit/ui/components/ui/dialog";
import { useIsMobileViewport } from "@/hooks/use-platform";
import { AccountType } from "@/lib/constants";
import { useSettingsContext } from "@/lib/settings-provider";
import type { Account } from "@/lib/types";
import { AccountForm } from "./account-form";

export interface AccountEditModalProps {
  account?: Account;
  open?: boolean;
  onClose?: () => void;
}

export function AccountEditModal({ account, open, onClose }: AccountEditModalProps) {
  const { settings } = useSettingsContext();

  const defaultValues = {
    id: account?.id ?? undefined,
    name: account?.name ?? "",
    currentBalance: account?.currentBalance,
    accountType: (account?.accountType ?? "SECURITIES") as AccountType,
    group: account?.group ?? undefined,
    currency: account?.currency ?? settings?.baseCurrency ?? "USD",
    isDefault: account?.isDefault ?? false,
    isActive: account?.id ? account?.isActive : true,
    isArchived: account?.isArchived ?? false,
    trackingMode: account?.trackingMode,
    meta: account?.meta,
    // Phase 3 additions (D-06, D-11, D-18) — closes gap H-01:
    institution: account?.institution,
    openingBalance: account?.openingBalance,
    creditLimit: account?.creditLimit,
    statementCycleDay: account?.statementCycleDay,
    statementBalance: account?.statementBalance,
    minimumPayment: account?.minimumPayment,
    statementDueDate: account?.statementDueDate,
    rewardPointsBalance: account?.rewardPointsBalance,
    cashbackBalance: account?.cashbackBalance,
  };

  return (
    <Dialog open={open} onOpenChange={onClose} useIsMobile={useIsMobileViewport}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-[625px]">
        <AccountForm defaultValues={defaultValues} onSuccess={onClose} />
      </DialogContent>
    </Dialog>
  );
}
