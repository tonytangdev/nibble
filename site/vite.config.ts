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
 * no script at runtime. Ids get a per-use suffix so a drawing can appear twice, and black
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

// A static page. `base: "./"` keeps every URL relative, so the build works at a domain
// root and under a GitHub Pages project path alike.
export default defineConfig({
  base: "./",
  plugins: [inlineDesignSvg(), serveAppIcons()],
  clearScreen: false,
  // The popover's dev server owns 5173.
  server: { port: 5174, strictPort: true, fs: { allow: [".."] } },
  build: { outDir: "dist", emptyOutDir: true },
});
