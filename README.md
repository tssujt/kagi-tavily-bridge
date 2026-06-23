# kagi-tavily-bridge

Small Rust HTTP service that exposes Tavily-shaped `/search` and `/extract`
endpoints backed by Kagi's
[OpenAPI Rust client](https://github.com/kagisearch/kagi-openapi-rust).

The compatibility target is intentionally narrow: it returns the fields that
Hermes Agent and OpenClaw parse from Tavily responses. It is not a full Tavily
API replacement.

## Compatibility

Tavily API reference: <https://docs.tavily.com/api-reference/endpoint/search>
and <https://docs.tavily.com/api-reference/endpoint/extract>.

Supported endpoints:

- `GET /healthz`
- `POST /search`
- `POST /extract`

Hermes/OpenClaw-facing response fields:

- Search: `results[].title`, `results[].url`, `results[].content`
- Extract: `results[].url`, `results[].title`, `results[].content`,
  `results[].raw_content`, `failed_results`, `failed_urls`

Authentication accepts the official Tavily-style `Authorization: Bearer ...`
header, with JSON body `api_key` as a compatibility fallback for Hermes' current
provider implementation. Other Tavily request fields such as `search_depth`,
`topic`, `include_answer`, `include_images`, `extract_depth`, and `query` are
parsed for compatibility. Unsupported Tavily-only behavior is ignored or
degraded because Kagi does not expose the same semantics.

## Configuration

The Tavily-compatible bearer token or `api_key` request field is required and
is used as the Kagi API key for upstream calls. Kagi's base URL is fixed to the
SDK default from
[`kagi-openapi-rust`](https://github.com/kagisearch/kagi-openapi-rust).

```bash
export BIND_ADDR='127.0.0.1:8080'
cargo run
```

## Docker

```bash
docker build -t kagi-tavily-bridge .
docker run --rm -p 8080:8080 kagi-tavily-bridge
```

The Docker image defaults `BIND_ADDR` to `0.0.0.0:8080` so port publishing works
without extra environment variables.

## Hermes Agent

Hermes' Tavily provider supports a custom base URL. Point it at this service:

```bash
export TAVILY_API_KEY='kg_...'
export TAVILY_BASE_URL='http://127.0.0.1:8080'
```

## OpenClaw

Configure Tavily's `baseUrl` to this service:

```json
{
  "baseUrl": "http://127.0.0.1:8080",
  "apiKey": "kg_..."
}
```

## Examples

```bash
curl -s http://127.0.0.1:8080/search \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer kg_...' \
  -d '{"query":"rust axum","max_results":5}'
```

```bash
curl -s http://127.0.0.1:8080/extract \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer kg_...' \
  -d '{"urls":["https://example.com"]}'
```

Kagi extract accepts at most 10 URLs per request, so this bridge enforces the
same limit.

## Development

The Kagi Rust client is pulled directly from GitHub and locked to a commit in
`Cargo.toml` for reproducible builds.

```bash
prek run --all-files
cargo test --locked
docker build -t kagi-tavily-bridge .
```

## License

MIT. See [LICENSE](LICENSE).
