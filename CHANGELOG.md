## Unreleased


## v0.3.0 - 2022-08-21
### Bug Fixes
- make DI container get_factory calls in the injectable macro valid

### Build System/Dependency Changes
- change license in manifests to LGPL-2.1-only

### Chores
- change license to MIT or Apache-2.0

### Code Refactoring
- change errors to be more sane
- make the declare_default_factory macro take a ty
- only re-export DIContainer
- add Cargo feature for preventing circular dependencies
- move creating a dependency trace to it's own function
- hide AnyFactory from docs
- limit the factory macro to the factory feature

### Code Testing
- correct DI container bind tests
- reduce repetition in DI container tests

### Documentation Changes
- change project descriptions to describe it as a framework
- fix declare_default_factory example
- add injection of 3rd-party structs & traits to features list
- add example for displaying a unbound interface error
- simplify with-3rd-party example
- add a example that uses a 3rd party library
- fix IFactory example use statement
- improve the factory example
- correct examples
- fix DI container module documentation
- add license shield to readme
- add factory feature notices

### Features
- allow bind interface to default factory
- prevent binding the same interface more than once
- add detection and prevention of circular dependencies
- add hide impl of Injectable from documentation

### BREAKING CHANGE

Major improvements have been made to error types and the error_stack crate is no longer used

Only DIContainer is re-exported from the di_container module

The 'to' and 'to_factory' methods of BindingBuilder now return 'Result'


## v0.2.1 - 2022-08-01
### Build System/Dependency Changes
- bump versions to 0.2.1

### Documentation Changes
- add v0.2.1 to changelog
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

