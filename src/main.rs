use std::io;

fn main() {
    // `stargazer bench`: print the bench line and exit (testing tools run
    // the engine this way to fingerprint a build).
    if std::env::args().nth(1).as_deref() == Some("bench") {
        println!(
            "{}",
            stargazer::uci::bench_report(stargazer::bench::BENCH_DEPTH)
        );
        return;
    }
    let stdin = io::stdin();
    // Stdout itself rather than a lock on it: the search thread prints too.
    stargazer::uci::run(stdin.lock(), io::stdout());
}
