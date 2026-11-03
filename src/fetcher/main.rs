// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-huggingface-fetcher requires the 'std' feature");

use asimov_huggingface_module::{fetch_commits, fetch_repo, parse_ref, to_jsonld};
use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use std::error::Error;

/// Ingest a Hugging Face Hub repository's metadata and history as RDF.
///
/// Example: asimov-huggingface-fetcher -o jsonld meta-llama/Llama-3-8B
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Output format: jsonld (default) or cli.
    #[arg(short = 'o', long, value_name = "FORMAT", default_value = "jsonld")]
    output: String,

    /// Limit to the N most recent commits (default: all).
    #[arg(short = 'n', long, value_name = "N")]
    max: Option<usize>,

    /// A Hugging Face model/dataset/space id or URL
    /// (e.g. `google-bert/bert-base-uncased` or
    /// `https://huggingface.co/datasets/rajpurkar/squad`).
    #[arg(value_name = "REPO")]
    repo: Option<String>,
}

pub fn main() -> Result<SysexitsError, Box<dyn Error>> {
    // Load environment variables from `.env`:
    asimov_module::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = asimov_module::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    // Handle the `--version` flag:
    if options.flags.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(EX_OK);
    }

    // Handle the `--license` flag:
    if options.flags.license {
        print!("{}", include_str!("../../UNLICENSE"));
        return Ok(EX_OK);
    }

    // Configure logging & tracing:
    #[cfg(feature = "tracing")]
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    let Some(repo) = options.repo.as_deref() else {
        eprintln!("asimov-huggingface-fetcher: a REPO argument is required");
        return Ok(EX_USAGE);
    };

    let repo_ref = match parse_ref(repo) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("asimov-huggingface-fetcher: {e}");
            return Ok(EX_USAGE);
        },
    };

    let info = fetch_repo(&repo_ref)?;
    let commits = fetch_commits(&repo_ref, options.max)?;

    match options.output.as_str() {
        "jsonld" => println!("{}", to_jsonld(&repo_ref, &info, &commits)?),
        "cli" => {
            for c in &commits {
                let short = &c.id[..c.id.len().min(8)];
                let who = c
                    .authors
                    .first()
                    .map(|a| a.handle())
                    .unwrap_or_else(|| "unknown".to_string());
                println!("{short}  {}  <{}>  {}", c.date, who, c.title);
            }
        },
        other => {
            eprintln!("asimov-huggingface-fetcher: unknown --output format: {other}");
            return Ok(EX_USAGE);
        },
    }

    Ok(EX_OK)
}
