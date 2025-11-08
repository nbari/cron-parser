# Release Process

This document describes the release process for `cron-parser`.

## Prerequisites

- You must have push access to the repository
- Set up `CARGO_REGISTRY_TOKEN` secret in GitHub repository settings
  - Get your token from https://crates.io/me
  - Add it to repository Settings → Secrets and variables → Actions → New repository secret
  - Name: `CARGO_REGISTRY_TOKEN`

## Creating a Release

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "0.11.0"  # Update this
```

### 2. Update CHANGELOG.md

Move changes from `[Unreleased]` to a new version section:

```markdown
## [0.11.0] - 2024-12-15

### Added
- New feature XYZ

### Changed
- Improved ABC

### Fixed
- Bug fix for DEF
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.11.0"
git push origin develop
```

### 4. Merge to Main

Create a PR from `develop` to `main` and merge it.

### 5. Create and Push Tag

```bash
git checkout main
git pull origin main
git tag -a 0.11.0 -m "Release 0.11.0"
git push origin 0.11.0
```

### 6. Automated Release

Once the tag is pushed, GitHub Actions will automatically:

1. **Build and Test**: Run all tests on the release build
2. **Generate Release Notes**: Extract changelog for this version
3. **Create GitHub Release**: 
   - Create a GitHub release with the tag
   - Attach source code archives (.tar.gz and .zip)
   - Add SHA256 checksums
4. **Publish to crates.io**: Automatically publish to crates.io

### 7. Verify Release

Check the following:

- [ ] GitHub Release created: https://github.com/nbari/cron-parser/releases
- [ ] Published to crates.io: https://crates.io/crates/cron-parser
- [ ] Documentation updated: https://docs.rs/cron-parser

## Manual Publishing (if needed)

If automatic publishing fails, you can publish manually:

```bash
# Dry run first
cargo publish --dry-run

# Actual publish
cargo publish
```

Or use the GitHub Actions workflow dispatch:

1. Go to Actions → "Publish to crates.io"
2. Click "Run workflow"
3. Select branch: `main`
4. Set "dry-run" to `false`
5. Click "Run workflow"

## Hotfix Releases

For urgent fixes:

1. Create a branch from `main`: `git checkout -b hotfix/0.10.1 main`
2. Make the fix and commit
3. Update version to `0.10.1` in `Cargo.toml`
4. Update `CHANGELOG.md`
5. Create PR to `main`
6. After merge, tag and release as normal (e.g., `git tag -a 0.10.1`)
7. Backport to `develop`: `git checkout develop && git merge main`

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backwards compatible
- **Patch** (0.0.1): Bug fixes, backwards compatible

## Troubleshooting

### Release workflow failed

Check the GitHub Actions logs:
- Go to Actions tab
- Find the failed "Release" workflow
- Review the logs for errors

### Publishing to crates.io failed

Common issues:
- Version already published (can't republish same version)
- Missing or invalid `CARGO_REGISTRY_TOKEN`
- Cargo.toml validation errors

Run `cargo publish --dry-run` locally to debug.

### Release notes are empty

Make sure `CHANGELOG.md` has an entry for the version being released:
- Format: `## [0.11.0] - 2024-12-15`
- Must be under the version heading

## Post-Release

After a successful release:

1. Announce on relevant channels (if applicable)
2. Update any dependent projects
3. Create a new `[Unreleased]` section in `CHANGELOG.md`
