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
