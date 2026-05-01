// Zod validation schemas for transaction domain (Phase 4, plan 04-05).
//
// Money fields use z.number() — wire format is JSON number (rust_decimal serde-float).
// Client schemas are UX-only; server re-validates in NewTransaction::validate() (T-04-05).

import { z } from "zod";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const todayPlus1Day = () => {
  const d = new Date();
  d.setDate(d.getDate() + 1);
  return d.toISOString().slice(0, 10);
};

// ---------------------------------------------------------------------------
// Split schema
// ---------------------------------------------------------------------------

export const NewSplitSchema = z.object({
  categoryId: z.string().min(1, "Category is required"),
  amount: z.number().positive("Amount must be greater than 0"),
  notes: z.string().nullable().optional(),
  sortOrder: z.number().int().nonnegative(),
});

// ---------------------------------------------------------------------------
// Base transaction fields shared by income/expense
// ---------------------------------------------------------------------------

const baseFields = {
  accountId: z.string().min(1, "Account is required"),
  amount: z.number().positive("Amount must be greater than 0"),
  currency: z.string().min(1, "Currency is required"),
  transactionDate: z.string().refine((d) => d <= todayPlus1Day(), {
    message: "Date can't be more than a day in the future",
  }),
  notes: z.string().max(2000).nullable().optional(),
  fxRate: z.number().positive().nullable().optional(),
  fxRateSource: z.enum(["SYSTEM", "MANUAL_OVERRIDE"]).nullable().optional(),
} as const;

// ---------------------------------------------------------------------------
// Income / Expense transaction
// ---------------------------------------------------------------------------

export const NewIncomeOrExpenseTransactionSchema = z
  .object({
    ...baseFields,
    direction: z.enum(["INCOME", "EXPENSE"]),
    payee: z.string().min(1, "Payee is required"),
    categoryId: z.string().min(1, "Category is required").nullable().optional(),
    hasSplits: z.boolean().default(false),
    splits: z.array(NewSplitSchema).default([]),
    source: z.enum(["MANUAL", "CSV", "OFX", "SYSTEM"]).default("MANUAL"),
  })
  .superRefine((val, ctx) => {
    if (val.hasSplits) {
      if (val.splits.length < 2) {
        ctx.addIssue({
          code: "custom",
          path: ["splits"],
          message: "At least 2 splits required",
        });
      }
      const sum = val.splits.reduce((acc, s) => acc + s.amount, 0);
      if (Math.abs(sum - val.amount) > 0.005) {
        ctx.addIssue({
          code: "custom",
          path: ["splits"],
          message: "Split totals must equal the transaction amount.",
        });
      }
    } else if (!val.categoryId) {
      ctx.addIssue({
        code: "custom",
        path: ["categoryId"],
        message: "Category is required",
      });
    }
  });

// ---------------------------------------------------------------------------
// Transfer transaction
// ---------------------------------------------------------------------------

export const NewTransferTransactionSchema = z
  .object({
    sourceAccountId: z.string().min(1, "Source account is required"),
    destinationAccountId: z.string().min(1, "Destination account is required"),
    amount: z.number().positive("Amount must be greater than 0"),
    currency: z.string().min(1, "Currency is required"),
    transactionDate: baseFields.transactionDate,
    notes: baseFields.notes,
  })
  .superRefine((val, ctx) => {
    if (val.sourceAccountId === val.destinationAccountId) {
      ctx.addIssue({
        code: "custom",
        path: ["destinationAccountId"],
        message: "Source and destination accounts must be different.",
      });
    }
  });

// ---------------------------------------------------------------------------
// Transaction update
// ---------------------------------------------------------------------------

export const TransactionUpdateSchema = z.object({
  id: z.string(),
  direction: z.enum(["INCOME", "EXPENSE", "TRANSFER"]).optional(),
  amount: z.number().positive().optional(),
  currency: z.string().min(1).optional(),
  transactionDate: z.string().optional(),
  payee: z.string().nullable().optional(),
  notes: z.string().max(2000).nullable().optional(),
  categoryId: z.string().nullable().optional(),
  hasSplits: z.boolean().optional(),
  fxRate: z.number().positive().nullable().optional(),
  fxRateSource: z.enum(["SYSTEM", "MANUAL_OVERRIDE"]).nullable().optional(),
  splits: z.array(NewSplitSchema).optional(),
});

// ---------------------------------------------------------------------------
// CSV field mapping
// ---------------------------------------------------------------------------

export const CsvFieldMappingSchema = z
  .object({
    dateColumn: z.string().min(1, "Date column is required"),
    amountColumn: z.string().nullable().optional(),
    debitColumn: z.string().nullable().optional(),
    creditColumn: z.string().nullable().optional(),
    payeeColumn: z.string().min(1, "Payee column is required"),
    categoryColumn: z.string().nullable().optional(),
    notesColumn: z.string().nullable().optional(),
    currencyColumn: z.string().nullable().optional(),
    externalIdColumn: z.string().nullable().optional(),
    dateFormat: z.string().default("auto"),
    decimalSeparator: z.string().length(1).default("."),
    thousandsSeparator: z.string().length(1).nullable().optional(),
  })
  .superRefine((val, ctx) => {
    if (!val.amountColumn && (!val.debitColumn || !val.creditColumn)) {
      ctx.addIssue({
        code: "custom",
        path: ["amountColumn"],
        message: "Either Amount or both Debit and Credit columns are required",
      });
    }
  });

// ---------------------------------------------------------------------------
// Inferred input types
// ---------------------------------------------------------------------------

export type NewIncomeOrExpenseTransactionInput = z.input<
  typeof NewIncomeOrExpenseTransactionSchema
>;
export type NewTransferTransactionInput = z.input<typeof NewTransferTransactionSchema>;
export type TransactionUpdateInput = z.input<typeof TransactionUpdateSchema>;
