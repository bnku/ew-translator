# Runtime integration, tests, and docs

Status: done

- One HTTP client is reused and translation runs outside the Tauri event loop.
- Request IDs prevent old LLM responses from replacing newer translations.
- `--help`, README, and changelog describe environment variables, examples, precedence, config, and credential handling.
- Rust tests, Clippy, Svelte checks, frontend build, and release build pass.
