use claim_core::{
    graph::{Document, ReviewRequest},
    import, report, Bundle,
};
use clap::{Parser, Subcommand};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Parser)]
#[command(version, about = "Local evidence-first claim atlas")]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Evaluate {
        input: PathBuf,
        gold: PathBuf,
    },
    Init {
        bundle: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Import {
        export: PathBuf,
        claims: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long)]
        out: PathBuf,
    },
    Validate {
        input: PathBuf,
    },
    Analyze {
        input: PathBuf,
    },
    Review {
        input: PathBuf,
        request: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Timeline {
        input: PathBuf,
        subject: String,
    },
    Report {
        input: PathBuf,
        #[arg(long)]
        as_of: String,
        #[arg(long, default_value = "agent-harness")]
        focus: String,
        #[arg(long)]
        previous: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Diff {
        before: PathBuf,
        after: PathBuf,
    },
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn read(path: &Path) -> Result<String> {
    let mut s = String::new();
    File::open(path)?.take(4_000_001).read_to_string(&mut s)?;
    if s.len() > 4_000_000 {
        return Err("input exceeds 4 MB".into());
    }
    Ok(s)
}
fn load(path: &Path) -> Result<Document> {
    Ok(Document::parse(&read(path)?)?)
}
fn json<T: serde::Serialize>(v: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(v)?)
}
fn write(path: &Path, s: &str) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(path)?;
    f.write_all(s.as_bytes())?;
    f.sync_all()?;
    Ok(())
}
fn run() -> Result<()> {
    match Args::parse().command {
        Command::Evaluate { input, gold } => {
            let pairs =
                serde_json::from_str::<Vec<claim_core::evaluation::GoldPair>>(&read(&gold)?)?;
            println!(
                "{}",
                json(&claim_core::evaluation::evaluate(&load(&input)?, &pairs)?)?
            );
        }
        Command::Init { bundle, out } => write(
            &out,
            &json(&Document::new(Bundle::parse(&read(&bundle)?)?))?,
        )?,
        Command::Import {
            export,
            claims,
            title,
            out,
        } => {
            let b = import::import(
                &read(&export.join("manifest.json"))?,
                &read(&export.join("evidence.jsonl"))?,
                &read(&claims)?,
                &title,
            )?;
            write(&out, &json(&Document::new(b))?)?;
        }
        Command::Validate { input } => {
            let d = load(&input)?;
            println!(
                "{}",
                json(
                    &serde_json::json!({"valid":true,"claims":d.bundle.claims.len(),"revision":d.reviews.len()})
                )?
            );
        }
        Command::Analyze { input } => println!("{}", json(&load(&input)?.graph()?)?),
        Command::Review {
            input,
            request,
            out,
        } => {
            let r: ReviewRequest = serde_json::from_str(&read(&request)?)?;
            write(&out, &json(&load(&input)?.review(r)?)?)?;
        }
        Command::Timeline { input, subject } => {
            println!("{}", json(&load(&input)?.timeline(&subject)?)?)
        }
        Command::Report {
            input,
            as_of,
            focus,
            previous,
            out,
        } => {
            let d = load(&input)?;
            let p = previous.map(|p| load(&p)).transpose()?;
            let s = report::markdown(&d, &as_of, &focus, p.as_ref())?;
            if let Some(path) = out {
                write(&path, &s)?;
            } else {
                print!("{s}");
            }
        }
        Command::Diff { before, after } => {
            println!("{}", json(&report::diff(&load(&before)?, &load(&after)?)?)?)
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(2);
    }
}
