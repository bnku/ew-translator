# Settings and provider contract

Status: done

- CLI, environment, optional TOML config, and defaults resolve in a deterministic order.
- Sources, required keys/models, and API URLs are validated before Tauri initializes.
- Secrets stay out of command-line arguments, logs, summaries, and displayed network errors.
- Unit tests cover precedence, defaults, validation, aliases, and unknown config keys.
