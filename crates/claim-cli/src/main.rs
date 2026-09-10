use clap::Parser;
#[derive(Parser)]
#[command(version, about = "Local evidence-first claim atlas")]
struct Args {}
fn main() {
    Args::parse();
}
