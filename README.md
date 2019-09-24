BuildKit binding for Rust
=======

[![Actions Status]][Actions Link]

# Project structure

This repository contains three important building blocks to implement BuildKit frontends in Rust:

* [![buildkit-llb Crates Badge]][buildkit-llb Crates Link]
  [![buildkit-llb Docs Badge]][buildkit-llb Docs Link]
  [`buildkit-llb`](buildkit-llb/README.md) - high-level API to create BuildKit LLB graphs,

* [![buildkit-frontend Crates Badge]][buildkit-frontend Crates Link]
  [![buildkit-frontend Docs Badge]][buildkit-frontend Docs Link]
  [`buildkit-frontend`](buildkit-frontend/README.md) - foundation and utilities for BuildKit frontends,

* [![buildkit-proto Crates Badge]][buildkit-proto Crates Link]
  [![buildkit-proto Docs Badge]][buildkit-proto Docs Link]
  [`buildkit-proto`](buildkit-proto/README.md) - low-level protobuf interfaces to BuildKit.

[Actions Link]: https://github.com/denzp/rust-buildkit/actions
[Actions Status]: https://github.com/denzp/rust-buildkit/workflows/CI/badge.svg
[buildkit-llb Docs Badge]: https://docs.rs/buildkit-llb/badge.svg
[buildkit-llb Docs Link]: https://docs.rs/buildkit-llb/
[buildkit-llb Crates Badge]: https://img.shields.io/crates/v/buildkit-llb.svg
[buildkit-llb Crates Link]: https://crates.io/crates/buildkit-llb
[buildkit-frontend Docs Badge]: https://docs.rs/buildkit-frontend/badge.svg
[buildkit-frontend Docs Link]: https://docs.rs/buildkit-frontend/
[buildkit-frontend Crates Badge]: https://img.shields.io/crates/v/buildkit-frontend.svg
[buildkit-frontend Crates Link]: https://crates.io/crates/buildkit-frontend
[buildkit-proto Docs Badge]: https://docs.rs/buildkit-proto/badge.svg
[buildkit-proto Docs Link]: https://docs.rs/buildkit-proto/
[buildkit-proto Crates Badge]: https://img.shields.io/crates/v/buildkit-proto.svg
[buildkit-proto Crates Link]: https://crates.io/crates/buildkit-proto
