# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2019-10-30
### Added
- `Bridge::solve_with_cache` alternative that can use remote caching.

### Changed
- Frontend can accept custom options that implement `serde::DeserializeOwned`.

## [0.2.1] - 2019-10-19
### Added
- `Options::iter` method to get a list for values.

## [0.2.0] - 2019-10-06
### Added
- Example frontends and integration testing.

### Changed
- Define `Frontend` trait with `async_trait` proc-macro.

## [0.1.0] - 2019-09-30
Initial release.
