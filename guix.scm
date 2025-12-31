(use-modules (guix gexp)
             (guix packages)
             (guix import crate)
             (guix git-download)
             ((guix licenses)
              #:prefix license:)
             (guix build-system cargo)
             (gnu packages)
             (gnu packages pkg-config)
             (ice-9 popen)
             (ice-9 rdelim))

(define %source-dir
  (dirname (current-filename)))

(define (git-version)
  "Get version from git describe, or fallback to env var or default"
  (or (getenv "GUIX_PACKAGE_VERSION")
      (let* ((pipe (open-pipe* OPEN_READ "git" "describe" "--tags" "--always" "--dirty" "--match" "v*.*.*"))
             (version (read-line pipe))
             (status (close-pipe pipe)))
        (if (and (zero? status) (not (eof-object? version)))
            version
            "0.0.0-placeholder"))))

(define-public memo
  (package
    (name "memo")
    (version (git-version))
    (source
      (local-file %source-dir
                  #:recursive? #t
                  #:select? (git-predicate %source-dir)))
    (build-system cargo-build-system)
    (native-inputs (list pkg-config))
    (inputs (cons* (cargo-inputs-from-lockfile "Cargo.lock")))
    (arguments
    `(#:install-source? #f
      #:tests? #f))
    (home-page "https://github.com/ExFed/memo")
    (synopsis "Shell command memoization tool")
    (description
     "memo is a small program that memoizes (caches) shell command executions.
Run a command through memo once and it records stdout, stderr, and exit code.
Then, when you run the same command again from the same working directory,
memo instantly replays the cached output instead of re-running the command.")
    (license #f)))

memo
