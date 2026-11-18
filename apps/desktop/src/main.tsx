import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MainWindow } from "./main/MainWindow";
import "./styles.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("Root element #root not found");
}

createRoot(root).render(
  <StrictMode>
    <MainWindow />
  </StrictMode>,
);
