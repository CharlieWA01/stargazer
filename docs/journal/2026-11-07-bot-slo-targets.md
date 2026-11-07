# 2026-11-07: Targets for the bot's SLOs

## Goal
Finish Milestone 5b: turn the bot's report-only signals into targets,
using the data collected since 2026-10-27.

## What was done
- Read 11 days of the observer's records and the log metrics (pulled by
  Charlie; details in cloud-homelab's journal of the same date).
- Charlie chose the targets (DECISIONS.md). The alarms, the zeros on
  quiet periods, and a per-speed rating in the daily records were built
  in cloud-homelab.
- DESIGN.md section 6 and the roadmap updated: 5b is done.

## Problems and fixes
- The daily "rating" mixed bullet, blitz and rapid (it swung from 1,844
  to 2,186 between days). Now kept per speed.

## How it was verified
The numbers, from the observer's daily rows: 100% of checks online, 7
time losses in 649 games, at most 2 in a day. No code here changed.

## Impact
A release that hurts time management now shows up as an email within a
day, which matters for the time-management work on the roadmap.
