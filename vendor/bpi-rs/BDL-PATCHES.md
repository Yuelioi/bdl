# BDL HTTP observation patch

## Course response compatibility (BDL #5 / bpi-rs #15)

Synced targeted model fixes from the sibling SDK: nullable course coupons,
omitted discount descriptions and `accept_*` fields, plus optional DRM/HLS
metadata. Regression fixtures live in `crates/bdl-core/tests/fixtures/cheese`;
SDK model tests are maintained upstream. DRM metadata describes encryption;
BDL rejects encrypted streams before creating download resources.

## HTTP observation

Source: crates.io bpi-rs 0.3.0, upstream revision in `.cargo_vcs_info.json`.
The original MIT license and source are retained. The normalized Cargo manifest
is copied from that release, with unshipped example/contract-test target declarations
removed. This directory is excluded from workspace membership.

Local changes add an optional task-local request observer around the central
transport, and route the WBI navigation request through it. BDL's video metadata
request also uses this transport. Without an observer, the upstream behavior is
preserved. The observer holds a permit until the response has been inspected;
cancellation drops the permit. Redirects are disabled on the CLI's HTTP client.
The opt-in observer can enable one retry for GET connection/timeouts or 502/503/504,
after two seconds; each attempt obtains a new permit. API restrictions never retry.
The local minimum Rust version is 1.88, matching BDL.

This enables real HTTP request budgets and cross-process throttling for the CLI's
resolver calls (including WBI cache misses), without a TLS proxy, certificate
replacement, connection-count approximation, or modification of Cargo's registry.
It is not a blanket interceptor for every optional upstream module. CLI-used
resolver paths and their raw sends are covered by focused tests and source checks.

When updating bpi-rs, reapply/review these small changes against upstream and audit
new raw `.send()` paths; prefer removing the patch when upstream provides a hook.
