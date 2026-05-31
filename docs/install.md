# Install And Release

SBE should install like a normal developer tool: download one installer, click it, then run `sbe` from any terminal.

`target/` is not a release folder. It is Cargo's local build cache and is ignored by Git. Release users should only see generated artifacts such as `.msi` on Windows or archives on Unix platforms.

## Windows

Preferred distribution is a single MSI:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-msi.ps1 -Version 0.1.0
```

The MSI build requires WiX Toolset v3.x on `PATH`. Install it from an Administrator terminal, for example:

```powershell
choco install wixtoolset --no-progress -y
```

The MSI installs `sbe.exe` under Program Files and appends the install folder to the system `PATH`, so users can run `sbe` from a new terminal after installation.

The release artifact should be:

```text
sbe-0.1.0-windows-x64.msi
```

Do not publish `target/`, `.wxs`, `.wixobj`, or loose build folders as release downloads.

For development:

```powershell
cargo install --path crates/cli --force
```

## macOS and Linux

Release builds publish compressed native binaries from GitHub Actions. Development install:

```bash
cargo install --path crates/cli --force
```

## Verify

```bash
sbe version
sbe doctor /path/to/project
```

## Maintainer Release Flow

1. Update version metadata.
2. Run the quality gate:

```powershell
cargo fmt --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release -p sbe-cli
```

3. Create a tag such as `v0.1.0`.
4. GitHub Actions builds release artifacts:
   - `sbe-0.1.0-windows-x64.msi`
   - `sbe-linux-x64.tar.gz`
   - `sbe-macos-arm64.tar.gz`
