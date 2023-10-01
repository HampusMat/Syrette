## Unreleased


## v0.5.1 - 2023-10-01
### Bug Fixes
- allow dependency interface types other than trait & path
- set injectable macro dummies directly after parsing input
- make dummy Injectable & AsyncInjectable impls valid

### Build System/Dependency Changes
- bump version to 0.5.1
- remove examples from files excluded when packaging

### Chores
- make docs style html for macro crate not a symlink

### Code Refactoring
- remove impossible unwrap in injectable macro
- make camel cased text snake cased without regex
- remove unnecessary cloning of impl generics & self type
- remove unnecessary doc(cfg) attrs on private modules

### Code Testing
- import test util functions for injectable macro tests
- make unit tests not return Result

### Documentation Changes
- remove nonexistent feature from feature list in readme
- clarify named macro documentation
- fix custom CSS not used by docs.rs

### Style Improvements
- shorten lines exceeding 90 in width in injectable macro


## v0.5.0 - 2023-09-21
### Bug Fixes
- make the factory macro not change its input
- change terminology in injectable macro async flag error
- remove unwrap from generated implementations of Injectable
- add missing dummy async injectable impl

### Build System/Dependency Changes
- bump versions to 0.5.0
- exclude files when packaging
- change repository in Cargo.toml to the Github repo url
- add Cargo.lock to keep predicates-tree dependency old
- improve the fix for the predicates-tree MSRV change
- add temporary fix for predicates-tree MSRV change
- use mockall 0.11.4

### Chores
- fix warnings when only the async feature enabled
- remove the factory macro async flag

### Code Refactoring
- make the async DI container not inside a Arc
- replace threadsafe castable factory Fn impl with method
- make threadsafe castable factory take DI container param
- move castable factory to directory module
- make the blocking DI container not inside a Rc
- replace castable factory Fn impl with method
- make castable factory take DI container param
- move DI container get_binding_providable to other impl
- remove async DI container prelude module
- remove async DI container interface
- remove blocking DI container prelude module
- remove blocking DI container interface
- remove unnecessary block in the BindingBuilder::to method
- remove unnecessary phantom data fields from providers
- pass around BindingOptions instead of name
- replace use_dependency_history with a more generic macro
- remove useless Sealed impl for DependencyHistory
- use derive for DependencyHistory Default impl
- remove IDependencyHistory
- remove SomeThreadsafePtr & move variants to SomePtr
- rename the async flag of the declare_interface macro
- rename CastFromSync to CastFromArc
- remove async_closure macro from API
- remove calls to default on unit structs
- remove SomeThreadsafePtrError
- remove unnecessary must_use attributes
- make binding builder & configurator methods take self ownership
- put syn_path_to_string in a extension trait
- fix Clippy lint in threadsafe castable factory
- allow manual let else in macros crate
- fix Clippy lints
- add deny unsafe code in macros crate

### Code Testing
- import proc_macro2 TokenStream in dependency tests
- make the prevent-circular example an integration test
- add & improve MacroFlag unit tests
- clean up create_caster_fn_ident unit test
- clean up DeclareDefaultFactoryMacroArgs unit tests
- clean up FactoryMacroArgs unit tests
- clean up binding configurator unit tests
- clean up DeclareInterfaceArgs unit tests
- remove unused std::error::Error import
- remove vec macro usages in IteratorExt unit tests
- clean up InjectableMacroArgs unit tests
- remove Result return value of can_build_dependencies
- fix create_single_get_dep_method_call unit tests

### Documentation Changes
- add release 0.5.0 to changelog
- add examples to DI container & related functions
- add threadsafe flag to IFooFactory in async-factory example
- add existance reason to DependencyHistory docs
- remove the 'unbound' example
- add missing TransientPtr import to factory macro example
- improve injectable macro docs
- remove panics sections from macros
- move the injectable macro example section
- fix wrong quotes in binding configurators

