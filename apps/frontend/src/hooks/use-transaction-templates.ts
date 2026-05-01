import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  deleteTransactionTemplate,
  getTransactionTemplate,
  listTransactionTemplates,
  saveTransactionTemplate,
} from "@/adapters/shared/transactions";
import { QueryKeys } from "@/lib/query-keys";
import type { SaveTransactionTemplateRequest, TransactionTemplate } from "@/lib/types/transaction";

export function useTransactionTemplates() {
  return useQuery<TransactionTemplate[]>({
    queryKey: QueryKeys.TRANSACTION_TEMPLATES,
    queryFn: () => listTransactionTemplates(),
  });
}

export function useTransactionTemplate(id: string | null) {
  return useQuery<TransactionTemplate>({
    queryKey: QueryKeys.TRANSACTION_TEMPLATE(id ?? ""),
    queryFn: () => getTransactionTemplate(id!),
    enabled: !!id,
  });
}

export function useSaveTransactionTemplate() {
  const qc = useQueryClient();
  return useMutation<TransactionTemplate, Error, SaveTransactionTemplateRequest>({
    mutationFn: (req) => saveTransactionTemplate(req),
    onSuccess: () => qc.invalidateQueries({ queryKey: QueryKeys.TRANSACTION_TEMPLATES }),
  });
}

export function useDeleteTransactionTemplate() {
  const qc = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: (id) => deleteTransactionTemplate(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: QueryKeys.TRANSACTION_TEMPLATES }),
  });
}
