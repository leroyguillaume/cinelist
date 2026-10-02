import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "react-router";
import { initI18n } from "./i18n";
import { createQueryClient } from "./lib/queries";
import { router } from "./router";
import "./styles/app.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

void initI18n().then(() => {
  createRoot(root).render(
    <StrictMode>
      <QueryClientProvider client={createQueryClient()}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>,
  );
});
