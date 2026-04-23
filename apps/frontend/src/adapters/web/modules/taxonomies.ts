// Taxonomies domain handlers

export function handleGetTaxonomy(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}

export function handleCreateTaxonomy(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { taxonomy } = payload as { taxonomy: Record<string, unknown> };
  return { url, body: JSON.stringify(taxonomy) };
}

export function handleUpdateTaxonomy(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { taxonomy } = payload as { taxonomy: Record<string, unknown> };
  return { url, body: JSON.stringify(taxonomy) };
}

export function handleDeleteTaxonomy(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}

export function handleCreateCategory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { category } = payload as { category: Record<string, unknown> };
  return { url, body: JSON.stringify(category) };
}

export function handleUpdateCategory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { category } = payload as { category: Record<string, unknown> };
  return { url, body: JSON.stringify(category) };
}

export function handleDeleteCategory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { taxonomyId, categoryId } = payload as { taxonomyId: string; categoryId: string };
  return {
    url: `${url}/${encodeURIComponent(taxonomyId)}/categories/${encodeURIComponent(categoryId)}`,
    body: undefined,
  };
}

export function handleMoveCategory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { taxonomyId, categoryId, newParentId, position } = payload as {
    taxonomyId: string;
    categoryId: string;
    newParentId: string | null;
    position: number;
  };
  return { url, body: JSON.stringify({ taxonomyId, categoryId, newParentId, position }) };
}

export function handleImportTaxonomyJson(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { jsonStr } = payload as { jsonStr: string };
  return { url, body: JSON.stringify({ jsonStr }) };
}

export function handleExportTaxonomyJson(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}/export`, body: undefined };
}

export function handleGetAssetTaxonomyAssignments(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetId } = payload as { assetId: string };
  return { url: `${url}/${encodeURIComponent(assetId)}`, body: undefined };
}

export function handleAssignAssetToCategory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assignment } = payload as { assignment: Record<string, unknown> };
  return { url, body: JSON.stringify(assignment) };
}

export function handleRemoveAssetTaxonomyAssignment(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}
