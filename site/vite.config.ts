import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { defineConfig, type Plugin } from "vite";

const repo = fileURLToPath(new URL("../", import.meta.url));
const design = `${repo}design/`;

/**
 * The page points at the app's icons with `../crates/…` URLs. The build follows them, but
 * a browser cannot ask the dev server for a path above its root, so in dev those requests
 * are sent to the files themselves.
 */
function serveAppIcons(): Plugin {
  const icons = "/crates/nibble-app/icons/";
  return {
    name: "nibble-serve-app-icons",
    configureServer(server) {
      server.middlewares.use((request, _response, next) => {
        if (request.url?.startsWith(icons)) request.url = `/@fs/${repo}${request.url.slice(1)}`;
        next();
      });
    },
  };
}

/**
 * Replaces each `<!-- design:nibble-sleepy.svg -->` comment in the page with that drawing
 * from `design/`, so the built page is plain HTML whose mascot parts CSS can animate, with
 * no script for the drawings. Ids get a per-use suffix so a drawing can appear twice, and black
 * becomes `currentColor` so the menu bar icons follow the text colour.
 */
function inlineDesignSvg(): Plugin {
  return {
    name: "nibble-inline-design-svg",
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        let uses = 0;
        return html.replace(/<!--\s*design:([\w/-]+\.svg)\s*-->/g, (_match, file: string) => {
          uses += 1;
          const suffix = `-${String(uses)}`;
          return readFileSync(design + file, "utf8")
            .replace("<svg ", '<svg aria-hidden="true" focusable="false" ')
            .replace(/\bid="([^"]+)"/g, `id="$1${suffix}"`)
            .replace(/url\(#([^)]+)\)/g, `url(#$1${suffix})`)
            .replaceAll('"#000"', '"currentColor"');
        });
      },
    },
  };
}

type Os = "macos" | "windows" | "linux";

/** The systems the page lists, in its order. */
const systems: readonly { os: Os; name: string }[] = [
  { os: "macos", name: "macOS" },
  { os: "windows", name: "Windows" },
  { os: "linux", name: "Linux" },
];

interface Download {
  os: Os;
  arch: string;
  name: string;
  url: string;
  size: number;
}

interface Release {
  version: string;
  downloads: Download[];
}

const releasesPage = "https://github.com/tonytangdev/nibble/releases/latest";

/**
 * The installers a visitor downloads. The updater's files on a release (`latest.json`, the
 * `.tar.gz` bundles and their `.sig`) match none of these, so they are left out.
 */
function osOf(file: string): Os | null {
  if (/\.dmg$/i.test(file)) return "macos";
  if (/\.(exe|msi)$/i.test(file)) return "windows";
  if (/\.(AppImage|deb|rpm)$/i.test(file)) return "linux";
  return null;
}

function archOf(file: string): string {
  if (/aarch64|arm64/i.test(file)) return "arm64";
  if (/x86_64|x64|amd64/i.test(file)) return "x64";
  return "";
}

/** The chip a file is for, in the words its system uses. */
function chipName(download: Download): string {
  if (download.os === "macos") {
    if (download.arch === "arm64") return "Apple silicon";
    if (download.arch === "x64") return "Intel";
  }
  return download.arch;
}

/** The latest published release on GitHub. Drafts and pre-releases are never `latest`. */
async function latestRelease(): Promise<Release> {
  const headers: Record<string, string> = {
    Accept: "application/vnd.github+json",
    "User-Agent": "nibble-site-build",
  };
  const token = process.env.GITHUB_TOKEN;
  if (token) headers.Authorization = `Bearer ${token}`;
  const response = await fetch("https://api.github.com/repos/tonytangdev/nibble/releases/latest", {
    headers,
  });
  if (!response.ok) throw new Error(`GitHub answered ${String(response.status)}`);
  const body = (await response.json()) as {
    tag_name: string;
    assets: { name: string; browser_download_url: string; size: number }[];
  };
  const downloads: Download[] = [];
  for (const asset of body.assets) {
    const os = osOf(asset.name);
    if (os === null) continue;
    downloads.push({
      os,
      arch: archOf(asset.name),
      name: asset.name,
      url: asset.browser_download_url,
      size: asset.size,
    });
  }
  return { version: body.tag_name.replace(/^v/, ""), downloads };
}

function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function megabytes(bytes: number): string {
  return `${(bytes / 1_000_000).toFixed(1)} MB`;
}

function downloadList(release: Release | null): string {
  if (release === null) {
    return `<p class="fine">See the files on <a href="${releasesPage}">GitHub Releases</a>.</p>`;
  }
  const rows = systems.map(({ os, name }) => {
    const files = release.downloads.filter((download) => download.os === os);
    const body =
      files.length === 0
        ? '<p class="none">No build yet.</p>'
        : `<ul>${files
            .map((download) => {
              const details = [chipName(download), megabytes(download.size)].filter(Boolean);
              return (
                `<li><a href="${escapeHtml(download.url)}" data-os="${os}"` +
                ` data-arch="${escapeHtml(download.arch)}">${escapeHtml(download.name)}</a>` +
                ` <span>${escapeHtml(details.join(" · "))}</span></li>`
              );
            })
            .join("")}</ul>`;
    return `<div><dt>${name}</dt><dd>${body}</dd></div>`;
  });
  return (
    `<p class="files-title">nibble ${escapeHtml(release.version)}</p>` + `<dl>${rows.join("")}</dl>`
  );
}

/**
 * Lists the files of nibble's latest release, by system, where the page has
 * `<!-- release:downloads -->`, and points each `href="release:default"` at the first macOS
 * file: the right one for a visitor without scripts, as macOS comes first. The page's script
 * repoints it for other systems. Pushes to main and each published release rebuild the page.
 * Without GitHub, a local build links to the releases page instead; in CI it fails, so the
 * live page keeps its list.
 */
function releaseDownloads(): Plugin {
  let release: Promise<Release | null> | undefined;
  const load = () =>
    (release ??= latestRelease().catch((error: unknown) => {
      if (process.env.CI) throw error;
      console.warn(`nibble: could not read the latest release (${String(error)}).`);
      return null;
    }));
  return {
    name: "nibble-release-downloads",
    transformIndexHtml: {
      order: "pre",
      async handler(html) {
        if (!html.includes("release:")) return html;
        const latest = await load();
        const first = latest?.downloads.find((download) => download.os === "macos");
        return html
          .replace(/<!--\s*release:downloads\s*-->/g, downloadList(latest))
          .replaceAll('href="release:default"', `href="${escapeHtml(first?.url ?? "#download")}"`);
      },
    },
  };
}

// A static page. `base: "./"` keeps every URL relative, so the build works at a domain
// root and under a GitHub Pages project path alike.
export default defineConfig({
  base: "./",
  plugins: [inlineDesignSvg(), releaseDownloads(), serveAppIcons()],
  clearScreen: false,
  // The popover's dev server owns 5173.
  server: { port: 5174, strictPort: true, fs: { allow: [".."] } },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rolldownOptions: {
      input: {
        index: fileURLToPath(new URL("index.html", import.meta.url)),
        // Served by the host for any missing path.
        notFound: fileURLToPath(new URL("404.html", import.meta.url)),
      },
    },
  },
});
