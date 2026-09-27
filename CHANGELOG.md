# v1.2.0 (2026-09-27)

### Highlights

* **Pure Rust GUI:** Replaced Tauri and WebKitGTK with a lightweight, high-performance pure Rust UI powered by `eframe`/`egui`.
* **Zero External Dependencies:** Removed `xsel` and WebKit system libraries; selection handling is now fully native via `arboard`.
* **Smart Adaptive Window:** The popup tightly wraps around content (spinner, single words, or paragraphs) without extra empty margins or fixed dimensions.
* **Scrollbar for Long Text:** Automatic vertical scrollbar and smooth scrolling when translations exceed maximum height.
* **Intelligent Screen & Multi-Monitor Awareness:** Multi-monitor boundary detection via Xinerama. The popup never overflows off-screen and automatically flips upwards when near the bottom edge.
* **Interactive Text:** Translated text can be selected and copied (`Ctrl+C`); clicking outside or pressing `Esc` instantly dismisses the popup without interrupting in-window interactions.
* **Multi-Platform CI/CD:** Added automated GitHub Actions build and release workflows for Linux, Windows, and macOS.

# v1.1.0 (2026-08-29)

### Additions

* add Google Gemini, OpenRouter, OpenAI, and OpenAI-compatible translation sources
* configure providers through command-line options, environment variables, or an optional TOML file
* document all environment variables in the binary's `--help` output

### Changes

* keep Google Translate as the zero-configuration default while reporting a concise, actionable error when it returns HTTP 429
* run translation requests outside the Tauri event loop and ignore stale responses from older shortcut invocations
* validate provider credentials, models, and API URLs at startup without logging API keys
* print the resolved provider, model, target language, shortcut, and API URL at startup for easier diagnostics

### Fixes

* report missing `xsel`, invalid selection text, provider HTTP failures, and malformed API responses in the popup

### Maintenance

* add provider parser, configuration precedence, request-shape, and loopback HTTP tests
* document provider selection, configuration files, credential handling, and release installation


# v1.0.1 (2026-08-28)

### Fixes

* restore Google Translate requests with a modern HTTP/2 and TLS client
* report HTTP and response parsing failures instead of showing an empty popup
* make the translation popup receive focus reliably on X11 and hide on the first outside click
* render translated text safely instead of interpreting it as HTML
* update Tauri 1.x dependencies for compatibility with current Rust and WebKitGTK

### Maintenance

* add translation response tests and a live endpoint smoke test
* add reproducible Linux x86_64 release packaging with symbol stripping, UPX compression, and a SHA-256 checksum


# v1.0.0 (2023-10-30)

### Changes

* rewrite the application UI with Tauri and Svelte
* show translations in a borderless, always-on-top popup near the pointer
* add the animated usage preview


# v0.3.0 (2023-10-28)

### Changes

* replace the global hotkey manager


# v0.2.0 (2022-03-27)

### Additions

* `clap` for parse arguments and `--help` text
* set hotkey from cli


# v0.1.0 (2022-03-26)

### MVP
