import "@fontsource-variable/bricolage-grotesque";
import "@fontsource-variable/public-sans";
import "@fontsource/jetbrains-mono/500.css";
import "./styles.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
