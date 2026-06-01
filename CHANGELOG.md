# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-06-01

### Added

- `--pretty` flag for `resolve` and `platforms` commands (formatted JSON output)
- `--version` flag to print CLI version
- Docs site: per-platform install blocks with individual copy buttons
- Docs site: API Key section reordered before Install section
- Docs site: Download CTA scrolls to Platforms table; GitHub button restored
- Windows usage examples now use `.\bright-sdk-downloader.exe` with `.exe` suffix

## [1.0.0] - 2026-05-01

### Added

- Initial Rust implementation of BrightSDK download CLI + shared library
- CLI commands: `resolve`, `fetch`, `platforms`
- C FFI exports: `sdk_resolve`, `sdk_fetch`, `sdk_list_platforms`, `sdk_last_error`, `sdk_free_string`
- SHA-256 integrity verification of downloaded archives
- Zip Slip protection for both ZIP and tar.gz extraction
- Unix permission preservation during extraction
- Cross-platform builds: Linux x64, macOS x64/ARM64, Windows x64
- GitHub Actions CI: lint, test, build & release workflows

## [Unreleased]
