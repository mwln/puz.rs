# Releasing

This project ships two crates from one workspace, and they're versioned
together:

- `puz-parse`, the library ([crates.io](https://crates.io/crates/puz-parse))
- `puz`, the CLI ([crates.io](https://crates.io/crates/puz))

`puz` depends on `puz-parse`, so `puz-parse` is always published first.

Publishing is triggered by pushing a version tag, and those tags are protected,
so only repository admins can cut a release (see
[Tag protection](#tag-protection)). The rest of this document explains how that
works, then walks a maintainer with admin access through actually doing it.

## How a release happens

Releases are driven by tags. A maintainer runs `cargo release`, and the rest
follows from the tag it pushes:

1. `cargo release` bumps the crate versions, commits, tags `vX.Y.Z`, and pushes
   both `main` and the tag.
2. The tag triggers `.github/workflows/publish.yml`, which publishes `puz-parse`
   and then `puz` to crates.io.
3. The tag also triggers `.github/workflows/release.yml`, which creates the
   GitHub Release and attaches the `puz` CLI binaries.

Publishing to crates.io runs in CI, not on a maintainer's machine. That's why
`release.toml` sets `publish = false`. There's no crates.io token stored
anywhere; the workflow gets a short-lived one through crates.io Trusted
Publishing (OIDC).

The only binaries we build are for the `puz` CLI. `puz-parse` is a library, so
there's nothing to build for it.

## What a release needs

A release only goes out when these are in place:

- **A version tag pushed by an admin.** Tags matching `v*` are protected, so
  only repository admins can push one (see [Tag protection](#tag-protection)).
- **crates.io Trusted Publishing, configured for both crates.** This is a
  one-time setup that lets CI publish without a stored token (see
  [Trusted Publishing setup](#trusted-publishing-setup-one-time)).
- **A green `main`.** The release builds from the tagged commit, so `main` has
  to be in a releasable state before the tag is cut.

Maintainers cutting a release also need
[cargo-release](https://github.com/crate-ci/cargo-release) installed
(`cargo install cargo-release --locked`) and a clean, up-to-date `main`.

## Cutting a release

We use [cargo-release](https://github.com/crate-ci/cargo-release) to bump the
versions, tag, and push. Its behavior here is configured in
[`release.toml`](release.toml); see the
[cargo-release reference](https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md)
for what the individual settings do. From a clean, up-to-date `main`:

1. **Preview it.** `cargo release` does nothing without `--execute`, so run it
   once to see the planned version bump, commit, and tag:

   ```sh
   cargo release patch      # or: minor / major / an explicit version like 0.2.0
   ```

2. **Do it for real:**

   ```sh
   cargo release patch --execute
   ```

   This pushes `main` and the `vX.Y.Z` tag. The tag is what kicks off the two
   release workflows.

3. **Watch CI:**

   ```sh
   gh run list --limit 5
   ```

   - `publish` publishes `puz-parse` then `puz` to crates.io.
   - `release` creates the GitHub Release and uploads the `puz` CLI binaries.

Then double-check the new versions on crates.io and the binaries on the
[Releases page](https://github.com/mwln/puz.rs/releases).

## Backfilling binaries for an existing release

If a target's binary upload fails (or an asset needs to be re-attached), the
`release` workflow can be re-run manually against an existing tag without
re-publishing to crates.io:

```sh
gh workflow run release.yml -f tag=vX.Y.Z
```

On manual dispatch the workflow skips creating the release (it already exists)
and only builds and uploads the CLI binaries to that tag's release.

## Tag protection

A repository ruleset ("protect release tags") blocks creation, updating, and
deletion of `v*` tags for everyone except repository admins. This stops anyone
without admin access from triggering a publish or release by pushing a tag.
Admins bypass the rule, so `cargo release` works normally for them.

## Trusted Publishing setup (one-time)

Before CI can publish, crates.io
[Trusted Publishing](https://crates.io/docs/trusted-publishing) must be
configured for **both** crates. For each of `puz-parse` and `puz` on crates.io:

1. Go to the crate → **Settings → Trusted Publishing**.
2. Add a GitHub Actions publisher:
   - Repository owner: `mwln`
   - Repository name: `puz.rs`
   - Workflow filename: `publish.yml`
   - Environment: `release`

Once configured, `publish.yml` authenticates via OIDC with no stored token.

## Troubleshooting

- **`crate version already exists`.** The version was already published to
  crates.io. Bump to a new version; published versions are immutable.
- **Publish job fails on OIDC/auth.** Trusted Publishing isn't configured for
  that crate yet (see above).
- **A single binary target fails.** Fix the cause on `main`, then backfill
  that release's assets with `gh workflow run release.yml -f tag=vX.Y.Z`.