### Features
- make dependency history new method const
- make binding options name method const
- expose DI container get_bound methods to public API
- add DependencyHistory methods to public API
- make SomePtr implement Debug
- add internal logging for macros
- add constructor name flag to injectable macro
- improve macro error messages

### Performance Improvements
- reduce number of allocations in SynPathExt::to_string

### BREAKING CHANGE

The async DI container is no longer inside of a Arc. This affects AsyncBindingBuilder, AsyncBindingScopeConfigurator, AsyncBindingWhenConfigurator & AsyncInjectable

The blocking DI container is no longer inside of a Rc. This affects BindingBuilder, BindingScopeConfigurator, BindingWhenConfigurator & Injectable

The async DI container prelude module have been removed as it is no longer necessary seeing as the async DI container interface have been removed

IAsyncDIContainer have been removed and multiple structs no longer take a DI container generic parameter

The blocking DI container prelude module have been removed as it is no longer necessary seeing as the blocking DI container interface have been removed

IDIContainer have been removed and multiple structs no longer take a DI container generic parameter

The factory macro's async flag has been removed

The factory macro no longer
- Changes the return type to be inside of a TransientPtr
- Adds Send + Sync bounds when the threadsafe or the async flag is set
- Changes the return type be inside of a BoxFuture when the async flag is set

IDependencyHistory has been removed as part of an effort to simplify the API. This affects IDIContainer, DIContainer, IAsyncDIContainer, AsyncDIContainer, Injectable, AsyncInjectable, BindingBuilder, AsyncBindingBuilder, BindingScopeConfigurator, BindingWhenConfigurator, AsyncBindingScopeConfigurator, AsyncBindingWhenConfigurator and DependencyHistory

SomeThreadsafePtr has been removed and it's variants have been moved to SomePtr

The flag 'async' of the declare_interface macro has been renamed to 'threadsafe_sharable'. The reason being that the name 'async' was an outright lie. The new name describes exactly what the flag enables

The async_closure macro has been removed. This is because it is completely out of scope for this crate

SomeThreadsafePtrError has been removed and SomePtrError is now used by both the methods of SomePtr and of SomeThreadsafePtr

The methods of BindingBuilder, AsyncBuilder, BindingScopeConfigurator, AsyncBindingScopeConfigurator, BindingWhenConfigurator, AsyncBindingWhenConfigurator now take ownership of self


## v0.4.2 - 2022-11-28
### Bug Fixes
- allow for concrete type interfaces to be marked async
- make factories work again after Rust nightly-2022-11-07
- allow declaring a concrete type as it's own interface
- allow injectable macro flag arguments without a interface argument

### Build System/Dependency Changes
- bump versions to 0.4.2
- fix running macros tests on Rust stable

### Code Refactoring
- reorganize non-public API items
- use the async-lock crate instead of Tokio
- improve type param names, docs & more of casting
- fix some Clippy lints regarding format!()
- fix unused self clippy lint in blocking DI container
- improve cast error handling
- improve readability of cast functions

### Code Testing
- replace the test_util_macros crate with utility-macros
- add unit test for create_caster_fn_ident
- add unit tests for parsing injectable macro args
- remove some unused imports
- add unit tests for parsing declare_interface macro args
- add unit test for parsing factory type aliases
- make small improvements in the declare_default_factory macro args tests
- add unit tests for parsing factory macro args
- add unit tests for parsing declare_default_factory macro args
- split up cast unit tests into their respective modules

### Documentation Changes
- add a example to the crate root
- add terminology guide to readme
- add msrv
- add arguments for IoC & DI to readme
- add comments explaining the prevent-circular example


## v0.4.1 - 2022-10-30
### Bug Fixes
- remove unused Rust feature flag

### Build System/Dependency Changes
- bump versions to 0.4.1

