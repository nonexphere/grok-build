# Tasks — v1-02 API key provider foundation

- [ ] `PR102-01` ApiKey transport path via `LoginCoordinator::run_api_key_login` (env/explicit secret). <!-- C0-A PARTIAL: F-05 unregistered providers accepted; backend hardcoded Ephemeral; no-op XAI fallback not wired to binding-layer rejection. Wave C5-32. -->
- [x] `PR102-02` Secret never appears in CredentialMetadata Debug.
- [x] `PR102-03` Empty/missing secret rejected.
- [x] `PR102-04` Full TTY paste UX + per-provider descriptors for openrouter/groq/cloudflare.
- [x] `PR102-05` Prohibit XAI_API_KEY fallback for native third-party bindings (binding layer).
