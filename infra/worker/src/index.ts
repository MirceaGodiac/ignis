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

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const { pathname } = new URL(request.url);
    if (pathname === "/api" || pathname.startsWith("/api/")) {
      return forward(request, env);
    }
    // Everything else is the UI (SPA fallback is configured on the assets binding).
    return env.ASSETS.fetch(request);
  },
} satisfies ExportedHandler<Env>;
