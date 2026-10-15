export const UNPREPARED = "Masuk GitHub belum disiapkan.";
export const DENIED = "Masuk dibatalkan.";
export const EXPIRED = "Kode masuk kedaluwarsa. Coba lagi.";
export const NOT_RUN = "Masuk tidak bisa dijalankan.";
export const DEVICE_PAGE = "https://github.com/login/device";
export const CONNECTED = "Terhubung";

const NAME = /^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9_-])?$/;
const CODE = /^[A-Za-z0-9-]{4,20}$/;

function hiddenChar(value: string): boolean {
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    if (code <= 0x1f || code === 0x7f || (code >= 0x202a && code <= 0x202e) || (code >= 0x2066 && code <= 0x2069)) {
      return true;
    }
  }
  return false;
}

function shownName(value: unknown): string {
  if (typeof value !== "string") {
    return "";
  }
  const name = value.trim();
  if (name.length === 0 || name.length > 128 || hiddenChar(name)) {
    return "";
  }
  return name;
}

function shownAvatar(value: unknown): string {
  if (typeof value !== "string" || value.length === 0 || value.length > 512 || hiddenChar(value)) {
    return "";
  }
  try {
    const url = new URL(value);
    if (
      url.protocol !== "https:" ||
      url.hostname !== "avatars.githubusercontent.com" ||
      url.port !== "" ||
      url.username !== "" ||
      url.password !== "" ||
      url.pathname.length < 2
    ) {
      return "";
    }
  } catch {
    return "";
  }
  return value;
}

export type GithubView = {
  connected: boolean;
  login: string;
  name: string;
  avatarUrl: string;
  userCode: string;
  interval: number;
};

export function deviceHint(): string {
  return `Buka ${DEVICE_PAGE} lalu masukkan kode ini.`;
}

function record(value: unknown): Record<string, unknown> | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

export function readGithub(value: unknown): GithubView | null {
  const body = record(value);
  if (!body || typeof body.connected !== "boolean") {
    return null;
  }
  const connected = body.connected;
  const login =
    connected && typeof body.login === "string" && NAME.test(body.login) ? body.login : "";
  const userCode =
    !connected && typeof body.userCode === "string" && CODE.test(body.userCode)
      ? body.userCode
      : "";
  let interval = 0;
  if (userCode) {
    const raw = body.interval;
    interval =
      typeof raw === "number" && Number.isInteger(raw) && raw >= 5 && raw <= 3600 ? raw : 5;
  }
  return {
    connected,
    login,
    name: connected ? shownName(body.name) : "",
    avatarUrl: connected ? shownAvatar(body.avatarUrl) : "",
    userCode,
    interval,
  };
}

export function githubMessage(err: unknown): string {
  if (err === UNPREPARED || err === DENIED || err === EXPIRED || err === NOT_RUN) {
    return err;
  }
  return NOT_RUN;
}

export function pollDelayMs(interval: number): number {
  if (Number.isInteger(interval) && interval >= 5 && interval <= 3600) {
    return interval * 1000;
  }
  return 5000;
}

export const OWN_REPO = "Ini repo kamu. Unduh saja.";
export const FORK_FAILED = "Fork tidak dibuat.";

const FULL =
  /^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9_-])?\/[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9_-])?$/;

export type TakeAction = "download" | "fork" | "fork-download";

export type TakeChoice = {
  id: TakeAction;
  label: string;
};

export type TakeView = {
  forked: string;
  queued: boolean;
  needFolder: boolean;
  ownRepo: boolean;
  failed: boolean;
};

export function detailTakes(connected: boolean): TakeChoice[] {
  if (connected) {
    return [
      { id: "download", label: "Hanya unduh" },
      { id: "fork", label: "Hanya fork" },
      { id: "fork-download", label: "Fork, lalu unduh" },
    ];
  }
  return [{ id: "download", label: "Unduh" }];
}

export function readTake(value: unknown): TakeView | null {
  const body = record(value);
  if (
    !body ||
    typeof body.queued !== "boolean" ||
    typeof body.needFolder !== "boolean" ||
    typeof body.ownRepo !== "boolean" ||
    typeof body.failed !== "boolean"
  ) {
    return null;
  }
  const raw = typeof body.forked === "string" ? body.forked : "";
  const repo = raw.slice(raw.indexOf("/") + 1);
  const forked = FULL.test(raw) && !repo.endsWith(".git") ? raw : "";
  return {
    forked,
    queued: body.queued,
    needFolder: body.needFolder,
    ownRepo: body.ownRepo,
    failed: body.failed || (raw !== "" && forked === ""),
  };
}

export function forkResultNote(view: TakeView): string {
  if (view.ownRepo) {
    return OWN_REPO;
  }
  if (view.failed) {
    return FORK_FAILED;
  }
  return view.forked;
}
