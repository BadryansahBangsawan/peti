export type HistoryRow = {
  id: number;
  owner: string;
  repo: string;
  gitRef: string;
  folder: string;
  finishedAt: string;
};

export function historyName(row: Pick<HistoryRow, "owner" | "repo" | "gitRef">): string {
  const name = `${row.owner}/${row.repo}`;
  const gitRef = row.gitRef.trim();
  return gitRef.length > 0 ? `${name}@${gitRef}` : name;
}

export function readHistory(value: unknown): HistoryRow[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const rows: HistoryRow[] = [];
  for (const item of value) {
    if (item === null || typeof item !== "object") {
      continue;
    }
    const row = item as Record<string, unknown>;
    if (
      typeof row.id !== "number" ||
      !Number.isSafeInteger(row.id) ||
      typeof row.owner !== "string" ||
      row.owner.trim().length === 0 ||
      typeof row.repo !== "string" ||
      row.repo.trim().length === 0 ||
      typeof row.gitRef !== "string" ||
      typeof row.folder !== "string" ||
      row.folder.trim().length === 0 ||
      typeof row.finishedAt !== "string" ||
      row.finishedAt.trim().length === 0
    ) {
      continue;
    }
    rows.push({
      id: row.id,
      owner: row.owner,
      repo: row.repo,
      gitRef: row.gitRef,
      folder: row.folder,
      finishedAt: row.finishedAt,
    });
  }
  return rows;
}

export function recentHistory(rows: readonly HistoryRow[], limit = 5): HistoryRow[] {
  const count = limit < 0 ? 0 : limit;
  return rows.slice(0, count);
}
