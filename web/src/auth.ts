// Google sign-in for a UI served apart from the API (Vercel → `ignis serve --auth google`).
//
// Off unless the build sets VITE_GOOGLE_CLIENT_ID: then local dev, the Vite proxy and the Worker
// deployment behave exactly as before. When on:
// - the user signs in with Google Identity Services (GIS) before the app renders;
// - the ID token is kept in memory only (never localStorage) and sent as
//   `Authorization: Bearer <token>` on every request to `/api/*`;
// - `/api/*` requests go to VITE_API_BASE (e.g. https://api.example.org) when it is set;
// - the token lasts about an hour: GIS is asked for a fresh one a few minutes before it expires
//   (silent when the browser allows it, otherwise the sign-in button comes back), and a 401 from
//   the API does the same, then retries once.
// All API traffic in this app goes through `fetch` (api.ts, research/http.ts, which also carries the
// event stream), so one wrapper covers it. A plain <a href>/<img src> to an API URL cannot carry the
// header: such links must fetch the bytes and use a blob URL instead.

const CLIENT_ID = (import.meta.env.VITE_GOOGLE_CLIENT_ID as string | undefined)?.trim() || "";
const API_BASE = ((import.meta.env.VITE_API_BASE as string | undefined)?.trim() || "").replace(/\/+$/, "");
const GIS_SRC = "https://accounts.google.com/gsi/client";
/** Ask for a new token this long before the current one expires. */
const REFRESH_MARGIN_MS = 5 * 60_000;

interface GisCredential {
  credential: string;
}
interface GisId {
  initialize(o: {
    client_id: string;
    callback: (c: GisCredential) => void;
    auto_select?: boolean;
    use_fedcm_for_prompt?: boolean;
    cancel_on_tap_outside?: boolean;
  }): void;
  prompt(): void;
  renderButton(el: HTMLElement, o: Record<string, unknown>): void;
  disableAutoSelect(): void;
}
declare global {
  interface Window {
    google?: { accounts: { id: GisId } };
  }
}

let token: string | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;
let waiters: Array<() => void> = [];
let overlay: HTMLDivElement | null = null;

export const googleSignInEnabled = CLIENT_ID !== "";

/** `exp` of a JWT, in ms (no verification: the server verifies; this only schedules a refresh). */
function expiryMs(jwt: string): number | null {
  try {
    const p = jwt.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
    const exp = JSON.parse(atob(p.padEnd(p.length + ((4 - (p.length % 4)) % 4), "="))).exp;
    return typeof exp === "number" ? exp * 1000 : null;
  } catch {
    return null;
  }
}

function gis(): GisId {
  const id = window.google?.accounts.id;
  if (!id) throw new Error("Google Identity Services did not load");
  return id;
}

function loadGis(): Promise<void> {
  if (window.google?.accounts.id) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = GIS_SRC;
    s.async = true;
    s.onload = () => resolve();
    s.onerror = () => reject(new Error(`could not load ${GIS_SRC}`));
    document.head.appendChild(s);
  });
}

function onCredential(c: GisCredential) {
  token = c.credential;
  hideOverlay();
  clearTimeout(refreshTimer);
  const exp = expiryMs(token);
  if (exp !== null) {
    refreshTimer = setTimeout(requestFreshToken, Math.max(10_000, exp - Date.now() - REFRESH_MARGIN_MS));
  }
  const w = waiters;
  waiters = [];
  w.forEach((f) => f());
}

/** Silent re-issue when the browser allows it; the button stays available as the fallback. */
function requestFreshToken() {
  showOverlay();
  try {
    gis().prompt();
  } catch {
    /* the button is shown */
  }
}

function showOverlay() {
  if (overlay) return;
  overlay = document.createElement("div");
  overlay.setAttribute("role", "dialog");
  overlay.setAttribute("aria-label", "Sign in");
  overlay.style.cssText =
    "position:fixed;inset:0;z-index:9999;display:flex;flex-direction:column;gap:16px;align-items:center;justify-content:center;background:Canvas;color:CanvasText;font:16px system-ui,sans-serif;padding:16px;text-align:center";
  const title = document.createElement("div");
  title.textContent = token ? "Your session is about to expire. Sign in again to continue." : "Sign in to IGNIS";
  const button = document.createElement("div");
  overlay.append(title, button);
  document.body.appendChild(overlay);
  try {
    gis().renderButton(button, { type: "standard", theme: "outline", size: "large", text: "signin_with" });
  } catch (e) {
    title.textContent = `Sign-in is unavailable: ${(e as Error).message}`;
  }
}

function hideOverlay() {
  overlay?.remove();
  overlay = null;
}

function waitForToken(): Promise<void> {
  return token ? Promise.resolve() : new Promise((r) => waiters.push(r));
}

function isApi(url: string): boolean {
  return url.startsWith("/api/") || url === "/api";
}

/**
 * Route `/api/*` to VITE_API_BASE and attach the Google ID token; a 401 asks for a new token and
 * retries once. Installed only when Google sign-in is configured.
 */
function installFetch() {
  const original = window.fetch.bind(window);
  window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    if (typeof input !== "string" || !isApi(input)) return original(input, init);
    const send = () => {
      const headers = new Headers(init?.headers);
      if (token) headers.set("authorization", `Bearer ${token}`);
      return original(API_BASE + input, { ...init, headers });
    };
    const res = await send();
    if (res.status !== 401 || init?.signal?.aborted) return res;
    token = null;
    requestFreshToken();
    await waitForToken();
    return send();
  };
}

/** Resolve once the app may render: immediately without Google sign-in, after sign-in with it. */
export async function ensureSignedIn(): Promise<void> {
  if (!googleSignInEnabled) return;
  await loadGis();
  gis().initialize({
    client_id: CLIENT_ID,
    callback: onCredential,
    auto_select: true,
    use_fedcm_for_prompt: true,
    cancel_on_tap_outside: false,
  });
  installFetch();
  requestFreshToken();
  await waitForToken();
}

/** Forget the token and stop automatic re-sign-in on this browser. */
export function signOut() {
  token = null;
  clearTimeout(refreshTimer);
  window.google?.accounts.id.disableAutoSelect();
  showOverlay();
}
