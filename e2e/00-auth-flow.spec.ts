import { expect, Page, test } from "@playwright/test";
import { execSync } from "node:child_process";

test.describe.configure({ mode: "serial" });

test.describe("Auth Flow: Sign-up → Verify → Login → API Key → Logout", () => {
  const BASE_URL = "http://localhost:1420";
  const TEST_EMAIL = "e2e-test-" + Date.now() + "@example.com";
  const TEST_PASSWORD = "TestPass123!";

  let page: Page;
  let sessionToken: string;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test("1. Register new account", async () => {
    await page.goto(BASE_URL + "/register");
    await page.waitForLoadState("networkidle");

    await page.locator('input[id="email"]').fill(TEST_EMAIL);
    await page.locator('input[id="password"]').fill(TEST_PASSWORD);
    await page.locator('input[id="confirmPassword"]').fill(TEST_PASSWORD);

    await page.getByRole("button", { name: "Create Account" }).click();

    await expect(page).toHaveURL(/\/register\/pending/, { timeout: 15000 });
    await expect(page.getByText("Check Your Email")).toBeVisible({ timeout: 10000 });
  });

  test("2. Verify email", async () => {
    const verificationToken = extractTokenFromLogs("Verification URL:");
    expect(verificationToken).toBeTruthy();

    await page.goto(BASE_URL + "/verify-email?token=" + verificationToken);
    await page.waitForLoadState("networkidle");

    await page.getByRole("button", { name: "Verify Email" }).click();
    await expect(page.getByText("Email Verified")).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(3500);
  });

  test("3. Login and capture session", async () => {
    await page.goto(BASE_URL + "/login");
    await page.waitForLoadState("networkidle");

    await page.locator('input[id="email"]').fill(TEST_EMAIL);
    await page.locator('input[id="password"]').fill(TEST_PASSWORD);

    // Capture the session token from the login response
    const [loginResp] = await Promise.all([
      page.waitForResponse((r) => r.url().includes("/api/v1/auth/login")),
      page.getByRole("button", { name: "Sign In" }).click(),
    ]);
    expect(loginResp.status()).toBe(200);

    // Extract token from Set-Cookie header or from cookies
    await page.waitForTimeout(1000);
    const cookies = await page.context().cookies();
    const session = cookies.find((c) => c.name === "wf_session");
    expect(session).toBeTruthy();
    sessionToken = session!.value;

    await expect(page).toHaveURL(/\/(onboarding|\/$)/, { timeout: 15000 });
  });

  test("4. Create API Key", async () => {
    const res = await page.evaluate(async (token) => {
      const r = await fetch("/api/v1/auth/api-keys", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ name: "E2E Test Key" }),
      });
      return { status: r.status, body: await r.json() };
    }, sessionToken);
    expect(res.status).toBe(201);
    expect(res.body.apiKey).toMatch(/wfk_live_/);
    expect(res.body.name).toBe("E2E Test Key");
  });

  test("5. List API Keys", async () => {
    const res = await page.evaluate(async (token) => {
      const r = await fetch("/api/v1/auth/api-keys", {
        headers: { Authorization: "Bearer " + token },
      });
      return { status: r.status, body: await r.json() };
    }, sessionToken);
    expect(res.status).toBe(200);
    expect(res.body).toHaveLength(1);
    expect(res.body[0].name).toBe("E2E Test Key");
  });

  test("6. Forgot password", async () => {
    await page.goto(BASE_URL + "/forgot-password");
    await page.waitForLoadState("networkidle");

    await page.locator('input[id="email"]').fill(TEST_EMAIL);
    await page.getByRole("button", { name: "Send Reset Link" }).click();

    await expect(page.getByText("Check Your Email")).toBeVisible({ timeout: 10000 });
  });

  test("7. Reset password", async () => {
    const resetToken = extractTokenFromLogs("Password reset URL:");
    expect(resetToken).toBeTruthy();

    const NEW_PASSWORD = "NewPass456!";

    await page.goto(BASE_URL + "/reset-password?token=" + resetToken);
    await page.waitForLoadState("networkidle");

    await page.locator('input[id="password"]').fill(NEW_PASSWORD);
    await page.locator('input[id="confirmPassword"]').fill(NEW_PASSWORD);

    await page.getByRole("button", { name: "Reset Password" }).click();

    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Login with new password and capture new session
    await page.locator('input[id="email"]').fill(TEST_EMAIL);
    await page.locator('input[id="password"]').fill(NEW_PASSWORD);

    const [loginResp] = await Promise.all([
      page.waitForResponse((r) => r.url().includes("/api/v1/auth/login")),
      page.getByRole("button", { name: "Sign In" }).click(),
    ]);
    expect(loginResp.status()).toBe(200);

    await page.waitForTimeout(1000);
    const cookies = await page.context().cookies();
    const session = cookies.find((c) => c.name === "wf_session");
    if (session) sessionToken = session.value;
  });

  test("8. Revoke API Key", async () => {
    const listRes = await page.evaluate(async (token) => {
      const r = await fetch("/api/v1/auth/api-keys", {
        headers: { Authorization: "Bearer " + token },
      });
      return r.json();
    }, sessionToken);

    expect(listRes.length).toBeGreaterThanOrEqual(1);
    const keyId = listRes[0].id;

    const deleteRes = await page.evaluate(
      async ([token, id]) => {
        const r = await fetch("/api/v1/auth/api-keys", {
          method: "DELETE",
          headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
          body: JSON.stringify({ id }),
        });
        return r.status;
      },
      [sessionToken, keyId] as [string, string],
    );
    expect(deleteRes).toBe(204);
  });

  test("9. Logout", async () => {
    await page.evaluate(async () => {
      await fetch("/api/v1/auth/logout", { method: "POST", credentials: "same-origin" });
    });
    await page.goto(BASE_URL + "/login");
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });
  });

  test("10. Cannot access protected pages after logout", async () => {
    const res = await page.evaluate(async () => {
      const r = await fetch("/api/v1/auth/me", { credentials: "same-origin" });
      return r.status;
    });
    expect(res).toBe(401);
  });

  function extractTokenFromLogs(marker: string): string {
    try {
      const log = execSync(
        "grep -o '" + marker + ".*' /tmp/whaleit-dev2.log 2>/dev/null | tail -1",
        { encoding: "utf8", timeout: 5000 },
      );
      const match = log.match(/token=([a-zA-Z0-9_-]+)/);
      return match ? match[1] : "";
    } catch {
      return "";
    }
  }
});
