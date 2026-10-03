# Public release checklist

Use this checklist before the first public release and when widening the export
scope. No workflow changes repository visibility or publishes to crates.io.

- [ ] Review the complete source diff and export manifest against the agreed
  calculation-only scope. Inspect the complete fresh Git history, not only HEAD.
- [ ] Confirm rights to license the original contributions under Apache-2.0 and
  retain all third-party notices. Automatic Delta-T code remains excluded.
- [ ] Confirm no catalogs, commercial rules, customer fixtures, credentials,
  private configuration or private Git history are present.
- [ ] In a full clone, fetch all repository branches and tags, then run
  `python scripts/check_public_boundary.py --history`. Inspect the review output
  without publishing matched secrets. Review commit identities/messages and
  GitHub issues, PR discussions, releases and Actions logs too.
- [ ] Run boundary checks, Rust tests, formatting, Clippy, rustdoc and WASM check.
- [ ] Review accuracy/time-scale limitations and run the synthetic JSON example.
- [ ] Review [the licensing model](licensing.md) and the locked dependency
  inventory. Confirm the rights holders authorise Apache-2.0 publication,
  including its permission for independent commercial/competing services.
- [ ] Check the About description, topics, homepage and README links. Keep
  preview services labelled as preview; do not advertise excluded products as
  included library features.
- [ ] Have the repository owner explicitly choose when to make this repo public.
- [ ] Enable GitHub secret scanning/push protection where available and protect
  the default branch with reviews and CI before accepting outside contributions.
- [ ] Enable private vulnerability reporting after public visibility is set,
  and check the reporting path described in `SECURITY.md`.
- [ ] Verify the repository can be read while signed out after publication.

A future crates.io release is separate: choose an available crate name, review
the actual package contents and license/notice inclusion, and then deliberately
change `publish = false`. No credentials are required for current CI.
