# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/ansg191/restic-operator/compare/restic-operator-v0.1.3...restic-operator-v0.1.4) - 2024-12-15

### Fixed

- set hostname of backup to job name

## [0.1.3](https://github.com/ansg191/restic-operator/compare/restic-operator-v0.1.2...restic-operator-v0.1.3) - 2024-12-15

### Fixed

- change `PASSWORD_FILE_PATH` to match volumes
- add default args when none are provided

## [0.1.2](https://github.com/ansg191/restic-operator/compare/restic-operator-v0.1.1...restic-operator-v0.1.2) - 2024-12-15

### Other

- move CRD gen from build.rs to xtask
- Add pre-release CI

## [0.1.1](https://github.com/ansg191/restic-operator/compare/restic-operator-v0.1.0...restic-operator-v0.1.1) - 2024-12-15

### Fixed

- fix missing registry in docker build

### Other

- fix docker build tag filter
