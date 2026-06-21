# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] - 2026-06-22

### Added

- Unit tests for `resolve_from_releases`: cert mode URL selection, plain-URL fallback,
  `NoCertHash` error, hash override precedence, `latest` version resolution,
  unknown platform error
- Unit tests for `parse_args`: `--cert` / `-c` flag, `--hash` / `-h` + `--cert` together,
  all flags combined, default cert=false

### Changed

- Refactored `resolve_sdk_with_hash` into `resolve_from_releases` (private) so URL
  routing logic is unit-testable without a network call

## [1.1.0] - 2026-06-21

### Added

- `-c, --cert` CLI flag to opt-in to certified URL download (Windows only)
- Certified build support: uses `cert_url_tpl` when `--cert` is passed and `ver_hash` is available
- `-h, --hash` CLI flag to supply a cert build hash for downloading older certified versions
- Clear error when certified hash is unavailable for a requested version (suggests `--hash` or contacting support)
- `resolve_sdk_with_hash(platform, version, hash_override, use_cert)` public API
- `PlatformConfig` struct has new optional fields: `cert_url_tpl`, `ver_hash`, `certified`
- `Error::NotFound` for HTTP 404 responses
- `Error::NoCertHash` when `--cert` is used but no hash is available

### Changed

- `fetch_sdk_with_progress()` now accepts `hash_override` and `use_cert` parameters
- Certified URL is **opt-in** via `--cert`; default download uses plain `url_tpl`

## [1.0.2] - 2026-06-01

### Fixed

- Usage text now shows the actual executable name at runtime (e.g. `bright-sdk-downloader.exe` on Windows)
- `--version` flag handled in `main` alongside other commands

## [1.0.1] - 2026-06-01

### Added

- `--pretty` flag for `resolve` and `platforms` commands (formatted JSON output)
- `--version` flag to print CLI version
- Docs site: per-platform install blocks with individual copy buttons
- Docs site: API Key section reordered before Install section
- Docs site: Download CTA scrolls to Platforms table; GitHub button restored
- Windows usage examples now use `.\bright-sdk-downloader.exe` with `.exe` suffix

## [1.0.0] - 2026-05-30

### Added

- Initial Rust implementation of BrightSDK download CLI + shared library
- CLI commands: `resolve`, `fetch`, `platforms`
- C FFI exports: `sdk_resolve`, `sdk_fetch`, `sdk_list_platforms`, `sdk_last_error`, `sdk_free_string`
- SHA-256 integrity verification of downloaded archives
- Zip Slip protection for both ZIP and tar.gz extraction
- Unix permission preservation during extraction
- Cross-platform builds: Linux x64, macOS x64/ARM64, Windows x64
- GitHub Actions CI: lint, test, build & release workflows
