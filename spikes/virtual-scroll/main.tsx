import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { VirtualScrollSpike } from "./app";
import "./style.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

createRoot(root).render(
  <StrictMode>
    <VirtualScrollSpike />
  </StrictMode>,
);
