// Account Commands
import type { Account } from "@/lib/types";
import type { newAccountSchema } from "@/lib/schemas";
import type z from "zod";

import { invoke, logger, isDesktop } from "./platform";

type NewAccount = z.infer<typeof newAccountSchema>;
type SerializedAccount = Omit<Account, "createdAt" | "updatedAt"> & {
  createdAt: Date | string;
  updatedAt: Date | string;
};

function normalizeAccountDates(account: SerializedAccount): Account {
  return {
    ...account,
    createdAt: account.createdAt instanceof Date ? account.createdAt : new Date(account.createdAt),
    updatedAt: account.updatedAt instanceof Date ? account.updatedAt : new Date(account.updatedAt),
  };
}

export const getAccounts = async (includeArchived?: boolean): Promise<Account[]> => {
  try {
    const accounts = await invoke<SerializedAccount[]>("get_accounts", {
      includeArchived: includeArchived ?? false,
    });
    return accounts.map(normalizeAccountDates);
  } catch (error) {
    logger.error("Error fetching accounts.");
    throw error;
  }
};

export const createAccount = async (account: NewAccount): Promise<Account> => {
  try {
    const created = await invoke<SerializedAccount>("create_account", { account });
    return normalizeAccountDates(created);
  } catch (error) {
    logger.error("Error creating account.");
    throw error;
  }
};

export const updateAccount = async (account: NewAccount): Promise<Account> => {
  try {
    // Platform-aware: desktop strips currency (immutable after creation)
    const payload = isDesktop
      ? (() => {
          const { currency: _, ...rest } = account;
          return rest;
        })()
      : account;
    const updated = await invoke<SerializedAccount>("update_account", { accountUpdate: payload });
    return normalizeAccountDates(updated);
  } catch (error) {
    logger.error("Error updating account.");
    throw error;
  }
};

export const deleteAccount = async (accountId: string): Promise<void> => {
  try {
    await invoke<void>("delete_account", { accountId });
  } catch (error) {
    logger.error("Error deleting account.");
    throw error;
  }
};
