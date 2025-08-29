#!/usr/bin/env bash

set -euo pipefail

CLI_MANIFEST=$(cargo read-manifest --manifest-path Cargo.toml)
CLI_VERSION=$(echo $CLI_MANIFEST | jq -r .version)

echo $CLI_VERSION
