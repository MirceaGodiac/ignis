import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App } from "./App";
import "./styles.css";

// After a deploy, a tab opened on the previous build asks for lazy chunks (PDF viewer) that no
// longer exist; the SPA fallback answers with HTML and the import fails, which blanked the page.
// Reload once to pick up the current build; a second failure within a minute is left to show.
window.addEventListener("vite:preloadError", (event) => {
  let last = 0;
  try {
    last = Number(sessionStorage.getItem("ignis:chunk-reload") ?? 0);
  } catch {
    /* storage unavailable: reload anyway, once per page load */
  }
  if (Date.now() - last < 60_000) return;
  try {
    sessionStorage.setItem("ignis:chunk-reload", String(Date.now()));
  } catch {
    /* ignore */
  }
  event.preventDefault();
  window.location.reload();
});

const client = new QueryClient({ defaultOptions: { queries: { staleTime: 30_000, refetchOnWindowFocus: false } } });

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={client}>
      <App />
    </QueryClientProvider>
  </StrictMode>,
);
