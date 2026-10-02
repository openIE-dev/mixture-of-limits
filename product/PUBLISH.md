# Publish path — openIE-dev/mixture-of-limits

Clean-room Rust reference + `product/` mirror. Named in Cargo.toml; create when ready.

## Create (David / openIE-dev auth)

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
# if not yet a git repo:
git init
git add -A
git commit -m "Mixture of Limits clean-room + product A1–A6 surface"

gh repo create openIE-dev/mixture-of-limits \
  --source=. --public \
  --description "Mixture of Limits — navigation floors, NI certify-before-commit, satiation stop (Apache-2.0 OR MIT)" \
  --push
```

Active `gh` account for org: `dcharlot65-openie` (scopes include `repo`).

## Product embodiment package (docs)

```bash
cd /Users/dcharlot/data-share/vibe-coding/mixture-of-limits-product
# already git; optional remote:
# gh repo create openIE-dev/mixture-of-limits-product --source=. --public --push
```

Prefer **one** canonical code repo (`mixture-of-limits`) with `product/` inside; keep vibe-coding package as drafting mirror if needed.

## Verify

```bash
cargo test --workspace
cargo run -p mol-cli -- prove
cargo run -p mol-cli -- run --chore product/mol.yaml
```
