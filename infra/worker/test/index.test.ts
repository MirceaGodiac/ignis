import { afterEach, describe, expect, it, vi } from "vitest";
import worker, { type Env, USER_ASSERTION } from "../src/index";

const env = (over: Partial<Env> = {}): Env => ({
  ASSETS: { fetch: vi.fn(async () => new Response("ui")) } as unknown as Fetcher,
  ORIGIN_URL: "https://origin.example.org",
  ORIGIN_CLIENT_ID: "cid.access",
  ORIGIN_CLIENT_SECRET: "secret",
  ...over,
});

afterEach(() => vi.unstubAllGlobals());

function captureFetch(response: Response = new Response("{}")) {
  const seen: Request[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string, init: RequestInit) => {
      seen.push(new Request(url, init));
      return response;
    }),
  );
  return seen;
}

describe("api forwarding", () => {
  it("sends the service token and the user's assertion, never client-supplied identity", async () => {
    const seen = captureFetch();
    const req = new Request("https://ignis.example.org/api/v1/episodes?after=3", {
      headers: {
        "cf-access-jwt-assertion": "user.jwt.sig",
        [USER_ASSERTION]: "forged.jwt.sig",
        "cf-access-client-id": "attacker",
        "cf-access-authenticated-user-email": "admin@example.org",
        cookie: "CF_Authorization=user.jwt.sig",
        "last-event-id": "7",
      },
    });
    const res = await worker.fetch(req, env());
    expect(res.status).toBe(200);
    const sent = seen[0]!;
    expect(sent.url).toBe("https://origin.example.org/api/v1/episodes?after=3");
    expect(sent.headers.get(USER_ASSERTION)).toBe("user.jwt.sig");
    expect(sent.headers.get("cf-access-client-id")).toBe("cid.access");
    expect(sent.headers.get("cf-access-client-secret")).toBe("secret");
    expect(sent.headers.get("cf-access-jwt-assertion")).toBeNull();
    expect(sent.headers.get("cf-access-authenticated-user-email")).toBeNull();
    expect(sent.headers.get("cookie")).toBeNull();
    expect(sent.headers.get("last-event-id")).toBe("7");
  });

  it("refuses a request without an Access assertion and does not call the origin", async () => {
    const seen = captureFetch();
    const res = await worker.fetch(new Request("https://ignis.example.org/api/v1/episodes"), env());
    expect(res.status).toBe(401);
    expect(((await res.json()) as { code: string }).code).toBe("scope_denied");
    expect(seen).toHaveLength(0);
  });

  it("streams the event stream through unbuffered", async () => {
    const enc = new TextEncoder();
    const stream = new ReadableStream({
      start(c) {
        c.enqueue(enc.encode("id: 1\nevent: x\ndata: {}\n\n"));
      },
    });
    captureFetch(new Response(stream, { headers: { "content-type": "text/event-stream" } }));
    const res = await worker.fetch(
      new Request("https://ignis.example.org/api/v1/episodes/e/events", {
        headers: { "cf-access-jwt-assertion": "u" },
      }),
      env(),
    );
    expect(res.headers.get("cache-control")).toBe("no-cache");
    expect(res.body).toBeInstanceOf(ReadableStream);
    const { value } = await res.body!.getReader().read();
    expect(new TextDecoder().decode(value)).toContain("id: 1");
  });

  it("reports a refused service token instead of redirecting the browser", async () => {
    captureFetch(
      new Response(null, {
        status: 302,
        headers: { location: "https://team.cloudflareaccess.com/cdn-cgi/access/login" },
      }),
    );
    const res = await worker.fetch(
      new Request("https://ignis.example.org/api/v1/capabilities", {
        headers: { "cf-access-jwt-assertion": "u" },
      }),
      env(),
    );
    expect(res.status).toBe(502);
  });

  it("is unavailable, not open, when the origin is not configured", async () => {
    const seen = captureFetch();
    const res = await worker.fetch(
      new Request("https://ignis.example.org/api/v1/capabilities", {
        headers: { "cf-access-jwt-assertion": "u" },
      }),
      env({ ORIGIN_CLIENT_SECRET: "" }),
    );
    expect(res.status).toBe(503);
    expect(seen).toHaveLength(0);
  });
});

describe("ui", () => {
  it("serves everything else from the assets binding", async () => {
    const e = env();
    const res = await worker.fetch(new Request("https://ignis.example.org/episodes/42"), e);
    expect(await res.text()).toBe("ui");
    expect(e.ASSETS.fetch).toHaveBeenCalledOnce();
  });
});

