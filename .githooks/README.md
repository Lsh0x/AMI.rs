# Git Hooks for AMI.rs

This directory contains Git hooks to help maintain code quality.

## Available Hooks

### pre-commit

Runs before each commit to ensure:
- ✅ Code is properly formatted (`cargo fmt`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- 🧪 Optionally run tests (disabled by default for speed)

## Installation

### Option 1: Configure Git to Use These Hooks (Recommended)

```bash
git config core.hooksPath .githooks
```

This tells Git to use the hooks in this directory instead of `.git/hooks`.

### Option 2: Symlink Individual Hooks

```bash
ln -sf ../../.githooks/pre-commit .git/hooks/pre-commit
```

### Option 3: Copy the Hook

```bash
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Customization

You can edit the hooks to:
- Enable test running on every commit (uncomment the test section)
- Add additional checks
- Skip checks temporarily with `git commit --no-verify`

## Skipping Hooks

If you need to skip the pre-commit checks for a specific commit:

```bash
git commit --no-verify -m "your message"
```

⚠️ Use this sparingly! The hooks are there to catch issues early.

