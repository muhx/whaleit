import { readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const ENV_PATH = join(dirname(fileURLToPath(import.meta.url)), "..", ".env.web");

const pad = (value) => String(value).padStart(2, "0");

const getTimestamp = () => {
  const now = new Date();
  return `${now.getUTCFullYear()}${pad(now.getUTCMonth() + 1)}${pad(now.getUTCDate())}T${pad(
    now.getUTCHours(),
  )}${pad(now.getUTCMinutes())}${pad(now.getUTCSeconds())}Z`;
};

export const prepE2eEnv = async () => {
  const timestamp = getTimestamp();
  const dbName = `whaleit_e2e_${timestamp}`;

  const content = await readFile(ENV_PATH, "utf8");

  let updated = content;
  if (content.includes("DATABASE_URL=")) {
    updated = content.replace(
      /^DATABASE_URL=.*$/m,
      `DATABASE_URL=postgres://whaleit:password@localhost:5432/${dbName}`,
    );
  } else {
    updated = `DATABASE_URL=postgres://whaleit:password@localhost:5432/${dbName}\n` + content;
  }

  await writeFile(ENV_PATH, updated);
  console.log(`Updated .env.web to use PostgreSQL database: ${dbName}`);

  try {
    execSync(
      `docker exec whaleit-postgres psql -U whaleit -d postgres -c "DROP DATABASE IF EXISTS \\"${dbName}\\";" -c "CREATE DATABASE \\"${dbName}\\";"`,
      { stdio: "pipe" },
    );
    console.log(`Created fresh PostgreSQL database: ${dbName}`);
  } catch {
    console.warn(
      `Could not create database automatically. Create it manually: CREATE DATABASE "${dbName}"`,
    );
  }
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await prepE2eEnv();
}
