# GitHub Actions Workflows

## Build and Release ([`build-and-release.yml`](build-and-release.yml))

This workflow automatically builds native executables for all platforms when code is pushed to the `main` branch.

### Supported Platforms

| Platform | Runner | Target Triple | Binary Name |
|----------|--------|---------------|-------------|
| macOS Intel (x86_64) | macos-13 | x86_64-apple-darwin | mm3sim |
| macOS Apple Silicon (aarch64) | macos-latest | aarch64-apple-darwin | mm3sim |
| Windows amd64 | windows-latest | x86_64-pc-windows-msvc | mm3sim.exe |
| Linux amd64 | ubuntu-latest | x86_64-unknown-linux-gnu | mm3sim |

### How It Works

1. **Build Phase**: Four parallel jobs build release binaries for each platform
2. **Release Phase**: Once all builds complete, a single job creates a GitHub Release with all binaries

### Manual Trigger

You can manually trigger the workflow by going to:
- `Actions` tab in your repository
- Select `Build and Release` workflow
- Click "Run workflow"

Or via GitHub API:
```bash
curl -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/OWNER/REPO/actions/workflows/build-and-release.yml/dispatches \
  -d '{"ref":"main"}'
```

### Release Contents

When a push to `main` is detected, the workflow will:
1. Build all platforms in parallel
2. Create a release tagged with the branch name (e.g., `main`)
3. Upload binaries for all four platforms as release assets

You can download and verify all binaries from the GitHub Release page.

### Notes

- The workflow uses separate jobs per platform to maximize parallelization
- Each artifact is uploaded separately, then merged in the final step
- Binary names are determined by Cargo's default naming convention
- The `generate_release_notes: true` option creates automatic release notes based on git commits
