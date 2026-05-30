# bright-sdk-downloader-rs

> BrightSDK download CLI + shared library (Rust) — resolve versions, fetch and extract SDK archives.

[![Lint](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/lint.yml/badge.svg)](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/lint.yml)
[![Test](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/test.yml/badge.svg)](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/test.yml)
[![E2E](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/e2e.yml/badge.svg)](https://github.com/BrightSDK/bright-sdk-downloader-rs/actions/workflows/e2e.yml)
[![Release](https://img.shields.io/github/v/release/BrightSDK/bright-sdk-downloader-rs)](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest)

## Demo

![CLI usage demo](demo/usage.gif)

## What it does

Single source of truth for downloading BrightSDK archives across all integration tools (CLI, Gradle plugin, Unity plugin). Ships as:

- **Standalone CLI binary** (~1.5 MB) — drop-in replacement for the Node.js version
- **Shared library** (`.dll` / `.so` / `.dylib`, ~1.7 MB) — for direct FFI from C#, Java, etc.

### Features

- Version resolution via BrightSDK API
- Archive download from CDN (HTTPS + redirect handling)
- Extraction with Zip Slip protection and Unix permission preservation
- SHA-256 integrity verification
- Zero runtime dependencies — fully statically linked

## Platforms

| Platform | CLI binary | Shared library |
|----------|-----------|----------------|
| Linux x64 | [bright-sdk-downloader-linux-x64](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/bright-sdk-downloader-linux-x64) | [libbright_sdk_download.so](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/libbright_sdk_download.so) |
| macOS x64 | [bright-sdk-downloader-macos-x64](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/bright-sdk-downloader-macos-x64) | [libbright_sdk_download.dylib](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/libbright_sdk_download.dylib) |
| macOS ARM64 | [bright-sdk-downloader-macos-arm64](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/bright-sdk-downloader-macos-arm64) | [libbright_sdk_download.dylib](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/libbright_sdk_download.dylib) |
| Windows x64 | [bright-sdk-downloader-win-x64.exe](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/bright-sdk-downloader-win-x64.exe) | [bright_sdk_download.dll](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest/download/bright_sdk_download.dll) |

Download all from [Releases](https://github.com/BrightSDK/bright-sdk-downloader-rs/releases/latest).

## API Key

All commands require an `SDK_API_KEY` environment variable.

1. Open [BrightSDK API Keys](https://bright-sdk.com/cp/settings/company_profile#api_keys) (links directly to the section)
2. Click **+ Add**, set an expiration, give it a name, click **Generate key**
3. Copy the key immediately (it is shown only once)
4. Export it in your shell:

```bash
export SDK_API_KEY=<your-api-key>
```

See the full [step-by-step guide with screenshots](https://brightsdk.github.io/bright-sdk-downloader-rs/obtain-api-key.html).

## CLI Usage

```bash
export SDK_API_KEY=<your-api-key>

# Resolve latest version + download URL
bright-sdk-downloader resolve -p android

# Download and extract SDK archive
bright-sdk-downloader fetch -p tizen -o ./libs

# List all available platforms
bright-sdk-downloader platforms
```

### Commands

| Command | Description | Output |
|---------|-------------|--------|
| `resolve` | Resolve version + download URL | `{platform, version, url}` |
| `fetch` | Download and extract archive | `{platform, version, url, output}` |
| `platforms` | List available platform keys | `[{key, last_version}]` |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --platform` | Platform key (`android`, `ios`, `tizen`, `webos`, `node`, `win`, `macos`, `unity`) | required |
| `-v, --version` | SDK version or `latest` | `latest` |
| `-o, --output` | Output directory (fetch only) | `.` |

### Environment

| Variable | Required | Description |
|----------|----------|-------------|
| `SDK_API_KEY` | Yes | BrightSDK API key for authentication |

## Shared Library (FFI)

The library exposes a C ABI for use from any language:

```c
// Returns JSON string (caller must free with sdk_free_string), or NULL on error
char* sdk_resolve(const char* platform, const char* version);
char* sdk_fetch(const char* platform, const char* version, const char* output_dir);
char* sdk_list_platforms(void);

// Error handling
char* sdk_last_error(void);

// Memory management
void sdk_free_string(char* ptr);
```

### C# / Unity example

```csharp
using System.Runtime.InteropServices;

[DllImport("bright_sdk_download", CallingConvention = CallingConvention.Cdecl)]
static extern IntPtr sdk_fetch(string platform, string version, string outputDir);

[DllImport("bright_sdk_download", CallingConvention = CallingConvention.Cdecl)]
static extern IntPtr sdk_last_error();

[DllImport("bright_sdk_download", CallingConvention = CallingConvention.Cdecl)]
static extern void sdk_free_string(IntPtr ptr);
```

### Java (JNA) example

```java
public interface BrightSdkDownload extends Library {
    BrightSdkDownload INSTANCE = Native.load("bright_sdk_download", BrightSdkDownload.class);
    String sdk_resolve(String platform, String version);
    String sdk_fetch(String platform, String version, String outputDir);
    void sdk_free_string(Pointer ptr);
}
```

## Building from source

```bash
# Prerequisites: Rust toolchain (https://rustup.rs)
cargo build --release

# Run tests
cargo test

# Build for all platforms (requires Docker for cross-compilation)
./build-all.sh
```

## License

MIT
