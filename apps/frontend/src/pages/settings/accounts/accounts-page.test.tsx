import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type React from "react";
import { useAccounts } from "@/hooks/use-accounts";
import type { Account, TrackingMode } from "@/lib/types";
import { AccountType } from "@/lib/constants";
import { useQuery } from "@tanstack/react-query";
import SettingsAccountsPage from "./accounts-page";
import { AccountEditModal } from "./components/account-edit-modal";

vi.mock("@/hooks/use-accounts", () => ({
  useAccounts: vi.fn(),
}));

vi.mock("@tanstack/react-query", async (importActual) => {
  const actual = await importActual<typeof import("@tanstack/react-query")>();
  return {
    ...actual,
    useQuery: vi.fn(),
  };
});

vi.mock("./components/use-account-mutations", () => ({
  useAccountMutations: () => ({
    createAccountMutation: { mutate: vi.fn() },
    deleteAccountMutation: { mutate: vi.fn() },
    updateAccountMutation: { mutate: vi.fn() },
  }),
}));

vi.mock("./components/account-operations", () => ({
  AccountOperations: () => null,
}));

vi.mock("../settings-header", () => ({
  SettingsHeader: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: () => ({ settings: { baseCurrency: "USD" } }),
}));

vi.mock("@/hooks/use-platform", () => ({
  useIsMobileViewport: vi.fn().mockReturnValue(false),
}));

vi.mock("@whaleit/ui/components/ui/dialog", () => ({
  Dialog: ({ children, open }: { children?: React.ReactNode; open?: boolean }) =>
    open ? <div data-testid="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  DialogHeader: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children?: React.ReactNode }) => <h2>{children}</h2>,
  DialogDescription: ({ children }: { children?: React.ReactNode }) => <p>{children}</p>,
  DialogFooter: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  DialogTrigger: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
}));

vi.mock("@whaleit/ui/components/ui/button", () => ({
  Button: ({
    children,
    ...rest
  }: React.ButtonHTMLAttributes<HTMLButtonElement> & { children?: React.ReactNode }) => (
    <button {...rest}>{children}</button>
  ),
}));

vi.mock("@whaleit/ui/components/ui/checkbox", () => ({
  Checkbox: ({
    checked,
    onCheckedChange,
    ...rest
  }: {
    checked?: boolean;
    onCheckedChange?: (v: boolean) => void;
  } & React.HTMLAttributes<HTMLInputElement>) => (
    <input
      type="checkbox"
      checked={checked}
      onChange={(e) => onCheckedChange?.(e.target.checked)}
      {...rest}
    />
  ),
}));

vi.mock("@whaleit/ui/components/ui/alert", () => ({
  Alert: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDescription: ({ children }: { children?: React.ReactNode }) => <p>{children}</p>,
}));

vi.mock("@whaleit/ui/components/ui/alert-dialog", () => ({
  AlertDialog: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogContent: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogHeader: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogTitle: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogDescription: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogFooter: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AlertDialogCancel: ({ children }: { children?: React.ReactNode }) => <button>{children}</button>,
}));

