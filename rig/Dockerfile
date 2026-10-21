# The test container: one SPRT test (the new engine against the old one),
# then it exits. Built for 64-bit ARM (AWS Graviton) by the SPRT workflow;
# also builds and runs on an ordinary x86 PC for local testing.
# See docs/runbooks/sprt.md and DESIGN.md section 5.

# --- Stage 1: build fastchess, the program that runs the games ---------------
# This stage runs on the build machine's own CPU ($BUILDPLATFORM) and
# cross-compiles for the target, because compiling C++ under ARM emulation
# took over half an hour. TARGETARCH and BUILDARCH are set by Docker.
FROM --platform=$BUILDPLATFORM debian:trixie-slim AS fastchess
ARG TARGETARCH
ARG BUILDARCH
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates g++ git make \
 && if [ "$TARGETARCH" = arm64 ] && [ "$BUILDARCH" != arm64 ]; then \
      apt-get install -y --no-install-recommends g++-aarch64-linux-gnu; fi \
 && rm -rf /var/lib/apt/lists/*
# Pinned to a release tag and checked against its commit, so every image
# runs the same match program.
ARG FASTCHESS_TAG=v1.8.2-alpha
ARG FASTCHESS_COMMIT=f618e34540f94f4719ad3817950618dabe441318
RUN git clone --depth 1 --branch "$FASTCHESS_TAG" https://github.com/Disservin/fastchess /src \
 && test "$(git -C /src rev-parse HEAD)" = "$FASTCHESS_COMMIT"
# NATIVE= turns off -march=native: the image is built on one machine and
# run on another, so it mustn't assume the build machine's CPU.
RUN if [ "$TARGETARCH" = arm64 ] && [ "$BUILDARCH" != arm64 ]; then \
      prefix=aarch64-linux-gnu-; else prefix=; fi \
 && make -C /src -j"$(nproc)" NATIVE= CXX="${prefix}g++" \
 && "${prefix}strip" /src/fastchess

# --- Stage 2: fetch the opening book -----------------------------------------
# 8moves_v3: 34,700 balanced openings, 8 moves each side, public domain (CC0),
# from official-stockfish/books. Pinned to a commit and checked by hash.
# Only files, so it also runs on the build machine's own CPU.
FROM --platform=$BUILDPLATFORM debian:trixie-slim AS book
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl unzip \
 && rm -rf /var/lib/apt/lists/*
ARG BOOKS_COMMIT=a804a97e709f40aa75a00f9d19e84915b7a3a45c
ARG BOOK_SHA256=7e1e9dd118b4bb97d8a8b5b8a790c86e21f8509d59a27d2883767d94477be02e
RUN curl -fsSL -o /book.zip \
      "https://raw.githubusercontent.com/official-stockfish/books/$BOOKS_COMMIT/8moves_v3.pgn.zip" \
 && echo "$BOOK_SHA256  /book.zip" | sha256sum -c - \
 && unzip -q /book.zip -d /book

# --- Final image: fastchess, the book, the AWS CLI and the run script --------
# Debian 13 ("trixie"), because the engines are built on GitHub's Ubuntu
# 24.04 machines and need a C library at least that new (glibc 2.39). A
# fully static build would run anywhere but searches about 30% slower.
FROM debian:trixie-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends awscli ca-certificates python3 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=fastchess /src/fastchess /usr/local/bin/fastchess
COPY --from=book /book/8moves_v3.pgn /opt/rig/8moves_v3.pgn
COPY run_test.py /opt/rig/run_test.py
ENV STARGAZER_BOOK=/opt/rig/8moves_v3.pgn
WORKDIR /work
# Python is process 1 so it receives AWS's stop signal and can save results.
ENTRYPOINT ["python3", "-u", "/opt/rig/run_test.py"]
