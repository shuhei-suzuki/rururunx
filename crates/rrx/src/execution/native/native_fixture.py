# Local protocol fixture only: no provider, account, model, network or authentication.
import json
import os
import subprocess
import sys
import time

if sys.argv[1:] == ["--version"]:
    print("codex-cli 0.160.0" if PROVIDER == "codex" else "2.1.283 (Claude Code)")
    sys.exit(0)

def send(value):
    print(json.dumps(value), flush=True)

def complete(payload):
    with open(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-ready"), "w") as ready:
        ready.write("ready\n")
    release = os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-release")
    while not os.path.exists(release):
        time.sleep(0.02)
    with open("fixture-result.txt", "w") as result:
        result.write(os.environ["RRX_UNIT_ID"] + "\n")
    # The fixture qualifies protocol correlation, not the installed shim/native-agent matrix.
    subprocess.run(["/usr/bin/git", "add", "fixture-result.txt"], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(["/usr/bin/git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false", "commit", "-m", "protocol fixture"], check=True, stdout=subprocess.DEVNULL)

for line in sys.stdin:
    value = json.loads(line)
    if PROVIDER == "codex":
        method = value.get("method")
        if method == "initialized":
            continue
        if method == "initialize":
            result = {"codexHome": "/fixture/native-home", "userAgent": "fixture", "platformFamily": "unix", "platformOs": "macos" if sys.platform == "darwin" else "linux"}
        elif method == "account/read":
            result = {"account": {"type": "chatgpt", "planType": "pro"}}
        elif method == "environment/status":
            result = {"status": "ready"}
        elif method == "account/rateLimits/read":
            result = {"rateLimits": {"limitId": "fixture", "primary": {"usedPercent": 20}, "secondary": None}}
        elif method == "thread/start":
            result = {"thread": {"id": os.environ["RRX_UNIT_ID"]}, "cwd": os.getcwd(), "modelProvider": "openai"}
        elif method == "turn/start":
            send({"id": value["id"], "result": {"turn": {"id": "fixture-turn"}}})
            complete(value["params"]["input"][0]["text"])
            send({"method": "turn/completed", "params": {"threadId": os.environ["RRX_UNIT_ID"], "turn": {"id": "fixture-turn", "status": "completed"}}})
            continue
        else:
            send({"id": value["id"], "error": {"code": -32601}})
            continue
        send({"id": value["id"], "result": result})
    else:
        native = sys.argv[sys.argv.index("--session-id") + 1]
        if value["type"] == "control_request":
            send({"type": "control_response", "response": {"subtype": "success", "request_id": value["request_id"], "response": {}}})
        elif value["type"] == "user":
            send({"type": "system", "subtype": "init", "session_id": native, "cwd": os.getcwd(), "tools": ["Bash"], "mcp_servers": []})
            complete(value["message"]["content"])
            send({"type": "result", "session_id": native, "subtype": "success", "is_error": False, "result": "fixture complete"})
