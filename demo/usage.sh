#!/usr/bin/env bash
# Usage demo script for asciinema recording — uses mocked output, no real API calls
# Run: asciinema rec demo/usage.cast --command "bash demo/usage.sh"

_type() {
  local text="$1"
  for ((i=0; i<${#text}; i++)); do
    printf '%s' "${text:$i:1}"
    sleep 0.04
  done
  echo
}

_prompt() {
  printf '\e[1;32m❯\e[0m '
  _type "$1"
  sleep 0.5
}

_banner() {
  echo
  printf '\e[1;36m%s\e[0m\n' "$1"
  echo
  sleep 0.4
}

_out() {
  while IFS= read -r line; do
    echo "$line"
    sleep 0.08
  done <<< "$1"
  sleep 0.3
}

_json() {
  printf '\e[0;33m%s\e[0m\n' "$1"
  sleep 0.5
}

_progress() {
  local steps=("$@")
  for s in "${steps[@]}"; do
    printf '\r\e[36m%s\e[0m' "$s"
    sleep 0.6
  done
  printf '\r\e[K'
}

clear
sleep 0.5

printf '\e[1;37m'
echo "  _          _       _     _              _ _    "
echo " | |__  _ __(_) __ _| |__ | |_   ___  __| | | __"
echo " | '_ \| '__| |/ _\` | '_ \| __| / __|/ _\` | |/ /"
echo " | |_) | |  | | (_| | | | | |_  \__ \ (_| |   < "
echo " |_.__/|_|  |_|\__, |_| |_|\__| |___/\__,_|_|\_\\"
echo "               |___/                             "
printf '\e[0m'
echo "  bright-sdk-downloader (Rust) — CLI Demo"
echo
sleep 1

# Step 1: Show help
_banner "--- Show available commands ---"
_prompt "bright-sdk-downloader --help"
_out "bright-sdk-downloader — BrightSDK download CLI (Rust)

Commands:
  resolve    Resolve version + download URL (JSON)
  fetch      Download and extract SDK archive
  platforms  List available platform keys

Options:
  -p, --platform   Platform key (android, ios, tizen...)
  -v, --version    Version or \"latest\" (default: latest)
  -o, --output     Output directory (default: .)

Environment:
  SDK_API_KEY      Required. BrightSDK API key.

Examples:
  bright-sdk-downloader resolve -p android
  bright-sdk-downloader fetch -p ios -o ./libs
  bright-sdk-downloader platforms"
sleep 1

# Step 2: List platforms
_banner "--- List available platforms ---"
_prompt "bright-sdk-downloader platforms"
_json '[{"key":"android","last_version":"1.623.17"},{"key":"ios","last_version":"1.620.3"},{"key":"node","last_version":"1.616.950"},{"key":"tizen","last_version":"1.616.950"},{"key":"webos","last_version":"1.616.950"},{"key":"win","last_version":"1.616.950"},{"key":"macos","last_version":"1.616.950"},{"key":"unity","last_version":"1.616.950"}]'
sleep 1

# Step 3: Resolve android
_banner "--- Resolve latest Android SDK version ---"
_prompt "bright-sdk-downloader resolve -p android"
_json '{"platform":"android","version":"1.623.17","url":"https://cdn.example.com/sdk/android/1.623.17/brd_sdk_android.tar.gz","sha256":"a1b2c3..."}'
sleep 1

# Step 4: Resolve with specific version
_banner "--- Resolve specific version ---"
_prompt "bright-sdk-downloader resolve -p ios -v 1.620.3"
_json '{"platform":"ios","version":"1.620.3","url":"https://cdn.example.com/sdk/ios/1.620.3/brd_sdk_ios.tar.gz"}'
sleep 1

# Step 5: Fetch with progress bar
_banner "--- Fetch SDK → download + verify + extract ---"
_prompt "bright-sdk-downloader fetch -p tizen -o ./libs"
_progress \
  "████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   5% | 00:00:00 | resolve" \
  "██████████████████░░░░░░░░░░░░░░░░░░░░░░░  45% | 00:00:01 | download" \
  "██████████████████████████████████░░░░░░░░  85% | 00:00:03 | download" \
  "████████████████████████████████████░░░░░░  88% | 00:00:04 | verify" \
  "██████████████████████████████████████░░░░  95% | 00:00:04 | extract" \
  "████████████████████████████████████████░░ 100% | 00:00:05 | done"
_out "Done → ./libs (5.2s)"
_json '{"platform":"tizen","version":"1.616.950","url":"https://cdn.example.com/sdk/tizen/1.616.950/brd_sdk_tizen.tar.gz","output":"./libs"}'
sleep 0.5
_prompt "ls ./libs"
_out "brd_sdk.wgt
brd_sdk_conf.json
README.md"
sleep 1

# Step 6: Fetch for CI usage
_banner "--- Use in CI pipeline (exit code) ---"
_prompt "bright-sdk-downloader fetch -p android -o ./app/libs && echo 'OK'"
_progress \
  "████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   5% | 00:00:00 | resolve" \
  "██████████████████████████████████████░░░░  95% | 00:00:06 | extract" \
  "████████████████████████████████████████░░ 100% | 00:00:07 | done"
_out "Done → ./app/libs (7.1s)"
_json '{"platform":"android","version":"1.623.17","url":"https://cdn.example.com/sdk/android/1.623.17/brd_sdk_android.tar.gz","output":"./app/libs"}'
printf '\e[1;32mOK\e[0m\n'
sleep 1

# Step 7: Binary size comparison
_banner "--- Binary size (Rust vs Node/pkg) ---"
echo "  bright-sdk-downloader (Rust):  1.5 MB"
echo "  bright-sdk (Node/pkg):        48.0 MB"
echo
printf '  \e[1;32m→ 32× smaller\e[0m\n'
sleep 1.5

echo
printf '\e[1;32m✓ Done!\e[0m\n'
sleep 1