vi.mock("@whaleit/ui", () => ({
  Button: ({
    children,
    ...rest
  }: React.ButtonHTMLAttributes<HTMLButtonElement> & { children?: React.ReactNode }) => (
    <button {...rest}>{children}</button>
  ),
  EmptyPlaceholder: Object.assign(
    ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
    {
      Icon: () => <span />,
      Title: ({ children }: { children?: React.ReactNode }) => <h2>{children}</h2>,
      Description: ({ children }: { children?: React.ReactNode }) => <p>{children}</p>,
    },
  ),
  Icons: new Proxy(
    {},
    {
      get: () => (props: React.SVGProps<SVGSVGElement>) => (
        <span {...(props as React.HTMLAttributes<HTMLSpanElement>)} />
      ),
    },
  ),
  Separator: () => <hr />,
  Skeleton: () => <div>loading</div>,
  Switch: ({
    checked,
    onCheckedChange,
    id,
    ...rest
  }: {
    checked: boolean;
    onCheckedChange: (v: boolean) => void;
    id?: string;
    "aria-describedby"?: string;
  }) => (
    <input
      type="checkbox"
      role="switch"
      id={id}
      checked={checked}
      onChange={(e) => onCheckedChange(e.target.checked)}
      {...rest}
    />
  ),
  ToggleGroup: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  ToggleGroupItem: ({ children, value }: { children?: React.ReactNode; value?: string }) => (
    <button data-value={value}>{children}</button>
  ),
  Tooltip: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
  TooltipProvider: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
  // AccountForm components (used by AccountEditModal regression tests):
  MoneyInput: ({
    value,
    onValueChange: _onValueChange,
    ...rest
  }: {
    value?: number | string | null;
    onValueChange?: (v: number | undefined) => void;
  } & React.InputHTMLAttributes<HTMLInputElement>) => (
    <input type="number" value={value ?? ""} onChange={() => undefined} readOnly {...rest} />
  ),
  ResponsiveSelect: ({
    value,
    onValueChange: _onValueChange,
    ...rest
  }: {
    value?: string;
    onValueChange?: (v: string) => void;
    options?: unknown[];
    placeholder?: string;
    sheetTitle?: string;
    sheetDescription?: string;
    triggerClassName?: string;
  } & React.HTMLAttributes<HTMLInputElement>) => (
    <input type="text" value={value ?? ""} onChange={() => undefined} readOnly {...rest} />
  ),
  Select: ({
    value,
    children: _children,
    onValueChange: _onValueChange,
    ...rest
  }: {
    value?: string;
    children?: React.ReactNode;
    onValueChange?: (v: string) => void;
  } & React.HTMLAttributes<HTMLInputElement>) => (
    <input type="text" value={value ?? ""} onChange={() => undefined} readOnly {...rest} />
  ),
  SelectContent: () => null,
  SelectItem: () => null,
  SelectTrigger: () => null,
  SelectValue: () => null,
  RadioGroup: ({ children, ...rest }: { children?: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
    <div {...rest}>{children}</div>
  ),
  RadioGroupItem: ({ value, ...rest }: { value?: string } & React.InputHTMLAttributes<HTMLInputElement>) => (
    <input type="radio" value={value} {...rest} />
  ),
  CurrencyInput: () => null,
  DatePickerInput: () => null,
}));

vi.mock("@whaleit/ui/components/ui/avatar", () => ({
  Avatar: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AvatarFallback: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
  AvatarImage: () => <span />,
}));

vi.mock("@whaleit/ui/components/ui/icons", () => ({
  Icons: new Proxy(
    {},
    {
      get: () => (props: React.SVGProps<SVGSVGElement>) => (
        <span {...(props as React.HTMLAttributes<HTMLSpanElement>)} />
      ),
    },
  ),
}));

vi.mock("@whaleit/ui/components/ui/skeleton", () => ({
  Skeleton: () => <div>loading</div>,
}));

vi.mock("@whaleit/ui/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
}));

const mockUseAccounts = vi.mocked(useAccounts);
const mockUseQuery = vi.mocked(useQuery);

function makeAccount(overrides: Partial<Account>): Account {
  return {
    id: overrides.id ?? "acc-1",
    name: overrides.name ?? "Acc 1",
    accountType: overrides.accountType ?? AccountType.SECURITIES,
    group: overrides.group,
    currency: overrides.currency ?? "USD",
    isDefault: false,
    isActive: overrides.isActive ?? true,
    isArchived: overrides.isArchived ?? false,
    trackingMode: "TRANSACTIONS" as TrackingMode,
    createdAt: new Date("2026-01-01T00:00:00Z"),
    updatedAt: new Date("2026-01-01T00:00:00Z"),
    creditLimit: overrides.creditLimit,
    currentBalance: overrides.currentBalance,
    ...overrides,
  } as Account;
}

