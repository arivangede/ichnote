# 2. License: MIT OR Apache-2.0, contributions under DCO

- Status: accepted
- Date: 2026-11-19

## Context

`docs/DESIGN.md` §12 describes an open-core model: the app, data format, and folder sync are
fully open source; a possible future paid product is a managed service (instant sync, real-time
collaboration, team management, backups), not features locked out of the app. The options
considered were MIT/Apache-2.0, AGPL-3.0, and MPL-2.0, plus CLA vs DCO for contributions.

## Decision

- The app, crates, and sync format are dual licensed **MIT OR Apache-2.0**, the Rust ecosystem norm.
- Contributions are accepted under the **Developer Certificate of Origin** (`git commit -s`),
  not a CLA.

## Consequences

- Maximum freedom for users and contributors; third parties may build competing services from
  this code. The future managed service is differentiated by operations, not by license.
- Because the license is permissive, relicensing is not needed for a commercial offering,
  so a CLA is unnecessary. A separate server component can still choose its own license.
- A DCO check in CI can be added once external contributions start.
