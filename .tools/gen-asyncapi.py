#!/usr/bin/env python3
"""Generate the AsyncAPI contract from the domain EVENT_CATALOG.

Usage: .tools/gen-asyncapi.py [--root DIR]
"""
import argparse
import json
import re
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()
    root = Path(args.root).resolve()
    source = (root / "crates/domain/src/event.rs").read_text()
    block = source.split("EVENT_CATALOG", 1)[1].split("];", 1)[0]
    events = re.findall(r'\("([a-z0-9_.]+)",\s*(\d+)\)', block)
    if not events:
        raise SystemExit("gen-asyncapi: no events found in EVENT_CATALOG")
    channels = {
        name: {
            "address": name,
            "messages": {
                "event": {"payload": {"$ref": "#/components/schemas/EventEnvelope"}}
            },
        }
        for name, _version in sorted(events)
    }
    for topic, parameter in (
        ("lab:{labId}", "labId"),
        ("card:{cardId}", "cardId"),
        ("run:{runId}", "runId"),
        ("system", None),
    ):
        channel = {
            "address": topic,
            "messages": {
                "event": {"payload": {"$ref": "#/components/schemas/DomainEvent"}}
            },
        }
        if parameter:
            channel["parameters"] = {
                parameter: {"description": f"{parameter} scope"}
            }
        channels[topic] = channel
    document = {
        "asyncapi": "3.0.0",
        "info": {
            "title": "AI-Lab Event Stream",
            "version": "1.0.0",
            "description": "WebSocket event envelope and core channels.",
        },
        "servers": {
            "ws": {
                "host": "127.0.0.1:8080",
                "protocol": "wss",
                "pathname": "/api/v1/ws",
            }
        },
        "channels": channels,
        "components": {
            "schemas": {
                "DomainEvent": {
                    "type": "object",
                    "required": [
                        "event_id",
                        "event",
                        "seq",
                        "lab_id",
                        "occurred_at",
                        "payload",
                    ],
                    "properties": {
                        "event_id": {"type": "string", "format": "uuid"},
                        "event": {"type": "string"},
                        "seq": {"type": "integer", "format": "int64"},
                        "lab_id": {"type": "string", "format": "uuid"},
                        "occurred_at": {"type": "string", "format": "date-time"},
                        "payload": {"type": "object"},
                    },
                },
                "EventEnvelope": {
                    "type": "object",
                    "required": [
                        "event_id",
                        "event",
                        "seq",
                        "lab_id",
                        "occurred_at",
                        "payload",
                    ],
                    "properties": {
                        "event_id": {"type": "string", "format": "uuid"},
                        "event": {"type": "string"},
                        "seq": {"type": "integer", "format": "int64"},
                        "lab_id": {"type": "string", "format": "uuid"},
                        "occurred_at": {"type": "string", "format": "date-time"},
                        "payload": {"type": "object"},
                    },
                }
            }
        },
    }
    output = root / ".docs/product/api/asyncapi.json"
    output.write_text(json.dumps(document, indent=2) + "\n")
    print(f"gen-asyncapi: wrote {output.relative_to(root)} ({len(channels)} channels)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
