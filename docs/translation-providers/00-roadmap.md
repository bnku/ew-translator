# Translation providers roadmap

## Goal

Keep the zero-configuration Google Translate flow while adding authenticated LLM translation through Google Gemini, OpenRouter, OpenAI, and OpenAI-compatible APIs.

## Contract

- Sources: `google-translate` (default), `gemini`, `openrouter`, `openai`.
- Precedence: command-line options, environment variables, optional TOML config, built-in defaults.
- API keys are accepted only through `EW_TRANSLATOR_API_KEY` or the optional config, never through a command-line flag.
- Default config: `$XDG_CONFIG_HOME/ew-translator/config.toml`, falling back to `~/.config/ew-translator/config.toml`.
- Explicit config path: `--config` or `EW_TRANSLATOR_CONFIG`.
- Provider base URL: `EW_TRANSLATOR_API_URL` or `--api-url`.
- Google Translate remains undocumented and rate-limited; its failures must not expose HTML response bodies.

## Architecture

`settings` resolves and validates all inputs once at startup. `translator` owns one reusable HTTP client and dispatches to provider-specific request/response adapters. The global shortcut shows a loading popup immediately and performs network work on a background thread so LLM latency does not freeze the Tauri event loop.

## Delivery order

1. Typed settings and provider contract.
2. Provider request/response implementations with parser tests.
3. Non-blocking runtime integration and stale-response protection.
4. CLI help, README examples, config reference, and complete build/test verification.

## Decisions

- Gemini defaults to stable `gemini-2.5-flash-lite`; every model remains overridable.
- OpenRouter and OpenAI-compatible sources require an explicit model.
- OpenAI-compatible mode uses Chat Completions because it is the common denominator across third-party servers.
- Plain HTTP API URLs are accepted for local servers; remote services should use HTTPS.

## Status

Complete. Verification: 16 Rust tests, Clippy with warnings denied, Svelte type checking, frontend production build, Rust release build, and a headless `--help` smoke check.
