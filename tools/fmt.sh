#!/usr/bin/env bash
set -e

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
exec cargo +stable fmt --all
