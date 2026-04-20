// Utilities domain handlers (backup, restore, app info)

export function handleBackupDatabaseToPath(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { backupDir } = payload as { backupDir: string };
  return { url, body: JSON.stringify({ backupDir }) };
}

export function handleRestoreDatabase(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { backupFilePath } = payload as { backupFilePath: string };
  return { url, body: JSON.stringify({ backupFilePath }) };
}
