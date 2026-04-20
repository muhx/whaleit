// Goals domain handlers

export function handleCreateGoal(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { goal } = payload as { goal: Record<string, unknown> };
  return { url, body: JSON.stringify(goal) };
}

export function handleUpdateGoal(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { goal } = payload as { goal: Record<string, unknown> };
  return { url, body: JSON.stringify(goal) };
}

export function handleDeleteGoal(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { goalId } = payload as { goalId: string };
  return { url: `${url}/${encodeURIComponent(goalId)}`, body: undefined };
}

export function handleUpdateGoalAllocations(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { allocations } = payload as { allocations: Record<string, unknown> };
  return { url, body: JSON.stringify(allocations) };
}
