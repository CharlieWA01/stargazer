# 2026-11-07: Game history and the status page (Milestone 5c design)

## Goal
Design 5c, the bot's game history in S3 with Athena and a public status
page, for Charlie to approve before anything is built.

## What was done
- Read the 5b observer and its table (cloud-homelab), the portfolio's
  site, deploy and headers, and current Athena and Glue pricing.
- Wrote the design doc "Stargazer 5c: game history and status page"
  (Claude Docs): a nightly archiver Lambda saves each day's games to S3
  and writes stats from Athena; the observer writes a small live file;
  a static page on the portfolio site reads both. Six choices, each with
  alternatives and costs.
- Charlie took all six recommendations (DECISIONS.md). DESIGN.md section
  6 marks 5c decided; the roadmap splits 5c into its three parts.

## Problems and fixes
- Two traps found in the design, before any code: the portfolio's deploy
  runs `aws s3 sync --delete`, which would delete the status files on
  every deploy (it gets `--exclude "data/*"`), and the site is cached
  for a day (the files are written with a 60-second `Cache-Control`,
  which the existing cache policy honours).
- The usual Athena tutorial uses a Glue crawler to find partitions: about
  $0.15 a run, $4.50 a month nightly. Partition projection does it for
  free.

## How it was verified
A design only; no code changed here. Prices were checked on 2026-11-07
(Athena $5 per TB with a 10 MB minimum per query; Glue's catalog free for
the first million objects and requests; crawlers $0.44 per DPU-hour).

## Impact
Nothing changes yet. When built: about $0.05 a month, one new private
bucket, two Lambdas writing one file each into the site bucket, and no
new public endpoint.
