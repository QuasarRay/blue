# AGENTS.MD

## Project Goals:

1. Reimplement Carbon Engine's Blue in Rust on top of Bevy.

2. There should be precise one to one correspondence between Original blue objects, methods, etc. and the Rust equivalents. struct names, struct hierarchy, impl block hierarchy and names, trait names, and hierarchy, etc. must be mechanically mapped one to one with the original. Therefore one to one equivalence is simultaneously Behavioral, structural, and nominal.

3. Both Blue Exposure and Blue must be unified into a single repository. So you will be reimplementing both, but do not recreate anything that PyO3 already provides.

4. AGGRESSIVELY use Metaprogramming and Automation to accelerate and simplify development. AGGRESSIVELY use CGP-Oriented metaprogramming, and metaprogramming with proc macro 2, proc macro 2 attribute, and proc macro 2 derive and proc macro 2 delegate, derive deftly and arcane macros to Automate, simplify and accelerate development.

5. Use CGP and derive-deftly and comprende and better_comprehension, proc macro 2, proc macro 2 attribute, and proc macro 2 derive and proc macro 2 delegate and arcane macros AGGRESSIVELY, to accelerate development and reduce boilerplate, and make the project look Pythonic and familiar to python developers: https://github.com/contextgeneric/cgp.git, https://gitlab.torproject.org/Diziet/rust-derive-deftly, https://github.com/QuasarRay/comprende.git, https://github.com/Natural-selection1/better-comprehension-in-rust.git, https://github.com/arcane-rs/arcane, CGP should own the application logic. The project should be designed around CGP-Oriented programming paradigm and design philosophy.

6. This AGENTS.MD file must exist in every directory of this project (The exact text, not an instruction to the agent to go and read the original. With each update in the root directory AGENTS.MD, the AGENTS.MD files in all other directories must be automatically updated. make it part of the CI/CD pipelines)

7. Do not modify main branch. always work in blue-rs-dev branch.

8. Make stackable pull requests and/or pushes incrementally to ensure that no work is duplicated or lost. turn my github account into your workspace. merge into Blue-rs-dev after each milestone.

9. The purpose of Metaprogramming is to accelerate development, if metaprogramming slowed you down, don't do it. Always choose the faster path for finishing the project.

10. Existing Rust Ecosystem must be used to the fullest potential. Do NOT try to recreate C++ semantics in Blue's Rust reimplementation. Application behavior and semantics must only correspond to original Blue exclusively in Blue-specific semantics, all traces of C++ semantics must be fully removed and replaced with Rust Semantics in Rust reimplementation of Blue.

11. The Rust Reimplementation of Blue MUST have Deep and Broad, Vertical and Horizontal integration with bevy. (Use newest release of Bevy)

12. The original blue does not cleanly separate the module of public api and the private internals implemenetation. The Rust Reimplementation must fix that flaw and cleanly and surgically separate Blue public API and private internals. Also, CGP must be used to ensure that internal changes can happen effortlessly with ZERO need for API Redesign. ALso, The architecture of Blue-rs must be redesigned around concurrent, async, parallel, and multi-threaded python. Also, some of original blue macros blue relies on are deprecated, either bring them back and modernize them, or replace them, depending on which one is more efficient.


Target Project Repo: https://github.com/QuasarRay/blue.git
