# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.0.1 - 2026-08-26

### Added

- Initial release: ingest a Hugging Face Hub repository's metadata, authors,
  and commit history (models, datasets, spaces) as RDF/JSON-LD.
- `asimov-huggingface-fetcher` binary with `jsonld` and `cli` output formats.
- Optional authentication via the `ASIMOV_HUGGINGFACE_TOKEN` environment
  variable for gated/private repositories.
