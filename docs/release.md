# Release Policy

SBE releases should look like a normal developer-tool release.

## Public Downloads

Publish only user-facing artifacts:

- Windows: one clickable `.msi` installer.
- Linux: one compressed archive containing the `sbe` binary.
- macOS: one compressed archive containing the `sbe` binary.

Do not publish:

- `target/`
- loose build folders
- WiX source files
- `.wixobj` or `.wixpdb`
- debug `.sbe/` indexes

## Why Installer Source Exists

Files under `packaging/` are source code for generating the installer, the same way Rust files are source code for generating `sbe.exe`. End users should never download those files. They download only the generated `.msi` from the release page.

