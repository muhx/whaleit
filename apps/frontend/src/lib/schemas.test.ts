import { describe, expect, it } from "vitest";
import { QuoteMode } from "./constants";
import { importActivitySchema, importMappingSchema, newAccountSchema } from "./schemas";

describe("schemas", () => {
  describe("importMappingSchema", () => {
    it("should accept valid quoteMode values in symbolMappingMeta", () => {
      const validMapping = {
        accountId: "test-account",
        symbolMappingMeta: {
          AAPL: {
            quoteMode: QuoteMode.MARKET,
          },
          CUSTOM: {
            quoteMode: QuoteMode.MANUAL,
          },
        },
      };

      const result = importMappingSchema.safeParse(validMapping);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.symbolMappingMeta?.AAPL.quoteMode).toBe(QuoteMode.MARKET);
        expect(result.data.symbolMappingMeta?.CUSTOM.quoteMode).toBe(QuoteMode.MANUAL);
      }
    });

    it("should reject invalid quoteMode values in symbolMappingMeta", () => {
      const invalidMapping = {
        accountId: "test-account",
        symbolMappingMeta: {
          AAPL: {
            quoteMode: "INVALID_MODE",
          },
        },
      };

      const result = importMappingSchema.safeParse(invalidMapping);
      expect(result.success).toBe(false);
    });

    it("should allow missing quoteMode in symbolMappingMeta", () => {
      const mappingWithoutQuoteMode = {
        accountId: "test-account",
        symbolMappingMeta: {
          AAPL: {
            exchangeMic: "XNAS",
            symbolName: "Apple Inc.",
          },
        },
      };

      const result = importMappingSchema.safeParse(mappingWithoutQuoteMode);
      expect(result.success).toBe(true);
    });
  });

  describe("importActivitySchema", () => {
    it("should accept valid quoteMode values", () => {
      const marketActivity = {
        accountId: "account-1",
        activityType: "BUY",
        date: "2024-01-15",
        symbol: "AAPL",
        unitPrice: 150.5,
        isDraft: false,
        quoteMode: QuoteMode.MARKET,
      };

      let result = importActivitySchema.safeParse(marketActivity);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.quoteMode).toBe(QuoteMode.MARKET);
      }

      const manualActivity = {
        accountId: "account-1",
        activityType: "BUY",
        date: "2024-01-15",
        symbol: "CUSTOM",
        unitPrice: 100.0,
        isDraft: false,
        quoteMode: QuoteMode.MANUAL,
      };

      result = importActivitySchema.safeParse(manualActivity);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.quoteMode).toBe(QuoteMode.MANUAL);
      }
    });

    it("should reject invalid quoteMode values", () => {
      const invalidActivity = {
        accountId: "account-1",
        activityType: "BUY",
        date: "2024-01-15",
        symbol: "AAPL",
        unitPrice: 150.5,
        isDraft: false,
        quoteMode: "INVALID_MODE",
      };

      const result = importActivitySchema.safeParse(invalidActivity);
      expect(result.success).toBe(false);
    });

    it("should allow missing quoteMode", () => {
      const activityWithoutQuoteMode = {
        accountId: "account-1",
        activityType: "BUY",
        date: "2024-01-15",
        symbol: "AAPL",
        unitPrice: 150.5,
        isDraft: false,
      };

      const result = importActivitySchema.safeParse(activityWithoutQuoteMode);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.quoteMode).toBeUndefined();
      }
    });
  });
});

const baseBank = {
  name: "Chase Checking",
  accountType: "CHECKING" as const,
  currency: "USD",
  openingBalance: "0",
};

const baseCC = {
  name: "Amex Gold",
  accountType: "CREDIT_CARD" as const,
  currency: "USD",
  openingBalance: "0",
  creditLimit: "5000",
  statementCycleDay: 15,
};

describe("newAccountSchema", () => {
  it("accepts a valid CHECKING account", () => {
    expect(newAccountSchema.safeParse(baseBank).success).toBe(true);
  });

  it("accepts a valid CREDIT_CARD account", () => {
    expect(newAccountSchema.safeParse(baseCC).success).toBe(true);
  });

  it("rejects CREDIT_CARD missing creditLimit", () => {
    const { creditLimit: _, ...rest } = baseCC;
    void _;
    const result = newAccountSchema.safeParse(rest);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((i) => i.path.includes("creditLimit"))).toBe(true);
    }
  });

  it("rejects CREDIT_CARD with statementCycleDay = 32", () => {
    const result = newAccountSchema.safeParse({ ...baseCC, statementCycleDay: 32 });
    expect(result.success).toBe(false);
  });

  it("rejects CHECKING with creditLimit set", () => {
    const result = newAccountSchema.safeParse({ ...baseBank, creditLimit: "1000" });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((i) => i.path.includes("creditLimit"))).toBe(true);
    }
  });

  it("rejects CHECKING missing openingBalance", () => {
    const { openingBalance: _, ...rest } = baseBank;
    void _;
    const result = newAccountSchema.safeParse(rest);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((i) => i.path.includes("openingBalance"))).toBe(true);
    }
  });
});
