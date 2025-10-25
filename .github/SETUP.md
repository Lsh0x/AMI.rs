# Repository Setup Guide

This guide walks you through setting up all the automated workflows for this repository.

## 1. Enable GitHub Pages

To deploy documentation automatically:

1. Go to your repository on GitHub
2. Click **Settings** (top navigation)
3. Click **Pages** (left sidebar)
4. Under **Build and deployment**:
   - **Source**: Select `GitHub Actions`
   - **Branch**: Leave as default (main)
5. Click **Save**

**Verification**: After the next push to `main`, the documentation will be deployed to:
```
https://<your-username>.github.io/AMI.rs/rustyiam/
```

## 2. Configure crates.io Publishing (Optional)

To automatically publish releases to crates.io:

### Generate a crates.io Token

1. Go to https://crates.io/settings/tokens
2. Click **New Token**
3. Name: `GitHub Actions - AMI.rs`
4. Scopes: Select `publish-update`
5. Click **Create Token**
6. **Copy the token** (you won't see it again!)

### Add Token to GitHub Secrets

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `CARGO_REGISTRY_TOKEN`
5. Value: Paste your crates.io token
6. Click **Add secret**

**Verification**: The next version bump will automatically publish to crates.io.

## 3. Configure Git Hooks (For Contributors)

All contributors should configure Git hooks for code quality:

```bash
git config core.hooksPath .githooks
```

This enables:
- **pre-commit**: Checks formatting and runs clippy
- **prepare-commit-msg**: Provides conventional commit template

## 4. Verify Workflows

Check that all workflows are enabled:

1. Go to **Actions** tab on GitHub
2. You should see these workflows:
   - ✅ CI
   - ✅ Code Coverage
   - ✅ Documentation Deployment
   - ✅ Version Bump & Deploy Docs
   - ✅ Deploy Documentation (Manual)
   - ✅ Changelog
   - ✅ Release
   - ✅ MSRV Check
   - ✅ PR Labeler

If any are disabled, click on them and enable.

## 5. Test the Setup

### Test Documentation Deployment

Trigger manual documentation deployment:

```bash
gh workflow run docs-manual.yml
```

Or via GitHub UI:
1. Go to **Actions** tab
2. Click **Deploy Documentation (Manual)**
3. Click **Run workflow**
4. Select branch: `main`
5. Click **Run workflow**

### Test Version Bump

Create a test commit on a branch:

```bash
git checkout -b test/setup
echo "# Test" >> README.md
git add README.md
git commit -m "docs: test automatic versioning"
git push origin test/setup
```

Create a PR and merge it. The version will be automatically bumped.

## 6. Optional: Configure Branch Protection

Protect the `main` branch to ensure quality:

1. Go to **Settings** → **Branches**
2. Click **Add rule** under "Branch protection rules"
3. Branch name pattern: `main`
4. Enable:
   - ✅ Require a pull request before merging
   - ✅ Require approvals (1)
   - ✅ Require status checks to pass before merging
     - Select: `test`, `fmt`, `clippy`, `doc`
   - ✅ Require branches to be up to date before merging
   - ✅ Require conversation resolution before merging
5. Click **Create**

## 7. Repository Secrets Summary

| Secret Name | Required | Purpose | Where to Get |
|------------|----------|---------|--------------|
| `GITHUB_TOKEN` | ✅ Yes (auto) | GitHub API, releases | Auto-provided by GitHub |
| `CARGO_REGISTRY_TOKEN` | ⚠️ Optional | Publish to crates.io | https://crates.io/settings/tokens |

## Troubleshooting

### GitHub Pages Returns 404

**Problem**: Documentation URL returns 404.

**Solutions**:
1. Verify Pages is enabled (see step 1)
2. Check that "Source" is set to "GitHub Actions"
3. Re-run the documentation workflow
4. Wait 2-3 minutes after deployment completes

### crates.io Publish Fails

**Problem**: Workflow fails at publish step.

**Solutions**:
1. Verify `CARGO_REGISTRY_TOKEN` secret is set correctly
2. Ensure you're a collaborator on the crate (if it already exists)
3. Check if version already exists on crates.io
4. Verify `Cargo.toml` metadata is complete

### Version Not Bumping

**Problem**: Commits pushed but no version bump triggered.

**Solutions**:
1. Ensure commits follow conventional format (see `.github/VERSIONING.md`)
2. Check workflow logs: **Actions** → **Version Bump & Deploy Docs**
3. Verify commits are on `main` branch
4. Check for `[skip ci]` in commit message

### Hooks Not Working

**Problem**: Pre-commit hook not running.

**Solutions**:
```bash
# Verify hooks are configured
git config core.hooksPath

# Should output: .githooks

# If not, configure it:
git config core.hooksPath .githooks

# Verify hook is executable
ls -la .githooks/pre-commit
# Should show: -rwxr-xr-x

# If not, make it executable:
chmod +x .githooks/pre-commit
```

## Additional Resources

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)

## Support

For issues or questions:
1. Check the [VERSIONING.md](.github/VERSIONING.md) guide
2. Review workflow logs in the **Actions** tab
3. Open an issue on GitHub

