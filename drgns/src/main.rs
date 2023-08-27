use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, required = false)]
    input: String,

    #[arg(short, long, action, default_value_t = false, required = false)]
    check: bool,
}

fn main() {
    let args = Args::parse();
}
