mod cli;

fn main() {
    if let Err(err) = cli::run_cli() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
