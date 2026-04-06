#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RESEARCH_DIR="$ROOT/docs/research"
FIGURE_BUILD_DIR="$RESEARCH_DIR/figures/build"
ARXIV_DIR="$ROOT/output/arxiv"
STAGE_DIR="$ARXIV_DIR/tim-paper"
TMP_BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tim-paper-arxiv.XXXXXX")"

cleanup() {
  rm -rf "$TMP_BUILD_DIR"
}
trap cleanup EXIT

mkdir -p "$ARXIV_DIR"

"$RESEARCH_DIR/build_paper.sh"

cd "$RESEARCH_DIR"
tectonic --keep-intermediates --keep-logs --outdir "$TMP_BUILD_DIR" tim-paper.tex

if [[ ! -f "$TMP_BUILD_DIR/tim-paper.bbl" ]]; then
  echo "expected $TMP_BUILD_DIR/tim-paper.bbl to exist after compilation" >&2
  exit 1
fi

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

cp "$RESEARCH_DIR/tim-paper.tex" "$STAGE_DIR/"
cp "$RESEARCH_DIR/tim-paper.bib" "$STAGE_DIR/"
cp "$TMP_BUILD_DIR/tim-paper.bbl" "$STAGE_DIR/"
cp "$RESEARCH_DIR/defs.tex" "$STAGE_DIR/"
cp "$RESEARCH_DIR/neurips_2021.sty" "$STAGE_DIR/"
cp "$RESEARCH_DIR/algorithm.sty" "$STAGE_DIR/"
cp "$FIGURE_BUILD_DIR/pipeline-figure.pdf" "$STAGE_DIR/"

(
  cd "$STAGE_DIR"
  tar -czf ../tim-paper-arxiv-flat.tar.gz \
    tim-paper.tex \
    tim-paper.bib \
    tim-paper.bbl \
    defs.tex \
    neurips_2021.sty \
    algorithm.sty \
    pipeline-figure.pdf
)

(
  cd "$ARXIV_DIR"
  tar -czf tim-paper-arxiv-source.tar.gz tim-paper
)

echo "Wrote $ARXIV_DIR/tim-paper-arxiv-flat.tar.gz"
