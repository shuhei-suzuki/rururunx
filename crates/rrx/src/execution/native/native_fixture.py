# Local protocol fixture only: no provider, account, model, network or authentication.
import json
import os
import subprocess
import sys
import time

if sys.argv[1:] == ["--version"]:
    print("codex-cli 0.160.0" if PROVIDER == "codex" else "2.1.294 (Claude Code)")
    sys.exit(0)

def send(value):
    print(json.dumps(value), flush=True)

def hold_bootstrap():
    if globals().get("BOOTSTRAP_HOLD", False):
        with open(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-bootstrap-ready"), "w") as ready:
            ready.write("ready")
        while True: time.sleep(0.02)

def complete(payload):
    if payload == "readonly-review":
        if PROVIDER == "claude":
            assert sys.argv[sys.argv.index("--permission-mode") + 1] == "plan"
        else:
            assert NATIVE_SANDBOX == "read-only"
        subprocess.run(["/usr/bin/git", "rev-parse", "HEAD"], check=True, stdout=subprocess.DEVNULL)
        try:
            with open("fixture-review-write", "w") as out: out.write("must be refused")
        except PermissionError:
            pass
        else:
            raise RuntimeError("readonly source accepted a normal write")
        return
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
    # A fixed test-peer scenario selects a native outcome without changing the
    # Runtime's immutable ContextVersion or outgoing input.
    scenario = globals().get("WORKFLOW_SCENARIO")
    if scenario and value.get("method") == "turn/start":
        value["params"]["input"][0]["text"] = scenario
    elif scenario and value.get("type") == "user":
        value["message"]["content"] = scenario
    if PROVIDER == "codex":
        method = value.get("method")
        failure = globals().get("BOOTSTRAP_CASE")
        if method == "initialize" and failure == "transport":
            sys.exit(0)
        if method == "initialize" and failure == "protocol":
            print("fixture deliberately malformed JSON",flush=True)
            continue
        if method == "environment/status" and failure == "rpc-unsupported":
            send({"id":value["id"],"error":{"code":-32601,"message":"PRIVATE_FIXTURE_ERROR_MUST_NOT_PERSIST"}})
            continue
        if method == "initialized":
            continue
        if method == "initialize":
            hold_bootstrap()
            result = {"codexHome": "/fixture/native-home", "userAgent": "fixture", "platformFamily": "unix", "platformOs": "macos" if sys.platform == "darwin" else "linux"}
        elif method == "account/read":
            result = {"account": None if failure == "auth" else {"type": "apiKey" if failure == "api" else "chatgpt", "planType": "pro"}}
        elif method == "environment/status":
            result = {"status": "unsupported" if failure == "environment" else "ready"}
        elif method == "account/rateLimits/read":
            result = {} if failure == "metadata" else {"rateLimits": {"limitId": "fixture", "primary": {"usedPercent": 20}, "secondary": None}}
        elif method == "thread/start":
            NATIVE_SANDBOX = value["params"].get("sandbox")
            result = {"thread": {"id": os.environ["RRX_UNIT_ID"]}, "cwd": os.getcwd(), "modelProvider": "openai"}
        elif method == "turn/start":
            send({"id": value["id"], "result": {"turn": {"id": "fixture-turn"}}})
            payload = value["params"]["input"][0]["text"]
            if payload.startswith("answer-"):
                if payload in ("answer-item-overflow","answer-event-overflow"):
                    for i in range(257 if payload=="answer-item-overflow" else 1025):
                        if payload=="answer-item-overflow": send({"method":"item/completed","params":{"threadId":os.environ["RRX_UNIT_ID"],"turnId":"fixture-turn","completedAtMs":0,"item":{"id":str(i),"type":"agentMessage","phase":"commentary","text":"bounded commentary"}}})
                        else: send({"method":"item/agentMessage/delta","params":{"threadId":os.environ["RRX_UNIT_ID"],"turnId":"fixture-turn","itemId":"draft","delta":"d"}})
                    while True: time.sleep(0.02)
                item = {"id":"fixture-answer","type":"agentMessage","phase":"final_answer","text":"APPROVE actual answer"}
                def emit(i, thread=None, turn="fixture-turn"):
                    send({"method":"item/completed","params":{"threadId":thread or os.environ["RRX_UNIT_ID"],"turnId":turn,"completedAtMs":0,"item":i}})
                if payload == "answer-null-phase": item["phase"] = None
                if payload == "answer-encoded-overflow": item["text"] = "\x01" * 349450
                if payload == "answer-foreign": emit(dict(item,text="FOREIGN must not persist"),"foreign", "foreign-turn")
                if payload == "answer-commentary": emit(dict(item,id="commentary",phase="commentary",text="commentary must not become final"))
                if payload == "answer-delta-only":
                    send({"method":"item/agentMessage/delta","params":{"threadId":os.environ["RRX_UNIT_ID"],"turnId":"fixture-turn","itemId":"draft","delta":"draft only"}})
                elif payload == "answer-raw-duplicate":
                    print('{"method":"item/completed","params":{"threadId":'+json.dumps(os.environ["RRX_UNIT_ID"])+',"turnId":"fixture-turn","item":{"id":"fixture-answer","type":"agentMessage","phase":"final_answer","text":"first","text":"second"}}}',flush=True)
                elif payload == "answer-depth":
                    nested = None
                    for _ in range(33): nested = [nested]
                    emit(dict(item,extra=nested))
                elif payload != "answer-terminal-only": emit(item)
                if payload == "answer-duplicate": emit(item)
                if payload == "answer-change": emit(dict(item,text="CHANGED answer"))
                if payload == "answer-two-finals": emit(dict(item,id="second-final",text="SECOND answer"))
                if payload == "answer-hold-after-final":
                    send({"id":"answer-barrier","method":"item/commandExecution/requestApproval","params":{"threadId":os.environ["RRX_UNIT_ID"],"turnId":"fixture-turn","itemId":"barrier","command":"fixture no execution","cwd":os.getcwd()}})
                    with open(os.path.join(os.environ["RRX_OUTPUT_DIR"],"fixture-answer-ready"),"w") as ready: ready.write("ready")
                    while not os.path.exists(os.path.join(os.environ["RRX_OUTPUT_DIR"],"fixture-release")): time.sleep(0.02)
                turn = {"id":"fixture-turn","status":"failed" if payload == "answer-failed" else "completed","items":[item]}
                if payload == "answer-failed": turn["error"] = {"codexErrorInfo":"other"}
                if payload == "answer-summary": turn.update(items=[],itemsView="summary")
                if payload == "answer-delta-only": turn["items"] = []
                send({"method":"turn/completed","params":{"threadId":os.environ["RRX_UNIT_ID"],"turn":turn}})
                continue
            if payload.startswith("capacity-") or payload == "authentication-terminal":
                info = {"capacity-rate":"rateLimitExceeded", "capacity-flex":"flexUnavailable", "capacity-overload":"serverOverloaded", "capacity-http":{"httpConnectionFailed":{"httpStatusCode":429}}, "capacity-retry-success":"rateLimitExceeded", "authentication-terminal":"unauthorized"}[payload]
                if payload == "capacity-retry-success":
                    send({"method":"error", "params":{"threadId":os.environ["RRX_UNIT_ID"], "turnId":"fixture-turn", "willRetry":True, "error":{"codexErrorInfo":info}}})
                send({"method":"turn/completed", "params":{"threadId":os.environ["RRX_UNIT_ID"], "turn":{"id":"fixture-turn", "status":"completed" if payload == "capacity-retry-success" else "failed", "error":{"codexErrorInfo":info}}}})
            elif payload in ("quota-terminal", "quota-retry-terminal", "quota-retry-held", "ordinary-failure"):
                if payload in ("quota-retry-terminal", "quota-retry-held"):
                    send({"method": "error", "params": {"threadId": os.environ["RRX_UNIT_ID"], "turnId": "fixture-turn", "willRetry": True, "error": {"codexErrorInfo": "usageLimitExceeded"}}})
                if payload == "quota-retry-held":
                    while not os.path.exists(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-quota-release")): time.sleep(0.02)
                error = {"codexErrorInfo": "usageLimitExceeded" if payload != "ordinary-failure" else "other"}
                send({"method": "turn/completed", "params": {"threadId": os.environ["RRX_UNIT_ID"], "turn": {"id": "fixture-turn", "status": "failed", "error": error}}})
            else:
                complete(payload)
                item = {"id":"fixture-answer","type":"agentMessage","phase":"final_answer","text":"fixture complete"}
                send({"method":"item/completed","params":{"threadId":os.environ["RRX_UNIT_ID"],"turnId":"fixture-turn","completedAtMs":0,"item":item}})
                send({"method": "turn/completed", "params": {"threadId": os.environ["RRX_UNIT_ID"], "turn": {"id": "fixture-turn", "status": "completed","items":[item]}}})
            continue
        else:
            send({"id": value["id"], "error": {"code": -32601}})
            continue
        send({"id": value["id"], "result": result})
    else:
        native = sys.argv[sys.argv.index("--session-id") + 1]
        if value["type"] == "control_request":
            hold_bootstrap()
            send({"type": "control_response", "response": {"subtype": "success", "request_id": value["request_id"], "response": {}}})
        elif value["type"] == "user":
            send({"type": "system", "subtype": "init", "session_id": native, "cwd": os.getcwd(), "tools": ["Bash"], "mcp_servers": []})
            payload = value["message"]["content"]
            if payload.startswith("answer-"):
                if payload in ("answer-changing-held","answer-sameuuid-changing-held","answer-identical-held"):
                    send({"type":"system","subtype":"session_state_changed","session_id":native,"state":"running"})
                    a={"type":"result","session_id":native,"uuid":"answer-A","subtype":"success","is_error":False,"result":"APPROVE A"}
                    send(a)
                    send(dict(a,uuid="answer-A" if payload=="answer-sameuuid-changing-held" else "answer-B",result="APPROVE A" if payload=="answer-identical-held" else "APPROVE B"))
                    send({"type":"system","subtype":"session_state_changed","session_id":native,"state":"idle"})
                    continue
                frame = {"type":"result","session_id":native,"subtype":"success","is_error":payload == "answer-failed","result":"APPROVE actual answer"}
                if payload == "answer-structured": frame["structured_output"] = {"decision":"APPROVE"}
                if payload == "answer-encoded-overflow": frame["result"] = "\x01" * 349450
                if payload == "answer-foreign": send(dict(frame,session_id="foreign",result="FOREIGN must not persist"))
                if payload == "answer-raw-duplicate":
                    print('{"type":"result","session_id":'+json.dumps(native)+',"subtype":"success","is_error":false,"result":"first","result":"second"}',flush=True)
                elif payload == "answer-depth":
                    nested = None
                    for _ in range(33): nested = [nested]
                    send(dict(frame,extra=nested))
                else: send(frame)
                continue
            if payload in ("authentication-terminal", "authentication-structured", "unsupported-control"):
                if payload == "unsupported-control":
                    send({"type":"control_request", "request_id":"future-control", "request":{"subtype":"future-control","input":{}}})
                else:
                    if payload == "authentication-structured":
                        send({"type":"assistant", "session_id":native, "parent_tool_use_id":None, "error":"authentication_failed", "message":{"role":"assistant","content":[]}})
                    send({"type":"result", "session_id":native, "subtype":"error_during_execution", "is_error":True, "result":"Not logged in: PRIVATE_FIXTURE_ERROR_MUST_NOT_PERSIST" if payload == "authentication-terminal" else ""})
            elif payload.startswith("capacity-"):
                if payload == "capacity-unknown-window":
                    send({"type":"rate_limit_event", "session_id":native, "rate_limit_info":{"rateLimitType":"future_window", "status":"rejected"}})
                if payload in ("capacity-rate", "capacity-cap-control", "capacity-background-control", "capacity-retry-success"):
                    assistant = {"type":"assistant", "session_id":native, "parent_tool_use_id":None, "error":"rate_limit", "message":{"role":"assistant","content":[]}}
                    if payload == "capacity-background-control": assistant["parent_tool_use_id"] = "background-tool"
                    send(assistant)
                if payload == "capacity-retry-success":
                    send({"type":"result", "session_id":native, "subtype":"success", "is_error":False})
                else:
                    send({"type":"result", "session_id":native, "subtype":"error_max_turns" if payload == "capacity-cap-control" else "success" if payload == "capacity-http" else "error_during_execution", "is_error":True, "api_error_status":429 if payload == "capacity-http" else None})
            elif payload.startswith("quota-"):
                def quota(bucket, status, session=native, reset=None):
                    send({"type": "rate_limit_event", "session_id": session, "rate_limit_info": {"rateLimitType": bucket, "status": status, "resetsAt": int(time.time()) + 3600 if reset is None else reset}})
                quota("future_window" if payload == "quota-unknown-held" else "five_hour", "rejected", "foreign" if payload in ("quota-foreign", "quota-foreign-held") else native, int(time.time()) - 1 if payload == "quota-live-recovery-held" else None)
                quota("five_hour" if payload == "quota-stale-available" else "seven_day", "allowed")
                if payload.endswith("-held"):
                    # An ordinary permission notification is an ordered barrier:
                    # its published status proves the prior quota frames were read.
                    send({"type":"control_request", "request_id":"quota-barrier", "request":{"subtype":"can_use_tool", "tool_use_id":"fixture-barrier", "tool_name":"Bash", "input":{"command":"echo fixture"}}})
                    if payload == "quota-live-recovery-held":
                        while not os.path.exists(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-recovery-release")): time.sleep(0.02)
                        quota("five_hour", "allowed")
                        send({"type":"control_request", "request_id":"recovery-barrier", "request":{"subtype":"can_use_tool", "tool_use_id":"fixture-recovery-barrier", "tool_name":"Bash", "input":{"command":"echo fixture"}}})
                    while not os.path.exists(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-quota-release")): time.sleep(0.02)
                if payload in ("quota-retry-success-held", "quota-live-recovery-held"):
                    send({"type":"result", "session_id":native, "subtype":"success", "is_error":False, "result":"fixture successful native retry"})
                    continue
                subtype = "error_max_turns" if payload == "quota-budget-failure" else "error_during_execution"
                send({"type": "result", "session_id": native, "subtype": subtype, "is_error": True})
            else:
                complete(payload)
                send({"type": "result", "session_id": native, "subtype": "success", "is_error": False, "result": "fixture complete"})
