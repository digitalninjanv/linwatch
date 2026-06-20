# Contributing

Thanks for improving `linwatch`. Keep changes small, measurable, and Linux-friendly.

## Local Checks

Run these before opening a pull request:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

## Design Rules

- Prefer `/proc`, `/sys`, and safe libc calls for core metrics.
- Keep optional external commands documented and non-critical.
- Avoid daemon, database, telemetry, and background service requirements.
- Preserve terminal accessibility: severity must not rely on color alone.
- Update README and tests when adding CLI flags, config keys, or output fields.

## Release Checklist

1. Update `Cargo.toml` version.
2. Run all local checks.
3. Tag the release:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow builds Linux tarballs and publishes SHA256 checksums.
