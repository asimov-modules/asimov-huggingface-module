# ASIMOV Hugging Face Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Package on Crates.io](https://img.shields.io/crates/v/asimov-huggingface-module)](https://crates.io/crates/asimov-huggingface-module)
[![Documentation](https://docs.rs/asimov-huggingface-module/badge.svg)](https://docs.rs/asimov-huggingface-module)

[ASIMOV] module for Hugging Face Hub repositories. Ingests a model's,
dataset's, or space's metadata, authors, and commit history as [JSON-LD] /
[RDF].

## ✨ Features

- Fetches repository metadata from the Hugging Face Hub API (no local clone).
- Accepts either a bare id (`org/name`, `name`) or a full `huggingface.co` /
  `hf.co` URL, auto-detecting models, datasets, and spaces.
- Emits one `Repository` node (downloads, likes, license, pipeline tag,
  library, tags, files) plus one `git:Commit` per commit — chained by
  `parent` — and one `schema:Person` per distinct author handle.
- Loads environment variables from `.env` (aka dotenv) files.
- Distributed as a standalone static binary with zero runtime dependencies.

## 🛠️ Prerequisites

- [Rust] 1.85+ (2024 edition) if building from source code.
- Optionally, an `ASIMOV_HUGGINGFACE_TOKEN` environment variable for
  gated/private repositories.

## ⬇️ Installation

### Installation with the [ASIMOV CLI]

```bash
asimov module install huggingface -v
```

### Installation from Source Code

```bash
cargo install asimov-huggingface-module
```

## 👉 Examples

```bash
asimov-huggingface-fetcher -o jsonld meta-llama/Llama-3-8B
asimov-huggingface-fetcher -o jsonld https://huggingface.co/google-bert/bert-base-uncased
asimov-huggingface-fetcher -o cli -n 5 https://huggingface.co/datasets/rajpurkar/squad
```

## ⚙ Configuration

This module requires no configuration. To access gated or private
repositories, supply a Hugging Face access token via the
`ASIMOV_HUGGINGFACE_TOKEN` environment variable (the standard `HF_TOKEN` and
`HUGGING_FACE_HUB_TOKEN` variables are also honored).

## 📚 Reference

### Installed Binaries

- `asimov-huggingface-fetcher`

### `asimov-huggingface-fetcher`

```
Ingest a Hugging Face Hub repository's metadata and history as RDF.

Usage: asimov-huggingface-fetcher [OPTIONS] <REPO>

Arguments:
  <REPO>  A Hugging Face model/dataset/space id or URL

Options:
  -o, --output <FORMAT>  Output format: jsonld (default) or cli [default: jsonld]
  -n, --max <N>          Limit to the N most recent commits (default: all)
  -d, --debug            Enable debugging output
      --license          Show license information
  -v, --verbose...       Enable verbose output (may be repeated for more verbosity)
  -V, --version          Print version information
  -h, --help             Print help
```

## 👨‍💻 Development

```bash
git clone https://github.com/asimov-modules/asimov-huggingface-module.git
```

### Vocabulary

This module emits `schema:` terms for the repository/person facets the Hub
exposes natively, alongside the `git:` vocabulary (`Commit`, `authoredBy`,
`parent`, `authorDate`, `message`) shared with the [ASIMOV Git module], plus
an inline `hf:` namespace for Hub-specific counters (`downloads`, `likes`,
`pipelineTag`, `library`, `file`).

---

[ASIMOV]: https://asimov.sh
[ASIMOV CLI]: https://cli.asimov.sh
[ASIMOV Git module]: https://github.com/asimov-modules/asimov-git-module
[JSON-LD]: https://json-ld.org
[RDF]: https://www.w3.org/TR/rdf12-primer/
[Rust]: https://rust-lang.org
