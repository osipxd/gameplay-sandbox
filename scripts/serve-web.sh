#!/bin/sh
set -eu

PORT="${1:-8000}"
python3 -m http.server "$PORT" -d web
