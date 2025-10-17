# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-17

- [**breaking**] project forked to nilp0inter/jetkvm_client and goals changed

## [Unreleased]

## [0.2.0](https://github.com/davehorner/jetkvm_control/compare/jetkvm_client-v0.1.4...jetkvm_client-v0.2.0) - 2025-03-09

### Added

- [**breaking**] interactive configuration features (-c) to edit jetkvm_client.toml files.

## [0.1.4](https://github.com/davehorner/jetkvm_control/compare/jetkvm_client-v0.1.3...jetkvm_client-v0.1.4) - 2025-03-09

### Added

- *(keyboard)* add send_key_combinations API and Lua binding
- *(examples)* add windows-alt-tab.lua, windows-notepad-helloworld.lua, windows-is_cmd_running.lua
- *(doc)* windows-alt-tab.lua has been extensively documented, specifically for send_key_combinations

## [0.1.3](https://github.com/davehorner/jetkvm_control/compare/v0.1.2...v0.1.3) - 2025-03-03

### Other

- *(deps)* switch webrtc dependency to crates.io registry

## [0.1.2](https://github.com/davehorner/jetkvm_control/compare/v0.1.1...v0.1.2) - 2025-03-03

### Added

- update dependencies, logging, and documentation

## [0.1.1](https://github.com/davehorner/jetkvm_control/compare/v0.1.0...v0.1.1) - 2025-03-02

### Added

- add Lua script execution mode and update configuration handling
- *(cli)* add command-line support and update dependency configuration
- *(lua)* add Lua engine for async RPC integration
- add lua support with feature flag

### Other

- add CHANGELOG.md and Cargo.lock

## [0.1.0](https://github.com/davehorner/jetkvm_control/releases/tag/v0.1.0) - 2025-03-02

### Added

- *(ci)* add release-plz workflow for automated releases
- add Windows Notepad example and update RPC field parsing

### Other

- Update README.md
- 🔧 Improve JetKVM Config Loading with Multi-Location Precedence
- Update README.md
- Create README.md
- initial release of jetkvm_client 0.1.0
- Initial commit
