# Research Workspace

This directory is the minimal paper workspace for TIM, with the English paper now anchored to a NeurIPS-style LaTeX source.

## Purpose

- Keep paper-facing artifacts under version control.
- Tie claims to the implemented architecture rather than to speculative extensions.
- Separate research assets from user-facing API/reference documentation.

## Current assets

- [Paper Outline](paper-outline.md)
- Canonical English paper source: `tim-paper.tex`
- Canonical Chinese paper source: `tim-paper-zh.tex`
- Canonical English paper template shape: `main.tex`-style NeurIPS preprint layout
- Canonical English paper style: `neurips_2021.sty`
- Local algorithm package: `algorithm.sty`
- Shared math definitions: `defs.tex`
- Working prose draft: [tim-paper.md](tim-paper.md)
- Chinese companion draft: [tim-paper.zh.md](tim-paper.zh.md)
- [System Diagram](../design/system-diagram.md)
- [Architecture Reference](../design/architecture.md)
- Final PDF artifact: `output/pdf/tim-paper.pdf`
- Build command: `docs/research/build_paper.sh`

## Writing discipline

- Treat the schema-driven extension model as the core claim.
- Keep terminology aligned with the codebase: schema, template, parser, validator, dispatcher, executor.
- Mark any forward-looking claims as future work until backed by code or measurements.

## Suggested next artifacts

- A short related-work note on agent tool schemas, execution gateways, and intent systems.
- A reproducible evaluation note with the exact experiments used in the paper.
- Figure exports derived from the Mermaid system diagram.
