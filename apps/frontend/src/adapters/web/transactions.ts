// Web-specific transaction import commands (multipart/form-data)
//
// The shared/transactions.ts wrappers use invoke<T> (JSON-only).
// CSV and OFX import require multipart form data, so they are overridden here.
// Mirror: apps/frontend/src/adapters/web/activities.ts:parseCsv

import type {
  TransactionCsvImportRequest,
  TransactionImportResult,
  TransactionOfxImportRequest,
} from "@/lib/types/transaction";
import { API_PREFIX, logger } from "./core";

async function extractErrorMessage(response: Response): Promise<string | null> {
  const contentType = response.headers.get("content-type") ?? "";

  if (contentType.includes("application/json")) {
    try {
      const payload = (await response.json()) as {
        message?: unknown;
        error?: unknown;
      };
      if (typeof payload.message === "string" && payload.message.trim()) {
        return payload.message.trim();
      }
      if (typeof payload.error === "string" && payload.error.trim()) {
        return payload.error.trim();
      }
    } catch {
      // Fall through to text parsing
    }
  }

  try {
    const text = (await response.text()).trim();
    return text || null;
  } catch {
    return null;
  }
}

/**
 * Import transactions from a CSV file.
 * Web implementation: POSTs multipart form data to /api/v1/transactions/import/csv.
 */
export const importTransactionsCsv = async (
  req: TransactionCsvImportRequest,
): Promise<TransactionImportResult> => {
  try {
    const formData = new FormData();
    formData.append("file", req.file);
    formData.append("accountId", req.accountId);
    formData.append("mapping", JSON.stringify(req.mapping));
    if (req.templateName) {
      formData.append("templateName", req.templateName);
    }

    const response = await fetch(`${API_PREFIX}/transactions/import/csv`, {
      method: "POST",
      body: formData,
      credentials: "same-origin",
    });

    if (!response.ok) {
      const details = await extractErrorMessage(response);
      const fallback = `Request failed (${response.status}${response.statusText ? ` ${response.statusText}` : ""})`;
      throw new Error(
        details
          ? `Failed to import CSV transactions: ${details}`
          : `Failed to import CSV transactions: ${fallback}`,
      );
    }

    return (await response.json()) as TransactionImportResult;
  } catch (err) {
    logger.error("Error importing CSV transactions:", err);
    throw err;
  }
};

/**
 * Import transactions from an OFX file.
 * Web implementation: POSTs multipart form data to /api/v1/transactions/import/ofx.
 */
export const importTransactionsOfx = async (
  req: TransactionOfxImportRequest,
): Promise<TransactionImportResult> => {
  try {
    const formData = new FormData();
    formData.append("file", req.file);
    formData.append("accountId", req.accountId);

    const response = await fetch(`${API_PREFIX}/transactions/import/ofx`, {
      method: "POST",
      body: formData,
      credentials: "same-origin",
    });

    if (!response.ok) {
      const details = await extractErrorMessage(response);
      const fallback = `Request failed (${response.status}${response.statusText ? ` ${response.statusText}` : ""})`;
      throw new Error(
        details
          ? `Failed to import OFX transactions: ${details}`
          : `Failed to import OFX transactions: ${fallback}`,
      );
    }

    return (await response.json()) as TransactionImportResult;
  } catch (err) {
    logger.error("Error importing OFX transactions:", err);
    throw err;
  }
};
