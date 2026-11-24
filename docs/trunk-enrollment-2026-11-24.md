# Trunk merge queue enrollment

Recorded 2026-11-24. This repository joined the fleet Trunk setup.

- Merge queue: 2c1ec788-2b44-46ab-8631-13652930ca22 (target branch main, Single mode with batching, squash merges)
- Ruleset fleet-trunk-updates: id 27994251, enforcement disabled, Trunk app bypass exempt
- Ruleset fleet-default-branch: id 27994255, active, strict up-to-date off, required checks: hygiene, msrv, rust
- Flaky Tests collection: looseleaf (id WsA9Ny0f)
- This pull request is the first-PR verification that the queue lands a change end to end.
