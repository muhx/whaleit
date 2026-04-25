import { describe, it, expect } from "vitest";
import { AccountType, AccountKind, accountKind, defaultGroupForAccountType } from "./constants";

describe("accountKind", () => {
  it("classifies CHECKING, SAVINGS, CASH as ASSET", () => {
    expect(accountKind(AccountType.CHECKING)).toBe(AccountKind.ASSET);
    expect(accountKind(AccountType.SAVINGS)).toBe(AccountKind.ASSET);
    expect(accountKind(AccountType.CASH)).toBe(AccountKind.ASSET);
  });

  it("classifies CREDIT_CARD, LOAN as LIABILITY", () => {
    expect(accountKind(AccountType.CREDIT_CARD)).toBe(AccountKind.LIABILITY);
    expect(accountKind(AccountType.LOAN)).toBe(AccountKind.LIABILITY);
  });

  it("classifies SECURITIES, CRYPTOCURRENCY as INVESTMENT", () => {
    expect(accountKind(AccountType.SECURITIES)).toBe(AccountKind.INVESTMENT);
    expect(accountKind(AccountType.CRYPTOCURRENCY)).toBe(AccountKind.INVESTMENT);
  });
});

describe("defaultGroupForAccountType", () => {
  it("returns 'Banking' for CHECKING and SAVINGS", () => {
    expect(defaultGroupForAccountType(AccountType.CHECKING)).toBe("Banking");
    expect(defaultGroupForAccountType(AccountType.SAVINGS)).toBe("Banking");
  });

  it("returns 'Credit Cards' for CREDIT_CARD", () => {
    expect(defaultGroupForAccountType(AccountType.CREDIT_CARD)).toBe("Credit Cards");
  });

  it("returns 'Loans' for LOAN", () => {
    expect(defaultGroupForAccountType(AccountType.LOAN)).toBe("Loans");
  });

  it("preserves existing mappings", () => {
    expect(defaultGroupForAccountType(AccountType.SECURITIES)).toBe("Investments");
    expect(defaultGroupForAccountType(AccountType.CASH)).toBe("Cash");
    expect(defaultGroupForAccountType(AccountType.CRYPTOCURRENCY)).toBe("Crypto");
  });
});
