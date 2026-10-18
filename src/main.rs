use std::io;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    stargazer::uci::run(stdin.lock(), stdout.lock());
}