describe("ntrs proxy", () => {
  const PDF = "https://ignis.example.org/ntrs/api/citations/20040142357/downloads/20040142357.pdf";
  const signedIn = (extra: Record<string, string> = {}) => ({ "cf-access-jwt-assertion": "u", ...extra });

  it("streams one NTRS download with Range forwarded and nothing identifying or cacheable", async () => {
    const enc = new TextEncoder();
    const body = new ReadableStream({
      start: (c) => {
        c.enqueue(enc.encode("%PDF-1.4"));
        c.close();
      },
    });
    const seen: Array<{ url: string; init: RequestInit }> = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init: RequestInit) => {
        seen.push({ url, init });
        return new Response(body, {
          status: 206,
          headers: {
            "content-type": "application/pdf",
            "content-range": "bytes 0-7/123456",
            "content-length": "8",
            "accept-ranges": "bytes",
            etag: '"abc"',
            "set-cookie": "connect.sid=s; HttpOnly",
            "x-frame-options": "SAMEORIGIN",
          },
        });
      }),
    );
    const res = await worker.fetch(
      new Request(PDF, { headers: signedIn({ range: "bytes=0-7", cookie: "CF_Authorization=x", authorization: "Bearer t" }) }),
      env(),
    );
    expect(seen).toHaveLength(1);
    expect(seen[0]!.url).toBe("https://ntrs.nasa.gov/api/citations/20040142357/downloads/20040142357.pdf");
    const sent = new Headers(seen[0]!.init.headers);
    expect(sent.get("range")).toBe("bytes=0-7");
    expect(sent.get("cookie")).toBeNull();
    expect(sent.get("authorization")).toBeNull();
    expect(sent.get("cf-access-jwt-assertion")).toBeNull();
    expect(seen[0]!.init.redirect).toBe("manual");
    expect(seen[0]!.init.cache).toBe("no-store");
    expect(res.status).toBe(206);
    expect(res.headers.get("content-range")).toBe("bytes 0-7/123456");
    expect(res.headers.get("cache-control")).toBe("private, no-store");
    expect(res.headers.get("set-cookie")).toBeNull();
    expect(res.headers.get("x-frame-options")).toBeNull();
    expect(res.body).toBeInstanceOf(ReadableStream);
    expect(await res.text()).toBe("%PDF-1.4");
  });

  it("forwards only the allow-listed path shape, never other hosts, queries or methods", async () => {
    const seen = captureFetch();
    for (const path of [
      "/ntrs/api/citations/2004014235/downloads/a.pdf", // 10 digits
      "/ntrs/api/citations/20040142357/downloads/.env",
      "/ntrs/api/citations/20040142357/downloads/a%2F..%2Fb.pdf",
      "/ntrs/api/citations/20040142357/downloads/a.pdf?x=https://evil.example.org",
      "/ntrs/api/citations/20040142357",
      "/ntrs/https://evil.example.org/x.pdf",
      "/ntrs",
    ]) {
      const res = await worker.fetch(new Request(`https://ignis.example.org${path}`, { headers: signedIn() }), env());
      expect(res.status, path).toBe(404);
    }
    const post = await worker.fetch(new Request(PDF, { method: "POST", body: "x", headers: signedIn() }), env());
    expect(post.status).toBe(405);
    expect(seen).toHaveLength(0);
  });

  it("is not an open proxy: no Access assertion, no request upstream", async () => {
    const seen = captureFetch();
    const res = await worker.fetch(new Request(PDF), env());
    expect(res.status).toBe(401);
    expect(seen).toHaveLength(0);
  });

  it("reports an NTRS outage as unavailable with a retry hint, never not found", async () => {
    captureFetch(new Response("<html>bad gateway</html>", { status: 502 }));
    const res = await worker.fetch(new Request(PDF, { headers: signedIn() }), env());
    expect(res.status).toBe(503);
    const body = (await res.json()) as { code: string; message: string; retry: { kind: string } };
    expect(body.code).toBe("upstream_unavailable");
    expect(body.retry.kind).toBe("after_delay");
    expect(body.message).toContain("502");
  });

  it("reports an unreachable NTRS as unavailable", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => { throw new TypeError("network down"); }));
    const res = await worker.fetch(new Request(PDF, { headers: signedIn() }), env());
    expect(res.status).toBe(502);
    expect(((await res.json()) as { code: string }).code).toBe("upstream_unavailable");
  });

  it("does not follow a redirect off the allow-listed host", async () => {
    captureFetch(new Response(null, { status: 302, headers: { location: "https://evil.example.org/x.pdf" } }));
    const res = await worker.fetch(new Request(PDF, { headers: signedIn() }), env());
    expect(res.status).toBe(502);
    expect(res.headers.get("location")).toBeNull();
  });
});
