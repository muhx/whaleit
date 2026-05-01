import { useQuery } from "@tanstack/react-query";
import { listPayeeCategoryMemory, lookupPayeeCategory } from "@/adapters";
import { QueryKeys } from "@/lib/query-keys";
import type { PayeeCategoryMemory } from "@/lib/types/transaction";

/**
 * Looks up the remembered category for a payee on a given account.
 * Only fires when `payee` is non-empty and `enabled` is true (D-15 auto-fill).
 */
export function useLookupPayeeCategory(accountId: string | null, payee: string, enabled = true) {
  return useQuery<PayeeCategoryMemory | null>({
    queryKey: QueryKeys.MERCHANT_CATEGORY_LOOKUP(accountId ?? "", payee),
    queryFn: () => lookupPayeeCategory(accountId!, payee),
    enabled: enabled && !!accountId && payee.trim().length > 0,
    staleTime: 60_000, // memory is stable; cache for 1 min
  });
}

/**
 * Returns all remembered payee→category entries for an account.
 */
export function useMerchantCategoryMemory(accountId: string | null) {
  return useQuery<PayeeCategoryMemory[]>({
    queryKey: QueryKeys.MERCHANT_CATEGORIES(accountId ?? ""),
    queryFn: () => listPayeeCategoryMemory(accountId!),
    enabled: !!accountId,
  });
}
