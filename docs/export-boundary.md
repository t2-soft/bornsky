# Export boundary

This repository is a fresh, reviewed file-level copy. No source repository Git
history, branches, tags, pull-request discussions, deployments or environment
files belong here. The original engine and services remain unchanged.

## Approved calculation modules

`angle`, `aspects`, `bodies`, `chart`, `coords`, `ephemeris` (Moon, Pluto and lunar
points), `houses`, `nutation`, and `time`.

Permitted data is limited to mathematical coefficients and synthetic or public
numerical test fixtures needed by those modules. Interpretation prose, commercial
rules and user-derived fixtures are outside this boundary.

## Excluded

- Authored catalogs, readings, translations and report templates.
- Forecasts, annual reports, convergence, significance/ranking and narrative rules.
- Matching/synastry products, horary judgments, rectification, directions,
  profections, progressions, astrocartography and other premium workflows.
- HTTP APIs, API keys, billing, entitlement checks, accounts, bookings and inbox.
- Databases, migrations, backups, logs, customer data, deployment files and secrets.
- Web/mobile application code, brand assets, private docs and private Git history.
- The prior automatic Delta-T implementation and its coefficient table.

Only two basic trigonometric identities needed by the house solvers were inlined
from a previously shared helper: `tan(dec) * tan(lat)` and `tan(lat) * sin(hour)`.
No primary-directions calculation or judgment was copied with them.

## Intentional standalone differences

- `natal_chart` requires explicit TT - UT1 seconds and returns `Result` for invalid
  inputs. The output records the correction and TT JD.
- The output version starts at 1 for this library. It is independent of any
  hosted API's versions or caching rules.
- Comments refer to public documentation; third-party attribution is retained.
- Cargo dependencies and lockfile are generated for this isolated workspace.

## Future updates

Copy individual reviewed files or apply focused patches. Never merge a private
repository's history, add it as a remote here, use a recursive directory copy, or
push an archive of the private workspace.

`export-manifest.json` enumerates every permitted file. Run
`python scripts/check_public_boundary.py` before every commit. CI checks it too,
including direct dependencies and common credential patterns. New files or
dependencies require an explicit boundary review. The check is a guardrail, not
a proof that arbitrary text is safe: reviewers must inspect every diff and any
manifest change for private logic, prose or identifying data.
