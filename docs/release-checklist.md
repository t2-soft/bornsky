# Public release checklist

The repository remains private while this preparation is reviewed. No workflow
changes repository visibility or publishes to crates.io.

- [ ] Review the complete source diff and export manifest against the agreed
  calculation-only scope. Inspect the complete fresh Git history, not only HEAD.
- [ ] Confirm rights to license the original contributions under Apache-2.0 and
  retain all third-party notices. Automatic Delta-T code remains excluded.
- [ ] Confirm no catalogs, commercial rules, customer fixtures, credentials,
  private configuration or private Git history are present.
- [ ] Run boundary checks, Rust tests, formatting, Clippy, rustdoc and WASM check.
- [ ] Review accuracy/time-scale limitations and run the synthetic JSON example.
- [ ] Have the repository owner explicitly choose when to make this repo public.
- [ ] Enable GitHub secret scanning/push protection where available and protect
  the default branch with reviews and CI before accepting outside contributions.

A future crates.io release is separate: choose an available crate name, review
the actual package contents and license/notice inclusion, and then deliberately
change `publish = false`. No credentials are required for current CI.
