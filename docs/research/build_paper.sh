#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RESEARCH_DIR="$ROOT/docs/research"
FIGURE_DIR="$RESEARCH_DIR/figures"
FIGURE_BUILD_DIR="$FIGURE_DIR/build"
OUTPUT_DIR="$ROOT/output/pdf"

mkdir -p "$FIGURE_BUILD_DIR" "$OUTPUT_DIR"

cd "$FIGURE_DIR"
tectonic --outdir "$FIGURE_BUILD_DIR" pipeline-figure.tex

cd "$RESEARCH_DIR"
tectonic --outdir "$OUTPUT_DIR" tim-paper.tex
tectonic --outdir "$OUTPUT_DIR" tim-paper-zh.tex
