# Provider implementations

Status: done

- Google Translate remains the default and its HTTP 429 page is replaced with a concise provider-specific error. Providers are selected explicitly; there is no automatic fallback.
- Gemini uses `generateContent` and `x-goog-api-key` authentication.
- OpenRouter and OpenAI-compatible sources share a Chat Completions adapter.
- Parser and loopback HTTP tests cover provider response shapes, paths, headers, bodies, and safe API errors.
