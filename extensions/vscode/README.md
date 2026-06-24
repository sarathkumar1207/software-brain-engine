# Software Brain Engine VS Code Extension

Foundation extension for running the local `sbe` CLI from VS Code.

## Commands

- `SBE: Trace Symbol`
- `SBE: Show Impact`
- `SBE: Explain Symbol`

The extension invokes `sbe trace --json`, `sbe impact --json`, and `sbe inspect --json`, parses the JSON output, and renders results in the Explorer tree. It does not use webviews.
