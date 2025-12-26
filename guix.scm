(use-modules (guix gexp)
             (guix packages)
             (guix import crate)
             (guix git-download)
             ((guix licenses)
              #:prefix license:)
             (guix build-system cargo)
             (gnu packages)
             (gnu packages tls)
             (gnu packages pkg-config))

(define %source-dir
  (dirname (current-filename)))

(define-public memo
  (package
    (name "memo")
    (version (or (getenv "GUIX_PACKAGE_VERSION") "dev"))
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
