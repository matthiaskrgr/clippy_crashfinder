This crate will run clippy on all crate archives in the cargo cache under ~/.cargo/registry/cache/github.com-1ecc6299db9ec823

The crate uses `rustwide` for sandboxing and clears the
cargo package cache and target dirs from time to time, to keep disk usage at bay.

Crashes will be saved and printed in after running.
