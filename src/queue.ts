export type QueueRow = {
  id: number;
  owner: string;
  repo: string;
  gitRef: string;
  status: string;
  error: string;
  bytes: number;
  total: number | null;
};

const statuses = new Set(["menunggu", "berjalan", "dijeda", "selesai", "gagal", "dibatalkan"]);

export function readQueue(value: unknown): QueueRow[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const rows: QueueRow[] = [];
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
      typeof row.status !== "string" ||
      !statuses.has(row.status)
    ) {
      continue;
    }
    rows.push({
      id: row.id,
      owner: row.owner,
      repo: row.repo,
      gitRef: row.gitRef,
      status: row.status,
      error: typeof row.error === "string" ? row.error.trim() : "",
      bytes: readBytes(row.bytes),
      total: readTotal(row.total),
    });
  }
  return rows;
}

export function rowMessage(row: Pick<QueueRow, "error">): string {
  return row.error.trim();
}

function readBytes(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    return 0;
  }
  return Math.floor(value);
}

function readTotal(value: unknown): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return null;
  }
  return value;
}

export function percentOf(bytes: number, total: number | null): number | null {
  if (total === null || !Number.isFinite(total) || total <= 0) {
    return null;
  }
  if (!Number.isFinite(bytes) || bytes < 0) {
    return null;
  }
  return Math.min(100, Math.floor((bytes / total) * 100));
}

const statusLabels: Record<string, string> = {
  berjalan: "Mengunduh",
  dijeda: "Dijeda",
  menunggu: "Menunggu",
  gagal: "Gagal",
  selesai: "Selesai",
  dibatalkan: "Dibatalkan",
};

export function statusLabel(status: string, labels: Record<string, string> = statusLabels): string {
  return labels[status] ?? statusLabels[status] ?? status;
}

const sizeUnits = ["B", "KB", "MB", "GB", "TB"];

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return "0 B";
  }
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < sizeUnits.length - 1) {
    value /= 1000;
    unit += 1;
  }
  if (unit === 0) {
    return `${Math.floor(value)} B`;
  }
  let rounded = Math.round(value * 10) / 10;
  if (rounded >= 1000 && unit < sizeUnits.length - 1) {
    rounded /= 1000;
    unit += 1;
  }
  return `${rounded.toFixed(1).replace(".", ",")} ${sizeUnits[unit]}`;
}

export function formatRate(bytesPerSecond: number): string {
  return `${formatSize(bytesPerSecond)}/s`;
}

export type SpeedSample = { bytes: number; at: number; speed: number | null };

export function nextSpeed(
  status: string,
  bytes: number,
  now: number,
  prev: SpeedSample | undefined,
): { sample: SpeedSample | null; speed: number | null } {
  if (status !== "berjalan") {
    return { sample: status === "dijeda" ? (prev ?? null) : null, speed: null };
  }
  if (!prev) {
    return { sample: { bytes, at: now, speed: null }, speed: null };
  }
  if (now <= prev.at || bytes <= prev.bytes) {
    return { sample: prev, speed: prev.speed };
  }
  const rate = (bytes - prev.bytes) / ((now - prev.at) / 1000);
  const speed = rate >= 1 ? rate : prev.speed;
  return { sample: { bytes, at: now, speed }, speed };
}

const transferGap = "  ·  ";

export type TransferPhrases = {
  waiting: string;
  downloaded: string;
  remaining: string;
  totalUnknown: string;
  saved: string;
};

const transferPhrases: TransferPhrases = {
  waiting: "Menunggu slot unduhan tersedia",
  downloaded: "{size} terunduh",
  remaining: "sisa {seconds} detik",
  totalUnknown: "ukuran total belum tersedia",
  saved: "progres tersimpan",
};

function transferFill(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{([a-z]+)\}/g, (token, key: string) =>
    Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : token,
  );
}

export function transferLine(
  row: {
    status: string;
    bytes: number;
    total: number | null;
    speed: number | null;
    error: string;
  },
  phrases: TransferPhrases = transferPhrases,
): string {
  const error = row.error.trim();
  if (row.status === "menunggu") {
    return error || phrases.waiting;
  }
  if (row.status !== "berjalan" && row.status !== "dijeda") {
    return error;
  }
  const percent = percentOf(row.bytes, row.total);
  const speed = row.status === "berjalan" && row.speed !== null && row.speed >= 1 ? row.speed : null;
  const parts: string[] = [];
  if (percent !== null && row.total !== null) {
    parts.push(`${percent}%`);
    parts.push(`${formatSize(row.bytes)} / ${formatSize(row.total)}`);
  } else {
    parts.push(transferFill(phrases.downloaded, { size: formatSize(row.bytes) }));
  }
  if (speed !== null) {
    parts.push(formatRate(speed));
  }
  if (percent !== null && speed !== null && row.total !== null && row.total > row.bytes) {
    const seconds = Math.ceil((row.total - row.bytes) / speed);
    parts.push(transferFill(phrases.remaining, { seconds }));
  } else if (percent === null && row.status === "berjalan") {
    parts.push(phrases.totalUnknown);
  }
  if (row.status === "dijeda") {
    parts.push(phrases.saved);
  }
  const line = parts.join(transferGap);
  return error.length > 0 ? `${error}${transferGap}${line}` : line;
}