beforeEach(() => {
  mockUseQuery.mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<
    typeof useQuery
  >);
});

function renderPage() {
  return render(
    <MemoryRouter>
      <SettingsAccountsPage />
    </MemoryRouter>,
  );
}

describe("SettingsAccountsPage", () => {
  it("renders all six AccountType groups when accounts of each type exist", () => {
    mockUseAccounts.mockReturnValue({
      accounts: [
        makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
        makeAccount({
          id: "2",
          name: "Amex Gold",
          accountType: AccountType.CREDIT_CARD,
          creditLimit: "5000",
          currentBalance: "1000",
        }),
        makeAccount({ id: "3", name: "Mortgage", accountType: AccountType.LOAN }),
        makeAccount({ id: "4", name: "Brokerage", accountType: AccountType.SECURITIES }),
        makeAccount({ id: "5", name: "Wallet Cash", accountType: AccountType.CASH }),
        makeAccount({ id: "6", name: "BTC Wallet", accountType: AccountType.CRYPTOCURRENCY }),
      ],
      isLoading: false,
    } as ReturnType<typeof useAccounts>);

    renderPage();

    expect(screen.getByText(/Banking/)).toBeInTheDocument();
    expect(screen.getByText(/Credit Cards/)).toBeInTheDocument();
    expect(screen.getByText(/Loans/)).toBeInTheDocument();
    expect(screen.getByText(/Investments/)).toBeInTheDocument();
    expect(screen.getByText(/^Cash$/)).toBeInTheDocument();
    expect(screen.getByText(/Crypto/)).toBeInTheDocument();
  });

  it("orders groups Banking -> Credit Cards -> Loans -> Investments -> Cash -> Crypto", () => {
    mockUseAccounts.mockReturnValue({
      accounts: [
        makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
        makeAccount({
          id: "2",
          name: "Amex Gold",
          accountType: AccountType.CREDIT_CARD,
          creditLimit: "5000",
        }),
        makeAccount({ id: "3", name: "Mortgage", accountType: AccountType.LOAN }),
        makeAccount({ id: "4", name: "Brokerage", accountType: AccountType.SECURITIES }),
        makeAccount({ id: "5", name: "Wallet Cash", accountType: AccountType.CASH }),
        makeAccount({ id: "6", name: "BTC Wallet", accountType: AccountType.CRYPTOCURRENCY }),
      ],
      isLoading: false,
    } as ReturnType<typeof useAccounts>);

    const { container } = renderPage();
    const text = container.textContent ?? "";
    const idxBanking = text.indexOf("Banking");
    const idxCC = text.indexOf("Credit Cards");
    const idxLoans = text.indexOf("Loans");
    const idxInv = text.indexOf("Investments");
    const idxCash = text.indexOf("Cash");
    const idxCrypto = text.indexOf("Crypto");

    expect(idxBanking).toBeGreaterThan(-1);
    expect(idxBanking).toBeLessThan(idxCC);
    expect(idxCC).toBeLessThan(idxLoans);
    expect(idxLoans).toBeLessThan(idxInv);
    expect(idxInv).toBeLessThan(idxCash);
    expect(idxCash).toBeLessThan(idxCrypto);
  });

  it("hides archived accounts by default", () => {
    mockUseAccounts.mockReturnValue({
      accounts: [
        makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
        makeAccount({
          id: "2",
          name: "Old Card",
          accountType: AccountType.CREDIT_CARD,
          isArchived: true,
          creditLimit: "1000",
        }),
      ],
      isLoading: false,
    } as ReturnType<typeof useAccounts>);

    renderPage();

    expect(screen.getByText("Chase Checking")).toBeInTheDocument();
    expect(screen.queryByText("Old Card")).not.toBeInTheDocument();
  });

  it("reveals archived accounts when Show archived toggle is on", async () => {
    mockUseAccounts.mockReturnValue({
      accounts: [
        makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
        makeAccount({
          id: "2",
          name: "Old Card",
          accountType: AccountType.CREDIT_CARD,
          isArchived: true,
          creditLimit: "1000",
        }),
      ],
      isLoading: false,
    } as ReturnType<typeof useAccounts>);

    renderPage();

    const user = userEvent.setup();
    const showArchived = screen.getByRole("switch", { name: /show archived/i });
    await user.click(showArchived);

    expect(screen.getByText("Old Card")).toBeInTheDocument();
  });

  it("shows Available credit chip on CREDIT_CARD rows but not on CHECKING rows", () => {
    mockUseAccounts.mockReturnValue({
      accounts: [
        makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
        makeAccount({
          id: "2",
          name: "Amex Gold",
          accountType: AccountType.CREDIT_CARD,
          creditLimit: "5000",
          currentBalance: "1000",
        }),
      ],
      isLoading: false,
    } as ReturnType<typeof useAccounts>);

    const { container } = renderPage();
    const txt = container.textContent ?? "";

    // Chip prefix appears (matches the "Available {amount}" pattern).
    expect(txt).toMatch(/Available/);
    // The chip text should be near the CC row name, not the checking row.
    const ccIdx = txt.indexOf("Amex Gold");
    const chkIdx = txt.indexOf("Chase Checking");
    const availIdx = txt.indexOf("Available");
    expect(ccIdx).toBeGreaterThan(-1);
    expect(availIdx).toBeGreaterThan(-1);
    // Chip is in the same group as the CC, so its index sits between Credit Cards
    // header and the next group header. Sanity: not adjacent to the CHECKING row.
    expect(Math.abs(availIdx - ccIdx)).toBeLessThan(Math.abs(availIdx - chkIdx));
  });
});

