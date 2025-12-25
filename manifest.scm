(use-modules (guix packages)
             (gnu)
             (gnu packages rust)
             (gnu packages pkg-config))

(packages->manifest (list pkg-config
                          rust
                          (list rust "cargo")
                          (list rust "tools")
                          rust-analyzer))
