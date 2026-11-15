/**
 * IGNIS edge Worker: serves the built UI (Workers Static Assets) and forwards `/api/*` to the
 * origin through Cloudflare Tunnel. It owns no research state, no jobs and no SQLite.
 *
 * Trust chain (infra/README.md §Identity):
 * - The UI hostname is an Access application: a request reaching this Worker carries the user's
 *   signed `Cf-Access-Jwt-Assertion`.
 * - The origin hostname is a second Access application that admits only this Worker's service
 *   token. The Worker sends that token and copies the user's assertion into
 *   `X-Ignis-User-Assertion`; the origin verifies both signatures (ignis-server `v1::access`).
 * - Client-supplied copies of these headers are dropped, never forwarded.
 *
 * Streaming: request and response bodies are passed through as streams (no `.text()`/`.json()`),
 * so SSE, large artifacts and uploads are not buffered here.
 *
 * NTRS proxy (M12, LANES_V2 §10): `/ntrs/api/citations/<id>/downloads/<file>` streams that one
 * NTRS download to the signed-in user. NTRS answers Range but sends no CORS headers, so PDF.js
 * cannot read it cross-origin; same-origin through here it can. Only that path shape on the one
 * allow-listed host is forwarded, `Range` passes through, redirects are not followed, and nothing
 * is cached or stored (`cache: "no-store"`, `cache-control: private, no-store`).
 */

export interface Env {
  ASSETS: Fetcher;
  /** `https://origin.example.org` — the tunnel hostname (no trailing slash). */
  ORIGIN_URL: string;
  /** Service token for the origin's Access application (`wrangler secret put`). */
  ORIGIN_CLIENT_ID: string;
  ORIGIN_CLIENT_SECRET: string;
}

/** Headers the client must never be able to set on the origin request. */
const STRIPPED = [
  "x-ignis-user-assertion",
  "cf-access-client-id",
  "cf-access-client-secret",
  "cf-access-jwt-assertion",
  "cf-access-authenticated-user-email",
  "cookie",
  "authorization",
];

export const USER_ASSERTION = "x-ignis-user-assertion";

function problem(status: number, code: string, message: string): Response {
  // Same shape as the origin's ErrorEnvelope.
  return Response.json(
    { code, message, retry: { kind: status >= 500 ? "after_delay" : "after_user_action", ...(status >= 500 ? { seconds: 30 } : {}) } },
    { status, headers: { "cache-control": "no-store" } },
  );
}

export async function forward(request: Request, env: Env): Promise<Response> {
  const assertion = request.headers.get("cf-access-jwt-assertion");
  if (!assertion) {
    // Access in front of the UI hostname always sets it; its absence means a misrouted request.
    return problem(401, "scope_denied", "not authenticated: no Access assertion on the request");
  }
  if (!env.ORIGIN_URL || !env.ORIGIN_CLIENT_ID || !env.ORIGIN_CLIENT_SECRET) {
    return problem(503, "capability_unavailable", "origin is not configured");
  }
  const url = new URL(request.url);
  const target = new URL(url.pathname + url.search, env.ORIGIN_URL);
  const headers = new Headers(request.headers);
  for (const h of STRIPPED) headers.delete(h);
  headers.set(USER_ASSERTION, assertion);
  headers.set("cf-access-client-id", env.ORIGIN_CLIENT_ID);
  headers.set("cf-access-client-secret", env.ORIGIN_CLIENT_SECRET);
  const hasBody = request.method !== "GET" && request.method !== "HEAD";
  const init: RequestInit = {
    method: request.method,
    headers,
    body: hasBody ? request.body : null,
    redirect: "manual",
  };
  let upstream: Response;
  try {
    upstream = await fetch(target.toString(), init);
  } catch (e) {
    return problem(502, "upstream_unavailable", `origin unreachable: ${String(e)}`);
  }
  // An Access login redirect from the origin app means the service token was refused: report it
  // instead of sending the browser to a login page for a hostname it cannot use.
  if (upstream.status >= 300 && upstream.status < 400 && upstream.headers.get("location")?.includes("cloudflareaccess.com")) {
    return problem(502, "upstream_unavailable", "origin refused the Worker's service token");
  }
  const out = new Headers(upstream.headers);
  if ((out.get("content-type") ?? "").startsWith("text/event-stream")) {
    out.set("cache-control", "no-cache");
    out.delete("content-length");
  }
  return new Response(upstream.body, { status: upstream.status, statusText: upstream.statusText, headers: out });
}

