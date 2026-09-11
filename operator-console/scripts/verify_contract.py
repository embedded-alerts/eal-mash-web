#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

cargo = (ROOT / "Cargo.toml").read_text()
main = (ROOT / "src/main.rs").read_text()
api = (ROOT / "src/api.rs").read_text()
views = (ROOT / "src/views.rs").read_text()
config = (ROOT / "src/config.rs").read_text()

required = {
    "canonical source route": "/v1/sources",
    "semantic text search": "/v1/semantic/search",
    "candidate-only evaluation": "/v1/semantic/evaluate",
    "tenant boundary": "x-eal-tenant-id",
    "redirect policy": "Policy::none()",
    "proxy boundary": ".no_proxy()",
    "bounded response": "response_limit_bytes",
}
for name, needle in required.items():
    if needle not in api:
        raise SystemExit(f"missing {name}: {needle}")

for forbidden in ["/scan", "/crawl", "/send", "next.js", "nextjs"]:
    if forbidden in main.lower() or forbidden in api.lower():
        raise SystemExit(f"forbidden operator route or stack marker: {forbidden}")

if "production startup blocked" not in config:
    raise SystemExit("production must fail closed until Shared Auth is certified")
if "crawler_controls_exposed: false" not in main:
    raise SystemExit("crawler controls must remain disabled")
if "notification_delivery_exposed: false" not in main:
    raise SystemExit("notification delivery must remain disabled")
if "htmx.org@2.0.4" not in views:
    raise SystemExit("HTMX must be pinned")
if "edition = \"2024\"" not in cargo:
    raise SystemExit("Rust 2024 edition is required")

print("operator console contract verified")
