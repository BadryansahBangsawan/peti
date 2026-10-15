export const LIMIT_MESSAGE = "Batas GitHub tercapai, coba lagi nanti";
export const FAILED_MESSAGE = "Pencarian tidak bisa dijalankan.";
export const EMPTY_DESCRIPTION = "Tidak ada deskripsi";
export const SEARCH_WAIT_MS = 300;

export type DiscoverCard = {
  owner: string;
  repo: string;
  description: string;
  language: string;
  stars: number;
  forks: number;
};

export type DiscoverRead = {
  cards: DiscoverCard[];
  limited: boolean;
  failed: boolean;
};

export type DiscoverNotice = "none" | "limited" | "failed" | "empty";

export function repoTitle(card: Pick<DiscoverCard, "owner" | "repo">): string {
  return `${card.owner}/${card.repo}`;
}

export function descriptionOf(card: Pick<DiscoverCard, "description">): string {
  const text = card.description.trim();
  return text.length > 0 ? text : EMPTY_DESCRIPTION;
}

export function metaLine(card: Pick<DiscoverCard, "language" | "stars" | "forks">): string {
  const counts = `${card.stars} star · ${card.forks} fork`;
  const language = card.language.trim();
  return language.length > 0 ? `${language} · ${counts}` : counts;
}

export function periodQuery(value: string): "day" | "week" | "month" {
  return value === "day" || value === "month" ? value : "week";
}

const REPO_NAME = /^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9_-])?$/;

export function cardKey(card: Pick<DiscoverCard, "owner" | "repo">): string {
  return `${card.owner}/${card.repo}`;
}

export function githubHref(owner: string, repo: string): string | null {
  if (!REPO_NAME.test(owner) || !REPO_NAME.test(repo)) {
    return null;
  }
  return `https://github.com/${owner}/${repo}`;
}

export function selectionLabel(count: number): string {
  return `${count} dipilih`;
}

export type QueueDiscover = {
  queued: number;
  needFolder: boolean;
};

export function readQueueDiscover(value: unknown): QueueDiscover | null {
  if (value === null || typeof value !== "object") {
    return null;
  }
  const body = value as Record<string, unknown>;
  if (
    typeof body.queued !== "number" ||
    !Number.isSafeInteger(body.queued) ||
    body.queued < 0 ||
    typeof body.needFolder !== "boolean"
  ) {
    return null;
  }
  return { queued: body.queued, needFolder: body.needFolder };
}

function count(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    return undefined;
  }
  return value;
}

export function readDiscover(value: unknown): DiscoverRead {
  if (value === null || typeof value !== "object") {
    return { cards: [], limited: false, failed: true };
  }
  const body = value as Record<string, unknown>;
  const cards: DiscoverCard[] = [];
  if (Array.isArray(body.repos)) {
    for (const item of body.repos) {
      if (item === null || typeof item !== "object") {
        continue;
      }
      const row = item as Record<string, unknown>;
      const stars = count(row.stars);
      const forks = count(row.forks);
      if (
        typeof row.owner !== "string" ||
        row.owner.trim().length === 0 ||
        typeof row.repo !== "string" ||
        row.repo.trim().length === 0 ||
        stars === undefined ||
        forks === undefined
      ) {
        continue;
      }
      cards.push({
        owner: row.owner,
        repo: row.repo,
        description: typeof row.description === "string" ? row.description : "",
        language: typeof row.language === "string" ? row.language : "",
        stars,
        forks,
      });
    }
  }
  if (body.limited === true) {
    return { cards, limited: true, failed: false };
  }
  if (body.failed === true) {
    return { cards: [], limited: false, failed: true };
  }
  return { cards, limited: false, failed: false };
}

export function applySearchResult(
  current: readonly DiscoverCard[],
  ticket: number,
  latest: number,
  incoming: DiscoverRead,
): { cards: DiscoverCard[]; notice: DiscoverNotice } {
  if (ticket !== latest) {
    return { cards: [...current], notice: "none" };
  }
  if (incoming.limited) {
    return { cards: [...current], notice: "limited" };
  }
  if (incoming.failed) {
    return { cards: [...current], notice: "failed" };
  }
  if (incoming.cards.length === 0) {
    return { cards: [], notice: "empty" };
  }
  return { cards: incoming.cards.map((card) => ({ ...card })), notice: "none" };
}
