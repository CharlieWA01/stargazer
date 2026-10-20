use std::io;

fn main() {
    let stdin = io::stdin();
    // Stdout itself rather than a lock on it: the search thread prints too.
    stargazer::uci::run(stdin.lock(), io::stdout());
}
