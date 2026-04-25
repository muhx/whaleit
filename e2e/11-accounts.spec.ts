import { expect, Page, test } from "@playwright/test";
import { BASE_URL, loginIfNeeded } from "./helpers";

test.describe.configure({ mode: "serial" });

// Unique names per run so the spec can be re-run against an unprepped DB
// without colliding with prior fixtures.
const checkingName = `E2E Checking ${Date.now()}`;
const ccName = `E2E CreditCard ${Date.now()}`;

/**
 * Phase 3 — Bank Accounts & Credit Cards
 *
 * Drives the full user flow for ACCT-01..07 against the live web app:
 *   1. Login + navigate to /settings/accounts
 *   2. Create a CHECKING account (Institution + Opening balance)
 *   3. Create a CREDIT_CARD with required CC fields (credit_limit, cycle_day)
 *   4. Open Update balance modal on the CC, persist new balance
 *   5. Archive the CHECKING account; confirm hidden by default
 *   6. Toggle "Show archived" Switch; confirm archived row reveals
 *
 * The unified accounts list is hosted under /settings/accounts (D-15 amendment),
 * NOT a top-level /accounts route.
 */
test.describe("Phase 3 — Bank Accounts & Credit Cards", () => {
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test("1. Setup: login + open /settings/accounts", async () => {
    test.setTimeout(180_000);
    await loginIfNeeded(page);
    await page.goto(`${BASE_URL}/settings/accounts`, { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: "Accounts", exact: true })).toBeVisible({
      timeout: 30_000,
    });
  });

  test("2. Create CHECKING account", async () => {
    // Open the create-account dialog
    const addButton = page.getByRole("button", { name: /Add account/i }).first();
    await expect(addButton).toBeVisible();
    await addButton.click();

    await expect(page.getByRole("heading", { name: /Add Account/i })).toBeVisible();

    // Account Name
    await page.getByLabel("Account Name").fill(checkingName);

    // Account Type — ResponsiveSelect renders a button with the FormLabel "Account Type"
    const typeTrigger = page.getByLabel("Account Type");
    await typeTrigger.click();
    await page.getByRole("option", { name: "Checking" }).click();

    // Institution (renders only when type is CHECKING/SAVINGS/CREDIT_CARD/LOAN)
    await expect(page.getByLabel("Institution")).toBeVisible();
    await page.getByLabel("Institution").fill("Test Bank");

    // Opening balance (MoneyInput) — fill with 0
    await page.getByLabel("Opening balance").fill("0");

    // Currency stays at default; pick USD explicitly to keep totals stable
    const currencyTrigger = page.getByLabel("Currency");
    const currentCurrencyText = await currencyTrigger.textContent();
    if (!currentCurrencyText?.includes("USD")) {
      await currencyTrigger.click();
      await page.waitForSelector('[role="listbox"], [role="option"]', {
        state: "visible",
        timeout: 5_000,
      });
      const searchInput = page.getByPlaceholder("Search currency...");
      if (await searchInput.isVisible()) {
        await searchInput.fill("USD");
        await page.waitForTimeout(200);
      }
      await page.getByRole("option", { name: /USD/ }).first().click();
      await page.waitForTimeout(200);
    }

    // Tracking Mode = Transactions (required to enable submit)
    const transactionsRadio = page.getByRole("radio", { name: /Transactions/i });
    await expect(transactionsRadio).toBeVisible();
    await transactionsRadio.click();

    // Submit
    const submitButton = page.getByRole("button", { name: /^Add Account$/i }).last();
    await submitButton.click();

    await expect(page.getByRole("heading", { name: /Add Account/i })).not.toBeVisible({
      timeout: 10_000,
    });

    // Row is rendered as a link to /accounts/:id
    await expect(page.getByRole("link", { name: checkingName })).toBeVisible({ timeout: 10_000 });
  });

  test("3. Create CREDIT_CARD account with required CC fields", async () => {
    const addButton = page.getByRole("button", { name: /Add account/i }).first();
    await addButton.click();

    await expect(page.getByRole("heading", { name: /Add Account/i })).toBeVisible();

    await page.getByLabel("Account Name").fill(ccName);

    const typeTrigger = page.getByLabel("Account Type");
    await typeTrigger.click();
    await page.getByRole("option", { name: "Credit Card" }).click();

    await page.getByLabel("Institution").fill("Test Bank");
    await page.getByLabel("Opening balance").fill("0");

    // CC-only fields render once CREDIT_CARD is selected
    const creditLimit = page.getByLabel("Credit limit");
    await expect(creditLimit).toBeVisible();
    await creditLimit.fill("5000");

    // Statement cycle day — shadcn Select with SelectTrigger
    const cycleDayTrigger = page.getByLabel("Statement cycle day");
    await cycleDayTrigger.click();
    await page.getByRole("option", { name: "15", exact: true }).click();

    // Currency — keep USD (default may already be USD; same flow as test 2)
    const currencyTrigger = page.getByLabel("Currency");
    const currentCurrencyText = await currencyTrigger.textContent();
    if (!currentCurrencyText?.includes("USD")) {
      await currencyTrigger.click();
      await page.waitForSelector('[role="listbox"], [role="option"]', {
        state: "visible",
        timeout: 5_000,
      });
      const searchInput = page.getByPlaceholder("Search currency...");
      if (await searchInput.isVisible()) {
        await searchInput.fill("USD");
        await page.waitForTimeout(200);
      }
      await page.getByRole("option", { name: /USD/ }).first().click();
      await page.waitForTimeout(200);
    }

    // Tracking Mode = Transactions
    const transactionsRadio = page.getByRole("radio", { name: /Transactions/i });
    await transactionsRadio.click();

    const submitButton = page.getByRole("button", { name: /^Add Account$/i }).last();
    await submitButton.click();

    await expect(page.getByRole("heading", { name: /Add Account/i })).not.toBeVisible({
      timeout: 10_000,
    });

    await expect(page.getByRole("link", { name: ccName })).toBeVisible({ timeout: 10_000 });

    // CC row should expose an "Available credit" chip (creditLimit - currentBalance = 5000 - 0)
    await expect(page.getByText(/Available/i).first()).toBeVisible();
  });

  test("4. Update credit-card balance via 'Update balance' modal", async () => {
    // Navigate into the CC detail page
    await page.getByRole("link", { name: ccName }).click();

    // The CC detail page renders the "Update balance" button on the Credit overview card
    const updateBtn = page.getByRole("button", { name: "Update balance" }).first();
    await expect(updateBtn).toBeVisible({ timeout: 10_000 });
    await updateBtn.click();

    // Modal opens with the current balance prefilled. The MoneyInput is the first
    // inputMode="decimal" control inside the dialog.
    const dialog = page.getByRole("dialog").filter({ hasText: "Update balance" });
    await expect(dialog).toBeVisible();
    const balanceInput = dialog.locator('input[inputmode="decimal"]').first();
    await expect(balanceInput).toBeVisible();
    await balanceInput.fill("750");

    // Save
    await dialog.getByRole("button", { name: "Save balance" }).click();

    // Success toast appears and the dialog closes
    await expect(page.getByText("Balance updated just now")).toBeVisible({ timeout: 10_000 });
    await expect(dialog).not.toBeVisible({ timeout: 5_000 });
  });

  test("5. Archive CHECKING account, confirm hidden by default on /settings/accounts", async () => {
    await page.goto(`${BASE_URL}/settings/accounts`, { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: "Accounts", exact: true })).toBeVisible();

    // The CHECKING row is rendered as a link inside an AccountItem; the row's
    // overflow menu trigger is a sibling DropdownMenuTrigger with aria-label "Open".
    // Strategy: locate the row container by its link text, then find the menu trigger.
    const checkingLink = page.getByRole("link", { name: checkingName });
    await expect(checkingLink).toBeVisible();

    // Walk up from the link to the row container (`.flex.items-center.justify-between.p-4`),
    // then find the DropdownMenu trigger (button with sr-only "Open" text).
    // Since locating by exact CSS class is brittle, we get the closest button-with-Open ancestor.
    const checkingRow = checkingLink.locator(
      'xpath=ancestor::*[.//button[normalize-space()="Open"]][1]',
    );
    const menuTrigger = checkingRow.getByRole("button", { name: "Open" });
    await expect(menuTrigger).toBeVisible();
    await menuTrigger.click();

    // Click the "Archive" menuitem
    await page.getByRole("menuitem", { name: "Archive" }).click();

    // Confirmation AlertDialog: click the inner Archive button (not the menuitem)
    const archiveDialog = page.getByRole("alertdialog");
    await expect(archiveDialog).toBeVisible();
    await archiveDialog.getByRole("button", { name: "Archive", exact: true }).click();
    await expect(archiveDialog).not.toBeVisible({ timeout: 5_000 });

    // Default view (filter="all", showArchived=false) should hide archived rows
    await page.waitForTimeout(500);
    await expect(page.getByRole("link", { name: checkingName })).toHaveCount(0);
  });

  test("6. Show archived toggle reveals archived row", async () => {
    // Still on /settings/accounts. Flip the Show archived Switch.
    const showArchivedSwitch = page.getByRole("switch", { name: /Show archived/i });
    await expect(showArchivedSwitch).toBeVisible();
    await showArchivedSwitch.click();

    // The archived CHECKING row should now be visible again, badged "Archived".
    await expect(page.getByRole("link", { name: checkingName })).toBeVisible({ timeout: 5_000 });
    await expect(page.getByText("Archived").first()).toBeVisible();
  });
});
