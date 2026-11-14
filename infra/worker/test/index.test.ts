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
