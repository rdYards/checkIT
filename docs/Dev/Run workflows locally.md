## Prerequisites

* Docker (required for running workflows)
* `act` CLI tool
* Basic knowledge of GitHub Actions workflows
## Installation
1. Install Docker:
   - On Linux: Follow your distribution's instructions for Docker
   - On macOS: Install Docker Desktop from [docker.com](https://www.docker.com/products/docker-desktop/)
   - On Windows: Install Docker Desktop from [docker.com](https://www.docker.com/products/docker-desktop/)

2. Install `act`:
   ```bash
   brew install act  # macOS
   sudo snap install act --classic  # Ubuntu
   choco install act  # Windows
   ```
## Basic Usage
To run the entire workflow locally:

```bash
act -j release
```

This will execute the release workflow using the local environment.
## Running Specific Jobs
You can run individual jobs from the workflow:

```bash
# Run the macOS release job
act -j macos-release

# Run the Linux package building job
act -j linux-release

# Run the Flatpak building job
act -j flatpak-release
```
## Using Tags
Since the workflow triggers on tag pushes, you can simulate this locally:

```bash
act -e event-tag.json
```

Where `event-tag.json` contains:
```json
{
  "repository": "checkIT",
  "ref": "refs/tags/v0.0.8",
  "ref_type": "tag"
}
```
## Useful Options
* `--secret` or `-s`: Pass secrets
* `--job`: Run specific job
* `--eventpath`: Specify custom event JSON
* `--dryrun`: Dry run without executing
* `--container-architecture`: Specify architecture (linux/amd64, linux/arm64)
## Examples
1. Test macOS build:
```bash
act -j macos-release --container-architecture linux/amd64
```

2. Test Linux builds with specific Rust version:
```bash
act -j linux-release --secret RUST_TOOLCHAIN=stable
```

3. Dry run of the entire release workflow:
```bash
act -j release --dryrun
```
## Troubleshooting
1. If you get permission errors, ensure Docker is running and your user has access
2. For missing tools, check the Docker image used by the workflow