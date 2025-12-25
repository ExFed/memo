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

- ~~Document sensitive output risk~~ DONE:
  - Memoizing `stdout`/`stderr` can persist secrets (tokens, passwords, private
    keys) to disk.
  - Add a prominent note to `--help` and docs.

- ~~Consider an opt-out~~ DONE:
  - Add a `MEMO_DISABLE=1` environment flag (execute without reading/writing cache).

## Robustness and concurrency

- ~~Handle stale lock files~~ SUPERSEDED by atomic directory rename:
  - Replaced lock-based concurrency with atomic directory rename strategy.
  - Each process writes to its own temp directory: `<digest>.tmp.<pid>/`
  - After completion, atomically renames temp dir to `<digest>/`
  - First rename wins; losers detect the existing directory and clean up
  - Orphaned temp directories are cleaned up on startup via `cleanup_temp_dirs()`
  - No locks needed, no stale lock problem possible.

- ~~Make lock waiting configurable~~ NO LONGER APPLICABLE:
  - Lock-based concurrency has been removed in favor of atomic directory rename.
  - All concurrent processes run to completion; first to rename wins.

- ~~Make cache writes more crash-resilient~~ DONE:
  - Implemented atomic directory rename: writes go to temp dir, then atomic rename.
  - Partial writes are isolated in temp directories and cleaned up on startup.
  - Cache entries are either complete or don't exist (no partial states visible).

## Code quality and warnings

- ~~Remove or reuse unused functions~~ DONE:
  - Cleaned up unused functions from `src/cache.rs`.
  - Removed lock-related infrastructure (no longer needed with atomic rename).

- ~~Update deprecated test helper~~ INACCURATE:
  - `assert_cmd::Command::cargo_bin` is deprecated. Switch to the recommended
    replacement (`cargo::cargo_bin_cmd!`) to avoid future breakage.
    - The function `cargo_bin` is deprecated, but the macro `cargo_bin` is not.
      The name clash is unfortunate and misleading.

## Test coverage

- ~~Add an integration test for argv collision avoidance~~ DONE:
  - Verify that `memo echo "a b"` and `memo echo a b` produce different cache
    entries (different digests).

- ~~Add a test for stale lock handling (if implemented)~~ NO LONGER APPLICABLE:
  - Lock-based concurrency has been replaced with atomic directory rename.
  - Temp directory cleanup is tested implicitly by the integration tests.
