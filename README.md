This crate will run clippy on all crate archives in the cargo cache under ~/.cargo/registry/cache/github.com-1ecc6299db9ec823

Crashing crates will be copied to /tmp/clippy_crashes

Crates will be extracted to /tmp/clippy_workdir

Cargo target dir will be set to ~/.clippy_fuzzy_target_dir/ and clearned every N builds to prevent having to rebuilt every dependency on different crate versions.
