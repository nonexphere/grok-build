#!/usr/bin/env python3
"""C7-D stdio vertical smoke driver.

Spawns the built `stdio_smoke_server` example binary as a REAL subprocess with
piped stdin/stdout/stderr, then drives the shipped FacadeProcessor + stdio
NDJSON transport through:

    initialize -> session/start -> turn/start -> session/read -> session/subscribe

and asserts the primary observables (real session id, real turn id bound to
the session, non-empty transcript turns/items, replay events). The full
scripted exchange is printed to stdout so the calling shell script can capture
it into `/tmp/grok-goal-5598c3040156/implementer/smoke/stdio-vertical.txt`.

Runtime: FakeRuntime (in-memory contract fake). Processor + transport are the
shipped production stdio path.
"""
import json
import os
import subprocess
import sys

PROTOCOL_VERSION = "2026-07-18.experimental-v2"
BIN = sys.argv[1]
WORKSPACE = "/tmp/grok-goal-5598c3040156/smoke-workspace"


def main() -> int:
    print("================================================================")
    print("C7-D stdio vertical smoke")
    print("================================================================")
    print(f"[driver] python3 {os.path.basename(__file__)}")
    print(f"[bin] {BIN}")
    print("[runtime] FakeRuntime (in-memory contract fake)")
    print("[processor] FacadeProcessor (shipped)")
    print("[transport] run_stdio_loop NDJSON over real process stdin/stdout")
    print("")

    proc = subprocess.Popen(
        [BIN],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        bufsize=1,
        universal_newlines=True,
    )

    # Read the server's stderr banner line (merged to stdout via STDERR->STDOUT).
    banner = proc.stdout.readline().rstrip("\n")
    print(f"[server] {banner}")
    print("")

    results = []

    def record(ok):
        results.append(ok)

    def assert_eq(label, actual, expected):
        ok = actual == expected
        record(ok)
        if ok:
            print(f"ASSERT OK: {label} == {expected!r}")
        else:
            print(f"ASSERT FAIL: {label} expected {expected!r} got {actual!r}")

    def assert_present(label, value):
        ok = bool(value) and value != "null"
        record(ok)
        if ok:
            print(f"ASSERT OK: {label} present ({value!r})")
        else:
            print(f"ASSERT FAIL: {label} missing/empty")

    def assert_ge(label, actual, threshold):
        ok = actual >= threshold
        record(ok)
        if ok:
            print(f"ASSERT OK: {label} >= {threshold} (got {actual})")
        else:
            print(f"ASSERT FAIL: {label} expected >= {threshold} got {actual}")

    def send(label, request):
        line = json.dumps(request, separators=(",", ":"))
        print(f">>> {label} REQUEST: {line}")
        proc.stdin.write(line + "\n")
        proc.stdin.flush()
        resp_line = proc.stdout.readline()
        if not resp_line:
            print(f"<<< {label} RESPONSE: (no response / EOF)")
            print("ASSERT FAIL: no response from subprocess")
            return {}
        resp_line = resp_line.rstrip("\n")
        print(f"<<< {label} RESPONSE: {resp_line}")
        try:
            return json.loads(resp_line)
        except json.JSONDecodeError as e:
            print(f"ASSERT FAIL: response not valid JSON: {e}")
            return {}

    # 1. initialize
    init_resp = send("initialize", {
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "clientInfo": {"name": "stdio-smoke", "version": "0.1.0"},
            "capabilities": {},
        },
    })
    init_result = init_resp.get("result", {})
    assert_eq("initialize.protocolVersion", init_result.get("protocolVersion"), PROTOCOL_VERSION)
    assert_present("initialize.serverInstanceId", init_result.get("serverInstanceId"))

    # 2. session/start
    session_resp = send("session/start", {
        "jsonrpc": "2.0", "id": 2, "method": "session/start",
        "params": {
            "workspaceRoot": WORKSPACE,
            "idempotencyKey": "smoke-session-1",
        },
    })
    session = session_resp.get("result", {}).get("session", {})
    session_id = session.get("sessionId")
    assert_present("session/start.sessionId", session_id)
    assert_eq("session/start.workspaceRoot", session.get("workspaceRoot"), WORKSPACE)
    if session_id:
        print(f"[extract] sessionId={session_id}")

    # 3. turn/start (bound to the real session id)
    turn_resp = send("turn/start", {
        "jsonrpc": "2.0", "id": 3, "method": "turn/start",
        "params": {
            "sessionId": session_id,
            "input": [{"type": "text", "text": "hello stdio"}],
            "idempotencyKey": "smoke-turn-1",
        },
    })
    turn = turn_resp.get("result", {}).get("turn", {})
    turn_id = turn.get("turnId")
    assert_present("turn/start.turnId", turn_id)
    assert_eq("turn/start.turn.sessionId == session/start.sessionId", turn.get("sessionId"), session_id)
    assert_eq("turn/start.turn.kind", turn.get("kind"), "user")
    if turn_id:
        print(f"[extract] turnId={turn_id}")

    # 4. session/read (transcript: turns + items)
    read_resp = send("session/read", {
        "jsonrpc": "2.0", "id": 4, "method": "session/read",
        "params": {
            "sessionId": session_id,
            "includeTurns": True,
            "includeItems": True,
        },
    })
    read_result = read_resp.get("result", {})
    turns = read_result.get("turns", []) or []
    items = read_result.get("items", []) or []
    assert_eq("session/read.sessionId", read_result.get("session", {}).get("sessionId"), session_id)
    assert_ge("session/read.turns.length", len(turns), 1)
    assert_ge("session/read.items.length", len(items), 1)
    # Item bodies are flattened by the protocol serializer: an agent_message
    # item carries top-level `type=="agent_message"` and a top-level `text`.
    agent_texts = [
        i.get("text")
        for i in items
        if i.get("type") == "agent_message"
    ]
    agent_text = agent_texts[0] if agent_texts else None
    if agent_text:
        print(f"ASSERT OK: session/read agent_message text present ({agent_text!r})")
        record(True)
    else:
        print("ASSERT FAIL: session/read missing agent_message text")
        record(False)

    # 5. session/subscribe (replay events)
    sub_resp = send("session/subscribe", {
        "jsonrpc": "2.0", "id": 5, "method": "session/subscribe",
        "params": {
            "sessionId": session_id,
            "afterEventSeq": "0",
        },
    })
    replay = sub_resp.get("result", {}).get("replay", {})
    events = replay.get("events", []) or []
    replayed_through = replay.get("replayedThrough")
    assert_ge("session/subscribe.replay.events.length", len(events), 1)
    assert_present("session/subscribe.replay.replayedThrough", replayed_through)

    # Close stdin so the subprocess drains and exits.
    try:
        proc.stdin.close()
    except Exception:
        pass
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()

    print("")
    print("================================================================")
    passed = sum(1 for r in results if r)
    failed = sum(1 for r in results if not r)
    print(f"SUMMARY: pass={passed} fail={failed}")
    print("================================================================")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
