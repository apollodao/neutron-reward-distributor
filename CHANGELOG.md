# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2023-11-04

### Changed

- Updates dependencies
- Changes `crate::msg::RewardInfo` to include `liquidity_manager_addr` for the `AstroportVault` and `AstroportPool` variants. N.B. this is a breaking API change.
