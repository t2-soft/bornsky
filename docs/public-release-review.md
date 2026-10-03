# Public release preparation — 2026-10-03

This records the technical review of the calculation-only repository based on
`c4b354ee23914003fefcba9a3bb8de1f5a57bf2c` and the accompanying release-preparation
changes. It does not certify rights ownership or change repository visibility.

## Reviewed

- All five existing commits reachable from fetched branches, tags and PR heads;
  file paths, text content and commit messages passed the boundary/credential
  pattern scan. Commit emails use GitHub's noreply addresses.
- The two earlier PR descriptions, issue/review-comment listings, release
  listings and four existing Actions logs. No credential-pattern matches were
  found in those logs; the PR descriptions discuss the public export boundary.
- Code comments and unfinished-work markers. Obsolete roadmap promises were
  replaced with current behaviour; provisional mean-apogee accuracy and other
  numerical limitations remain documented. Calculation code is unchanged.
- The full Apache-2.0, ERFA and astro-rust licence files match their upstream
  texts after whitespace normalization. Forty-eight external Cargo packages
  have declared permissive licence options, with the additional Unicode-3.0
  obligation recorded in the dependency inventory.
- Authored catalogs, service implementations and paid calculation workflows
  remain excluded. The public API link advertises a separate service; it does
  not distribute that service's implementation or credentials.

## Validation

Passed locally: 69 Rust unit/integration/doc tests; six boundary-check tests;
formatting; strict Clippy and rustdoc; WASM compilation; Rust 1.94 minimum-version
compilation; synthetic JSON example; relative documentation links; whitespace.
CI repeats the calculation checks and now checks reachable Git history too.

## Owner release steps

Merge the preparation PR after CI passes, then complete the
[release checklist](release-checklist.md). In particular, confirm the right to
license original contributions under Apache-2.0, including work supplied by
employees or contractors. Automated tests and licence inventories cannot prove
that authority. Enable the available public security-reporting/scanning features
and verify signed-out access after deliberately changing visibility.

Pattern scans do not prove the absence of every possible secret or private
detail. New commits, dependencies and GitHub discussions need continued review.
