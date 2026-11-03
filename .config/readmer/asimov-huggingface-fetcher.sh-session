$ asimov-huggingface-fetcher --help
Ingest a Hugging Face Hub repository's metadata and history as RDF.

Usage: asimov-huggingface-fetcher [OPTIONS] [REPO]

Arguments:
  [REPO]  A Hugging Face model/dataset/space id or URL (e.g. `google-bert/bert-base-uncased` or `https://huggingface.co/datasets/rajpurkar/squad`)

Options:
  -d, --debug            Enable debugging output
      --license          Show license information
  -v, --verbose...       Enable verbose output (may be repeated for more verbosity)
  -V, --version          Print version information
  -o, --output <FORMAT>  Output format: jsonld (default) or cli [default: jsonld]
  -n, --max <N>          Limit to the N most recent commits (default: all)
  -h, --help             Print help
