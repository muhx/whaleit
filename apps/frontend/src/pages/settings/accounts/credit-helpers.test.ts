import { describe, it, expect } from "vitest";
import { availableCredit, utilizationPercent, utilizationTier } from "./credit-helpers";

describe("availableCredit", () => {
  it("returns undefined when limit is missing", () => {
    expect(availableCredit(undefined, 100)).toBeUndefined();
  });
  it("treats missing balance as 0", () => {
    expect(availableCredit(1000, undefined)).toBe(1000);
  });
  it("subtracts balance from limit", () => {
    expect(availableCredit(1000, 250)).toBe(750);
  });
  it("clamps to 0 when balance exceeds limit", () => {
    expect(availableCredit(1000, 1200)).toBe(0);
  });
  it("returns undefined for non-finite limit", () => {
    expect(availableCredit(Number.NaN, 100)).toBeUndefined();
  });
});

describe("utilizationPercent", () => {
  it("returns undefined when limit is 0", () => {
    expect(utilizationPercent(0, 100)).toBeUndefined();
  });
  it("returns 25 for 250/1000", () => {
    expect(utilizationPercent(1000, 250)).toBe(25);
  });
  it("clamps to 100 when balance > limit", () => {
    expect(utilizationPercent(1000, 1500)).toBe(100);
  });
  it("treats missing balance as 0%", () => {
    expect(utilizationPercent(1000, undefined)).toBe(0);
  });
  it("returns undefined when limit missing", () => {
    expect(utilizationPercent(undefined, 100)).toBeUndefined();
  });
});

describe("utilizationTier", () => {
  it("classifies 10% as success", () => {
    expect(utilizationTier(10)).toBe("success");
  });
  it("classifies 50% as warning", () => {
    expect(utilizationTier(50)).toBe("warning");
  });
  it("classifies 95% as destructive", () => {
    expect(utilizationTier(95)).toBe("destructive");
  });
  it("returns undefined for undefined input", () => {
    expect(utilizationTier(undefined)).toBeUndefined();
  });
});
