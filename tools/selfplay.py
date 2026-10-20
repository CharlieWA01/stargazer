"""Play Stargazer against itself under a real clock, as a quick check that
it moves in time and only plays legal moves.

    cargo build --release
    python3 tools/selfplay.py target/release/stargazer

Each game gives both sides the same clock (for example 10 seconds plus 0.1
per move). This script keeps the clocks the way a GUI would: it times each
move from `go` to `bestmove`, subtracts it, then adds the increment. It
stops a game when the engine has no legal move (`bestmove 0000`) or after
300 half-moves (it doesn't know the draw rules itself). The engine refuses
an illegal move with an `info string`, which this script reports.

Not a strength test: that's SPRT (Milestone 3).
"""

import subprocess
import sys
import time

TIME_CONTROLS = [(10_000, 100), (3_000, 0), (1_000, 10)]  # (ms, increment ms)
MAX_PLIES = 300


def start(engine):
    process = subprocess.Popen(
        [engine], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1
    )
    send(process, "uci")
    while process.stdout.readline().strip() != "uciok":
        pass
    return process


def send(process, command):
    process.stdin.write(command + "\n")
    process.stdin.flush()


def play(engine, base_ms, increment_ms):
    engines = [start(engine), start(engine)]
    clocks = [base_ms, base_ms]
    moves, problems = [], []
    for ply in range(MAX_PLIES):
        side = ply % 2
        player = engines[side]
        position = "position startpos"
        if moves:
            position += " moves " + " ".join(moves)
        send(player, position)
        send(player, f"go wtime {clocks[0]} btime {clocks[1]} winc {increment_ms} binc {increment_ms}")
        started = time.monotonic()
        while True:
            line = player.stdout.readline().strip()
            if line.startswith("info string"):
                problems.append(f"ply {ply}: {line}")
            if line.startswith("bestmove"):
                break
        clocks[side] -= int((time.monotonic() - started) * 1000)
        if clocks[side] < 0:
            problems.append(f"lost on time at ply {ply}")
            break
        clocks[side] += increment_ms
        move = line.split()[1]
        if move == "0000":
            break
        moves.append(move)
    for player in engines:
        send(player, "quit")
        player.wait()
    return moves, clocks, problems


def main():
    engine = sys.argv[1] if len(sys.argv) > 1 else "target/release/stargazer"
    all_ok = True
    for base_ms, increment_ms in TIME_CONTROLS:
        moves, clocks, problems = play(engine, base_ms, increment_ms)
        print(f"{base_ms / 1000:g}s + {increment_ms / 1000:g}s: {len(moves)} half-moves, "
              f"clocks left {clocks[0]} ms / {clocks[1]} ms")
        print("  opening:", " ".join(moves[:12]))
        for problem in problems:
            print("  PROBLEM:", problem)
            all_ok = False
    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
