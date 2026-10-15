const RELEASE_URL = "https://api.github.com/repos/BadryansahBangsawan/peti/releases/latest";

export function versionLabel(release) {
  if (!release || typeof release.tag_name !== "string") return "";
  return release.tag_name.trim();
}

export function matchAsset(assets, ext, arch) {
  if (!Array.isArray(assets)) return null;
  const want = String(ext || "").toLowerCase();
  const list = assets.filter((asset) => {
    const name = String(asset && asset.name ? asset.name : "").toLowerCase();
    return want.length > 0 && name.endsWith(want) && asset.browser_download_url;
  });
  const named = (asset) => String(asset.name).toLowerCase();
  const arm = (asset) => /aarch64|arm64/.test(named(asset));
  const x64 = (asset) => /x86_64|x64|amd64/.test(named(asset));
  if (arch === "arm") {
    return (
      list.find((asset) => arm(asset)) ||
      list.find((asset) => !arm(asset) && !x64(asset)) ||
      null
    );
  }
  if (arch === "x64") {
    return (
      list.find((asset) => x64(asset)) ||
      list.find((asset) => !arm(asset) && !x64(asset)) ||
      null
    );
  }
  return list.find((asset) => x64(asset)) || list[0] || null;
}

const STEPS = {
  mac: ["Buka installer", "Pindahkan Peti", "Mulai dengan tautan"],
  win: ["Jalankan installer", "Selesaikan penyiapan", "Mulai dengan tautan"],
  linux: ["Buka berkas unduhan", "Izinkan menjalankan", "Mulai dengan tautan"],
};

function boot() {
  if (typeof document === "undefined") return;
  const page = document.querySelector("[data-release]");
  if (!page) return;

  const version = page.querySelector(".release-version");
  let assets = [];

  const paintLink = (link) => {
    const asset = matchAsset(assets, link.dataset.ext, link.dataset.arch);
    if (asset) link.href = asset.browser_download_url;
    else link.removeAttribute("href");
  };

  page.querySelectorAll(".get").forEach((link) => {
    link.addEventListener("click", (event) => {
      if (!link.getAttribute("href")) event.preventDefault();
    });
  });

  page.querySelectorAll(".card").forEach((card) => {
    card.querySelectorAll(".arch").forEach((button) => {
      button.addEventListener("click", () => {
        card.querySelectorAll(".arch").forEach((item) => {
          const on = item === button;
          item.classList.toggle("is-on", on);
          item.setAttribute("aria-pressed", on ? "true" : "false");
        });
        const link = card.querySelector(".get");
        if (link) {
          link.dataset.ext = button.dataset.ext;
          link.dataset.arch = button.dataset.arch;
          paintLink(link);
        }
        const format = card.querySelector(".fmt");
        if (format && button.dataset.fmt) format.textContent = button.dataset.fmt;
      });
    });
  });

  const stepTitles = page.querySelectorAll(".install-step h3");
  page.querySelectorAll(".tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      page.querySelectorAll(".tab").forEach((item) => {
        const on = item === tab;
        item.classList.toggle("is-on", on);
        item.setAttribute("aria-selected", on ? "true" : "false");
      });
      const lines = STEPS[tab.dataset.os] || STEPS.mac;
      stepTitles.forEach((title, index) => {
        if (lines[index]) title.textContent = lines[index];
      });
    });
  });

  fetch(RELEASE_URL, { headers: { Accept: "application/vnd.github+json" } })
    .then((response) => (response.ok ? response.json() : null))
    .then((release) => {
      const label = versionLabel(release);
      if (version && label) {
        version.textContent = label;
        version.hidden = false;
      }
      assets = release && Array.isArray(release.assets) ? release.assets : [];
      page.querySelectorAll(".get").forEach(paintLink);
    })
    .catch(() => undefined);
}

boot();
