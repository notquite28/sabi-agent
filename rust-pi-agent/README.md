# Rust Pi Agent

This crate is a small Rust learning port of Pi's core agent harness.

The original TypeScript implementation remains in `../pi/` for reference. This crate does not modify or depend on that code at runtime.

See:
- `../ROADMAP.md`
- `../docs/ARCHITECTURE.md`
- `../docs/PORTING_NOTES.md`

## Current State

The crate currently supports an OpenAI-compatible agent loop with basic tools:

- `read`
- `write`
- `bash`

## Configuration

Required. For AveMujicaAPI, paste your AveMujicaAPI token here; their docs use the OpenAI-compatible `Authorization: Bearer ...` format:

```bash
export OPENAI_API_KEY=...
```

For local development, you can also edit `.env`:

```dotenv
OPENAI_API_KEY=...
RUST_PI_MODEL=gpt-5.5
RUST_PI_BASE_URL=https://api.avemujica.moe/v1
```

AveMujicaAPI docs:

- Base URL: `https://api.avemujica.moe/v1`
- Model examples: `gpt-5.5`, or any exact model ID available to your API key
- Model list endpoint: `https://api.avemujica.moe/v1/models`

Optional:

```bash
export RUST_PI_MODEL=gpt-4.1-mini
export RUST_PI_BASE_URL=https://api.openai.com/v1
```

## Run

```bash
cargo run -- --help
cargo run -- --check-provider
cargo run -- "Say exactly: ok"
cargo run -- "Read README.md and summarize it"
cargo run
```

`--check-provider` verifies:

- `RUST_PI_BASE_URL/v1/models` is reachable through the configured base URL.
- `RUST_PI_MODEL` exists in the returned model list.
- `RUST_PI_BASE_URL/v1/chat/completions` accepts a minimal request for the selected model.
