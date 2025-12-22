# TODO

This file tracks follow-up improvements discovered during review. Items are
prioritized roughly by impact.

## Security and correctness

- Expand the cache key beyond argv:
  - Include current working directory in the hashed input.
  - Consider including a small, explicit allowlist of environment variables
    (for example `PATH`, `LANG`, `LC_ALL`) when they materially affect command
    behavior.
  - Document the trade-off: stronger correctness vs fewer cache hits.

- Document sensitive output risk:
  - Memoizing `stdout`/`stderr` can persist secrets (tokens, passwords, private
    keys) to disk.
  - Add a prominent note to `--help` and docs.

- Consider an opt-out:
  - Add a `MEMO_DISABLE=1` environment flag (execute without reading/writing cache).

## Robustness and concurrency

- Handle stale lock files:
  - If the process crashes, `<digest>.lock` may remain and block future runs.
  - Options: lock file contains PID + timestamp; treat old locks as stale; or
    use an OS file locking primitive when available.

- Make lock waiting configurable:
  - Current wait behavior is fixed (2 seconds). Consider `--lock-timeout` or
    an environment variable.

- Make cache writes more crash-resilient:
  - For metadata and outputs, consider a temp file plus atomic rename.
  - If durability matters, consider `fsync` on file and directory.

## Code quality and warnings

- Remove or reuse unused functions:
  - There are functions in `src/cache.rs` and `src/digest.rs` that are no longer
    referenced by the binary.
  - Either delete them or wire them into the main flow to avoid dead-code
    warnings.

- Update deprecated test helper:
  - `assert_cmd::Command::cargo_bin` is deprecated. Switch to the recommended
    replacement (`cargo::cargo_bin_cmd!`) to avoid future breakage.

## Test coverage

- Add an integration test for argv collision avoidance:
  - Verify that `memo echo "a b"` and `memo echo a b` produce different cache
    entries (different digests).

- Add a test for stale lock handling (if implemented):
  - Simulate an abandoned lock and confirm the program can recover.