describe("AccountEditModal pre-fill regression (H-01)", () => {
  it("pre-fills CC fields from the account prop", () => {
    const ccAccount = makeAccount({
      id: "cc-1",
      name: "Amex Gold",
      accountType: AccountType.CREDIT_CARD,
      institution: "Chase Bank",
      openingBalance: "0",
      creditLimit: "5000",
      statementCycleDay: 15,
      currentBalance: "1000",
    });
    render(
      <MemoryRouter>
        <AccountEditModal account={ccAccount} open onClose={() => undefined} />
      </MemoryRouter>,
    );
    expect(screen.getByLabelText(/Institution/i)).toHaveValue("Chase Bank");
    // openingBalance flows through MoneyInput → numeric value; must not be empty.
    const openingInput = screen.getByLabelText(/Opening balance/i) as HTMLInputElement;
    expect(openingInput.value).not.toBe("");
    // creditLimit: "5000" → MoneyInput renders 5000
    const creditLimitInput = screen.getByLabelText(/Credit limit/i) as HTMLInputElement;
    expect(Number.parseFloat(creditLimitInput.value)).toBe(5000);
    // statementCycleDay: 15 → Select renders "15"
    const cycleDayInput = screen.getByLabelText(/Statement cycle day/i) as HTMLInputElement;
    expect(Number.parseInt(cycleDayInput.value, 10)).toBe(15);
  });

  it("pre-fills CHECKING institution + openingBalance from the account prop", () => {
    const checkingAccount = makeAccount({
      id: "chk-1",
      name: "Daily Spending",
      accountType: AccountType.CHECKING,
      institution: "Wells Fargo",
      openingBalance: "1234.56",
    });
    render(
      <MemoryRouter>
        <AccountEditModal account={checkingAccount} open onClose={() => undefined} />
      </MemoryRouter>,
    );
    expect(screen.getByLabelText(/Institution/i)).toHaveValue("Wells Fargo");
    const openingInput = screen.getByLabelText(/Opening balance/i) as HTMLInputElement;
    expect(Number.parseFloat(openingInput.value)).toBeCloseTo(1234.56, 2);
  });
});
