# Wave: App Server protocol schema generation gate — 2026-07-20

## Evidence

```text
cargo run -p xai-grok-app-server-protocol --example generate-schema -- --check
Finished successfully; generate-schema --check exited 0
```

The generated protocol artifacts are clean and no source regeneration diff was
required. This closes AS109-07. It does not imply that product transports or
generated SDKs have complete behavioral conformance; those remain separate
gates.
