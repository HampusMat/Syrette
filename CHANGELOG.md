## Unreleased


## v0.2.1 - 2022-07-31
### Documentation Changes
- add doc comments & deny missing docs
- add contributing section in readme


## v0.2.0 - 2022-07-31
### Build System/Dependency Changes
- bump versions to 0.2.0
- add docs.rs all-features flag

### Code Refactoring
- tidy up DI container internals
- add Intertrait cast error
- rename InterfacePtr to TransientPtr
- add back Intertrait tests & Rc support
- hide castable factory from docs
- clean up intertrait lib

### Documentation Changes
- add v0.2.0 to changelog
- add binding singletons to list of features
- add asynchronous functionality to todo
- add generics support to list of features

### Features
- add injecting singletons into constructors
- implement binding singletons
- add support for generics

### Performance Improvements
- use ahash in DI container

### BREAKING CHANGE

InterfacePtr has been renamed to TransientPtr


## v0.1.1 - 2022-07-22
### Build System/Dependency Changes
- bump versions to 0.1.1
- add local path to macros dependency

### Code Refactoring
- make factories an optional feature
- re-export dependency of error_stack
- reorganize folder hierarchy

### Documentation Changes
- add v0.1.1 to changelog
- add optional factory feature name in readme
- add changelog
- fix typo in example in readme
- add shields, examples & more to readme
- rename example folder to examples
- use syrette from crates.io in example


## v0.1.0 - 2022-07-20
### Build System/Dependency Changes
- use syrette_macros from crates.io

### Chores
- add repository & keywords to Cargo manifests

### Code Refactoring
- use aggressive clippy linting
- remove unused intertrait code
- rename the castable_to macro to declare_interface
- reduce the capabilities of the castable_to macro
- reorganize and improve macros
- use common pointer type aliases
- add dedicated interface & error modules
- move injectable type provider to own file

### Code Testing
- add DI container unit tests

### Documentation Changes
- replace symlinked readme with a copy
- add readme symlink to syrette
- improve and clean up doc comment examples
- split example into multiple files
- correct declare_interface macro example
- remove the crate root example
- add example
- add documentation comments
- add readme

### Features
- add binding factories to DI container
- add DI container

### Style Improvements
- group imports
- add rustfmt config

