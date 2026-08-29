# ew-translator

A fast popup translator for X11. Select text in any application, press a global shortcut, and get the translation next to the pointer.

Google Translate remains the zero-configuration default. If its unofficial endpoint rate-limits you, ew-translator can use Google Gemini, OpenRouter, OpenAI, or any server that implements the OpenAI Chat Completions API.

Provider selection is explicit: ew-translator does not automatically fall back to another source after an error. For example, an OpenRouter model ID such as `google/gemini-2.5-flash-lite` still uses OpenRouter; the `google/` prefix identifies the model provider and does not select Google Translate.

![Usage preview](video.gif)

## Dependencies

`xsel` must be available in `PATH`.

## Install a prebuilt binary

GitHub releases provide an UPX-compressed Linux x86_64 binary and its SHA-256 checksum. Download both files into the same directory, then verify and install them:

```sh
sha256sum --check ew-translator.sha256
install -m 755 ew-translator ~/.local/bin/ew-translator
```

## Quick start

The default target language is Russian and the default shortcut is `Ctrl+Shift+F7`:

```sh
ew-translator
```

Change them with options or environment variables:

```sh
ew-translator --lang fr --hotkeys 'CTRL+SHIFT+F8'

EW_TRANSLATOR_LANG=fr \
EW_TRANSLATOR_HOTKEYS='CTRL+SHIFT+F8' \
ew-translator
```

## Translation sources

### Google Translate (default)

No credentials are needed:

```sh
EW_TRANSLATOR_SOURCE=google-translate ew-translator
```

This source uses an undocumented Google Translate endpoint and can return HTTP 429. Use one of the authenticated providers below when that happens.

### Google Gemini

```sh
EW_TRANSLATOR_SOURCE=gemini \
EW_TRANSLATOR_API_KEY='<gemini-api-key>' \
ew-translator
```

The default Gemini model is `gemini-2.5-flash-lite`. Override it when needed:

```sh
EW_TRANSLATOR_SOURCE=gemini \
EW_TRANSLATOR_API_KEY='<gemini-api-key>' \
EW_TRANSLATOR_MODEL='<gemini-model-id>' \
ew-translator
```

### OpenRouter

OpenRouter requires an explicit model ID:

```sh
EW_TRANSLATOR_SOURCE=openrouter \
EW_TRANSLATOR_API_KEY='<openrouter-api-key>' \
EW_TRANSLATOR_MODEL='<provider/model>' \
ew-translator
```

### OpenAI

```sh
EW_TRANSLATOR_SOURCE=openai \
EW_TRANSLATOR_API_KEY='<openai-api-key>' \
EW_TRANSLATOR_MODEL='<model-id>' \
ew-translator
```

### OpenAI-compatible server

Set the base URL ending at the API version. ew-translator appends `/chat/completions` unless it is already present:

```sh
EW_TRANSLATOR_SOURCE=openai \
EW_TRANSLATOR_API_KEY='<api-key-or-dummy-value>' \
EW_TRANSLATOR_MODEL='<server-model-id>' \
EW_TRANSLATOR_API_URL='http://127.0.0.1:1234/v1' \
ew-translator
```

Plain HTTP is accepted for local servers. Use HTTPS for remote services.

## Environment variables

| Variable | Meaning |
| --- | --- |
| `EW_TRANSLATOR_SOURCE` | `google-translate`, `gemini`, `openrouter`, or `openai` |
| `EW_TRANSLATOR_API_KEY` | API key for an authenticated provider |
| `EW_TRANSLATOR_MODEL` | Model ID; required for OpenRouter and OpenAI-compatible sources |
| `EW_TRANSLATOR_API_URL` | Provider base URL |
| `EW_TRANSLATOR_LANG` | Target language code or name |
| `EW_TRANSLATOR_HOTKEYS` | Global shortcut |
| `EW_TRANSLATOR_CONFIG` | Path to an optional TOML config |

The API key deliberately has no command-line option, keeping it out of shell history and process listings.

## Optional config

No config is required or generated. If present, the default path is:

- `$XDG_CONFIG_HOME/ew-translator/config.toml`; or
- `~/.config/ew-translator/config.toml` when `XDG_CONFIG_HOME` is unset.

Use another file with `--config <path>` or `EW_TRANSLATOR_CONFIG=<path>`. An explicitly selected missing or invalid file is an error.

```toml
source = "openrouter"
api_key = "<openrouter-api-key>"
model = "<provider/model>"
api_url = "https://openrouter.ai/api/v1"
lang = "ru"
hotkeys = "CTRL+SHIFT+F7"
```

If the config contains `api_key`, protect it with `chmod 600`. Environment variables are preferable for secrets.

Settings are resolved in this order:

1. command-line options;
2. environment variables;
3. optional TOML config;
4. built-in defaults.

## Command-line help

Run `ew-translator --help` for the complete and current list. Available options include `--source`, `--model`, `--api-url`, `--config`, `--lang`, and `--hotkeys`.
