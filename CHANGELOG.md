# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.3.10] - 2018-04-23

### Added
- Allow static linking of /usr/ on macOS (#42)
- Add support for parsing `-Wl,` style framework flags (#48)
- Parse defines in `pkg-config` output (#49)
- Rerun on `PKG_CONFIG_PATH` changes (#50)
- Introduce target-scoped variables (#58)
- Respect pkg-config escaping rules used with --cflags and --libs (#61)

### Changed
- Use `?` instead of `try!()` in the codebase (#63)