### Code Refactoring
- add dependency history type
- improve injectable macro error messages
- add Debug implementations for castable factories
- rename DI container binding map to DI container storage
- reduce DI container coupling
- improve internals of macros & add unit tests
- reorganize DI containers
- remove relying on Rust nightly for better handling of features
- clarify binding builder to_factory signature
- improve management of feature specific items
- stop using the async_trait macro for AsyncInjectable
- shorten async binding builder trait bounds

### Code Testing
- add binding configurator unit tests
- add castable factory unit tests
- add provider unit tests
- add binding builder unit tests
- add more unit tests
- fix the can bind to factory unit tests

### Documentation Changes
- add sealed notices to DI container interfaces
- add coverage badge
- fix the example usage in the readme
- fix unresolved links to DI container types
- fix spelling mistakes in blocking & async DI containers
- add binding builder examples
- add async binding builder examples
- remove unnecessary feature notices

### BREAKING CHANGE

Binding builders & configurators now take dependency history type arguments, the DetectedCircular variant of InjectableError now contains a dependency history field & the injectable traits take dependency history instead of a Vec

You now have to import the DI containers's interfaces to use the DI containers's methods

DIContainer, AsyncDIContainer & the binding structs have been relocated


## v0.4.0 - 2022-10-01
### Bug Fixes
- prevent problems caused by non send + sync traits
- add missing semicolon in the factory macro

### Build System/Dependency Changes
- bump versions to 0.4.0
- add required features for the async-factory example
- improve async dependencies

### Chores
- remove repetition of allowing clippy::module_name_repetitions

### Code Refactoring
- remove unused import in DI container module
- remove IFactory from public API
- remove repetition of declaring factory interfaces
- reorganize modules in the macros crate
- make the async & non-async DI container bind methods must_use
- add put factory return types in TransientPtr automatically
- prevent look for default factory without factory feature
- make async DI container be used inside of a Arc
- make DI container be used inside of a Rc
- improve DI container cast errors
- remove braces from expected injectable macro input
- rename the factory macro flag 'async' to 'threadsafe'
- remove unused import of ItemTrait
- improve async DI container cast errors
- replace arc cast panic with an error
- limit FactoryPtr & AnyFactory to the factory feature
- make DI container have single get function
- move specifying binding scope to a binding scope configurator
- improve private method names & clean up InjectableImpl

### Code Testing
- move some factory function types to type aliases

### Documentation Changes
- add v0.4.0 to changelog
- improve item links in the injectable macro
- fix unresolved link to TransientPtr
- make IFood in async example Send + Sync
- fix ambiguous link to the factory macro
- add missing modules in the async example
- add async support to readme
- improve & add examples
- use anyhow in the unbound example
- correct the example in the readme
- add CI shield to readme
- update the DI container example
- remove license shield from readme

### Features
- add bind async default factories to async DI container
- add binding async factories to async DI container
- add factory macro async flag
- allow factories to access async DI container
- allow factories access to DI container
- add a threadsafe flag to the declare_default_factory macro
- implement async functionality
- implement named bindings

### Style Improvements
- add rustfmt config options

### BREAKING CHANGE

The to_default_factory method of the blocking and async DI containers now expect a function returning another function

The to_factory & to_default_factory methods of AsyncBindingBuilder now expects a function returning a factory function

The async DI container is to be used inside of a Arc & it also no longer implements Default

Factory types should now be written with the Fn trait instead of the IFactory trait and the to_factory & to_default_factory methods of BindingBuilder now expect a function returning a factory function

The DI container is to be used inside of a Rc & it also no longer implements Default

The injectable macro no longer expects braces around it's flags

FactoryPtr has been limited to the factory feature

The DI container get_singleton & get_factory functions have been replaced by the get function now returning a enum

Specifying the scope of a DI container binding is now done with a binding scope configurator


## v0.3.0 - 2022-08-21
### Bug Fixes
- make DI container get_factory calls in the injectable macro valid

### Build System/Dependency Changes
- bump versions to 0.3.0
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
- add v0.3.0 to changelog
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

