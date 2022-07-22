## Unreleased


## v0.1.0 - 2022-07-20
### Build
- use syrette_macros from crates.io

### Chore
- add repository & keywords to Cargo manifests

### Docs
- replace symlinked readme with a copy
- add readme symlink to syrette
- improve and clean up doc comment examples
- split example into multiple files
- correct declare_interface macro example
- remove the crate root example
- add example
- add documentation comments
- add readme

### Feat
- add binding factories to DI container
- add DI container

### Refactor
- use aggressive clippy linting
- remove unused intertrait code
- rename the castable_to macro to declare_interface
- reduce the capabilities of the castable_to macro
- reorganize and improve macros
- use common pointer type aliases
- add dedicated interface & error modules
- move injectable type provider to own file

### Style
- group imports
- add rustfmt config

### Test
- add DI container unit tests

