"""Capture the first model request the pinned Codex sends for an IGNIS episode, without a model.

A local capturing model provider (answers "ok") and a stub IGNIS MCP server (lists the catalog
tools, never runs one) stand in for OpenAI and the host; `codex exec` runs once with the IGNIS
public-profile switches, base and developer instructions. Prints the request's parts and sizes.

    IGNIS_PREFIX_DUMP=runs/prefix cargo test -p ignis-mcp --test prefix_size
    python tools/codex/prefix_capture.py --codex <native codex binary> --dump runs/prefix [--legacy]

`--legacy` leaves out the switches added in docs/research/HARNESS_COST.md (the state before).
No network beyond 127.0.0.1, no tokens spent. The request bodies stay in <dump>/capture/.
"""
import argparse, json, os, subprocess, sys, tempfile, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

FEATURES_OFF = ["shell_tool", "unified_exec", "shell_snapshot", "computer_use", "browser_use",
                "browser_use_external", "in_app_browser", "image_generation", "apps", "plugins", "hooks",
                "memories", "multi_agent", "goals", "view_image", "sleep_tool"]


def serve(handler):
    srv = ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--codex", required=True)
    ap.add_argument("--dump", required=True)
    ap.add_argument("--model", default="gpt-6-luna")
    ap.add_argument("--legacy", action="store_true")
    a = ap.parse_args()
    tools = json.load(open(os.path.join(a.dump, "tools.json"), encoding="utf-8"))
    out = os.path.join(a.dump, "capture")
    os.makedirs(out, exist_ok=True)
    bodies = []

    class Model(BaseHTTPRequestHandler):
        def do_POST(self):
            body = self.rfile.read(int(self.headers.get("content-length", 0)))
            bodies.append(body)
            open(os.path.join(out, f"request{len(bodies)}.json"), "wb").write(body)
            self.send_response(200)
            self.send_header("content-type", "text/event-stream")
            self.end_headers()
            for e in [{"type": "response.created", "response": {"id": "r"}},
                      {"type": "response.output_item.done", "item": {
                          "type": "message", "role": "assistant", "id": "m",
                          "content": [{"type": "output_text", "text": "ok"}]}},
                      {"type": "response.completed", "response": {"id": "r", "usage": {
                          "input_tokens": 1, "input_tokens_details": {"cached_tokens": 0},
                          "output_tokens": 1, "output_tokens_details": {"reasoning_tokens": 0},
                          "total_tokens": 2}}}]:
                self.wfile.write(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n".encode())

        def log_message(self, *_):
            pass

    class Mcp(BaseHTTPRequestHandler):
        def do_POST(self):
            req = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))) or b"{}")
            if "id" not in req:
                self.send_response(202)
                self.end_headers()
                return
            res = {"tools": tools} if req.get("method") == "tools/list" else {}
            if req.get("method") == "initialize":
                res = {"protocolVersion": req["params"].get("protocolVersion", "2025-06-18"),
                       "capabilities": {"tools": {}}, "serverInfo": {"name": "ignis", "version": "0"}}
            b = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": res}).encode()
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(b)))
            self.end_headers()
            self.wfile.write(b)

        def do_GET(self):
            self.send_response(405)
            self.end_headers()

        def log_message(self, *_):
            pass

    model, mcp = serve(Model), serve(Mcp)
    q = json.dumps
    base = os.path.abspath(os.path.join(a.dump, "codex_base.md")).replace("\\", "/")
    dev = open(os.path.join(a.dump, "harness.md"), encoding="utf-8").read()
    new = [] if a.legacy else ["agents.enabled = false", "include_environment_context = false"]
    lines = [f"model = {q(a.model)}", 'model_provider = "capture"', f"model_instructions_file = {q(base)}",
             f"developer_instructions = {q(dev)}", 'web_search = "disabled"', 'sandbox_mode = "read-only"',
             'approval_policy = "on-request"', "include_permissions_instructions = false",
             "include_collaboration_mode_instructions = false", "include_apps_instructions = false",
             'cli_auth_credentials_store = "file"', *new, "[features]", *[f"{k} = false" for k in FEATURES_OFF],
             "[skills]", "include_instructions = false", "[skills.bundled]", "enabled = false",
             "[model_providers.capture]", 'name = "capture"',
             f'base_url = "http://127.0.0.1:{model.server_port}/v1"', 'wire_api = "responses"',
             'env_key = "IGNIS_CAPTURE_KEY"', "[mcp_servers.ignis]",
             f'url = "http://127.0.0.1:{mcp.server_port}/mcp"', f"enabled_tools = {q([t['name'] for t in tools])}",
             'default_tools_approval_mode = "approve"']
    if not a.legacy:
        lines.append("supports_parallel_tool_calls = true")
    with tempfile.TemporaryDirectory() as home:
        open(os.path.join(home, "config.toml"), "w", encoding="utf-8").write("\n".join(lines) + "\n")
        env = {k: v for k, v in os.environ.items() if not k.endswith("API_KEY")}
        env.update(CODEX_HOME=home, IGNIS_CAPTURE_KEY="capture")
        subprocess.run([a.codex, "exec", "--skip-git-repo-check", "What is the extinction limit?"],
                       cwd=home, env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=120)
    if not bodies:
        sys.exit("codex sent no model request")
    d = json.loads(bodies[0])
    print(f"first request: {len(bodies[0])} bytes")
    print({k: d[k] for k in ("parallel_tool_calls", "reasoning", "prompt_cache_key") if k in d})
    for item in d["input"]:
        size = len(json.dumps(item))
        if item.get("type") == "additional_tools":
            print(f"  tools{size:>14}")
            for ns in item["tools"]:
                for t in ns.get("tools", [ns]):
                    print(f"    {ns.get('name')}.{t.get('name'):<28}{len(json.dumps(t)):>8}")
        else:
            text = item.get("content", [{}])[0].get("text", "")[:60].replace("\n", " ")
            print(f"  {item.get('role', item.get('type')):<10}{size:>9}  {text}")


if __name__ == "__main__":
    main()
