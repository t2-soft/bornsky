# Licensing and commercial boundaries

## Public calculation library

Bornsky's original contributions in this repository are licensed under
[Apache-2.0](../LICENSE). This is a permissive open-source licence, not a
non-commercial, trial or source-available licence.

You may use, modify and redistribute the calculation library, including in
closed-source applications and commercial or competing hosted services. There
is no requirement to subscribe to Bornsky, pay us royalties, disclose your
application source, or contribute your modifications back solely because you
use this library. The licence includes a contributor patent grant with its
stated scope and patent-litigation termination conditions.

When distributing the code or derivative works, follow the full licence:
include the licence, preserve applicable attribution and NOTICE content, and
mark modified files. The software is supplied without warranty under the
licence's terms. This explanation does not replace the licence text.

## Inherited code and dependencies

Apache-2.0 does not replace the licences of adapted third-party portions:

- ERFA-derived nutation and precession routines retain the ERFA BSD-style
  notice in [licenses/ERFA.txt](../licenses/ERFA.txt).
- The lunar and Pluto coefficient tables retain the astro-rust MIT notice in
  [licenses/astro-rust.txt](../licenses/astro-rust.txt).
- Cargo dependencies retain their own licences. See the
  [locked dependency inventory](dependency-licenses.md) and
  [third-party provenance](../THIRD_PARTY_NOTICES.md).

Keep `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md` and `licenses/` with source
redistributions. Binary distributors must also reproduce the notices required
by their actual dependency set. The Cargo package's Apache-2.0 metadata describes
our original code; it does not erase inherited notices.

## Separate paid services

Bornsky's Cloudflare-based partner API is a separately operated commercial
service. API access, subscriptions, support and any service commitments are
governed by the agreements for that service, not this software licence.

The hosted service's implementation, authored interpretation catalogs, matching,
forecast synthesis, ranking, rectification and other excluded products are not
distributed here. The public licence grants no rights to that separate material.
See the [export boundary](export-boundary.md) for the full scope.

This separation does **not** restrict what others may build independently using
the public library. Apache-2.0 permits competition using the published code.
Publishing under this licence grants rights to recipients that cannot simply
be withdrawn by later making this repository private.

## Name and attribution

The software licence does not grant Bornsky trademark or branding rights beyond
the reasonable attribution uses stated in Apache-2.0 section 6. You may identify
the library you use; do not imply that a fork or independent service is an
official Bornsky product or is endorsed by Bornsky, IAU, SOFA, JPL or NASA.

## Contributions and releases

Contributors must have authority to submit their work under Apache-2.0 and must
identify any third-party sources and preserve their licences. Employment or
contractual ownership rights must be settled by the relevant rights holders;
an automated source scan cannot establish them.

Before the first public release, the repository owner must confirm that the
original contributions may be licensed this way. Package publication on
crates.io remains disabled and requires a separate review of package contents,
including all applicable licence files. See the [release checklist](release-checklist.md).
