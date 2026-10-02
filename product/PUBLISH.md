# Publish path — openIE-dev/mixture-of-limits

Canonical clean-room + `product/` mirror is **published**:
https://github.com/openIE-dev/mixture-of-limits

## Docs mirror

Keep product embodiment docs in-repo under `product/`. Optional separate
`openIE-dev/mixture-of-limits-product` is **deferred** — prefer one canonical code repo.

Drafting mirror may also live at:
`/Users/dcharlot/data-share/vibe-coding/mixture-of-limits-product/`

## Verify before push

```bash
cargo test --workspace
cargo run -p mol-cli -- prove   # 42 VERIFIED incl. A1–A13
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- bench
```

Active `gh` account for org: `dcharlot65-openie`.