/** The one upstream of the proxy. */
export const NTRS_ORIGIN = "https://ntrs.nasa.gov";
/** `/ntrs/api/citations/<11-digit id>/downloads/<file>`: the only forwarded shape. */
const NTRS_PATH = /^\/ntrs(\/api\/citations\/\d{11}\/downloads\/[A-Za-z0-9_-][A-Za-z0-9._-]*)$/;
/** Request headers the proxy forwards (everything else, cookies and identity included, is dropped). */
const NTRS_REQUEST_HEADERS = ["range", "if-range", "if-none-match", "if-modified-since", "accept"];
/** Response headers the proxy returns (no `set-cookie`, no `x-frame-options` of NTRS's own site). */
const NTRS_RESPONSE_HEADERS = ["content-type", "content-length", "content-range", "accept-ranges", "etag", "last-modified"];

export async function ntrsProxy(request: Request): Promise<Response> {
  if (request.method !== "GET" && request.method !== "HEAD") {
    return problem(405, "invalid_request", "the NTRS proxy only reads (GET, HEAD)");
  }
  // The UI hostname is behind Access; without its assertion this is not a signed-in user and the
  // Worker must not act as an open proxy.
  if (!request.headers.get("cf-access-jwt-assertion")) {
    return problem(401, "scope_denied", "not authenticated: no Access assertion on the request");
  }
  const url = new URL(request.url);
  const m = url.search === "" ? NTRS_PATH.exec(url.pathname) : null;
  if (!m) {
    return problem(404, "not_found", "not an NTRS download path");
  }
  const headers = new Headers();
  for (const h of NTRS_REQUEST_HEADERS) {
    const v = request.headers.get(h);
    if (v !== null) headers.set(h, v);
  }
  let upstream: Response;
  try {
    upstream = await fetch(NTRS_ORIGIN + m[1], { method: request.method, headers, redirect: "manual", cache: "no-store" });
  } catch (e) {
    return problem(502, "upstream_unavailable", `NTRS unreachable: ${String(e)}`);
  }
  if (upstream.status >= 300 && upstream.status < 400 && upstream.status !== 304) {
    // A redirect could lead anywhere; the proxy reaches only the allow-listed host.
    return problem(502, "upstream_unavailable", `NTRS redirected (${upstream.status}); not followed`);
  }
  if (upstream.status >= 500) {
    // An NTRS outage is "unavailable, retry later", never "not found"; its error page is not passed on.
    await upstream.body?.cancel();
    return problem(503, "upstream_unavailable", `NTRS answered ${upstream.status}`);
  }
  const out = new Headers();
  for (const h of NTRS_RESPONSE_HEADERS) {
    const v = upstream.headers.get(h);
    if (v !== null) out.set(h, v);
  }
  out.set("cache-control", "private, no-store");
  out.set("x-content-type-options", "nosniff");
  return new Response(upstream.body, { status: upstream.status, statusText: upstream.statusText, headers: out });
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const { pathname } = new URL(request.url);
    if (pathname === "/api" || pathname.startsWith("/api/")) {
      return forward(request, env);
    }
    if (pathname === "/ntrs" || pathname.startsWith("/ntrs/")) {
      return ntrsProxy(request);
    }
    // Everything else is the UI (SPA fallback is configured on the assets binding).
    return env.ASSETS.fetch(request);
  },
} satisfies ExportedHandler<Env>;
