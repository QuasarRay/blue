# Blue Rust Reimplementation: Codex Goal Execution Contract

## 1. Activation, objective, and governing decisions

This is an execution contract for reimplementing Carbon Engine's Blue and BlueExposure together in Rust. Execute it when the user explicitly invokes it as an implementation Goal. Reading, reviewing, or authoring this document does not activate the Goal. Respect the current collaboration mode; this document does not override Plan mode or higher-priority tool instructions.

**Every original Blue-specific object, type, interface, method, property, and responsibility MUST have exactly one canonical Rust counterpart. Correspondence MUST preserve behavior, names, ownership, and relationships. A matching name, wrapper, registry entry, or passing compilation result alone does not establish equivalence.**

The Goal is complete only when both libraries have been reimplemented in one repository, with complete Linux functionality, this correspondence contract satisfied, deep and broad Bevy integration, modern Python concurrency by default, successful compilation, successful execution of the applicable original tests, and verified publication of completed milestones to `blue-rs-dev`.

The user's governing decisions are:

1. Behavioral, structural, and nominal equivalence are simultaneous requirements. Preserve the object model, struct hierarchy, trait hierarchy, method ownership, and the responsibilities of each original entity through explicit Rust mappings.
2. Modernize the implementation completely. Preserve observable Blue-specific behavior while replacing C++ machinery, GIL-dependent assumptions, legacy scheduling internals, and deprecated production macros with maintained native Rust and Python facilities.
3. Reimplement Blue and BlueExposure together, but do not recreate anything PyO3 already provides. Preserve Blue-specific semantics around PyO3 where they differ from generic Python binding behavior.
4. Use the newest stable Bevy release. Bevy must participate throughout the real runtime, vertically and horizontally.
5. Concurrent, parallel, asynchronous, multithreaded Python is the default architecture. It is not a later optional feature or a facade around a globally serialized interpreter.
6. Choose the fastest path to finishing the complete project. CGP, generators, macros, and direct handwritten Rust are permitted choices. No metaprogramming technique is mandatory when it slows verified completion.
7. Successful compilation and the original Blue and BlueExposure tests are the verification standard. Do not invent a new verification campaign. Additional testing requires a concrete reason encountered during implementation and a subsequent human decision.
8. Linux is this Goal's complete acceptance target. Record Windows and macOS obligations for later work without claiming those platforms are accepted. Linux-first does not permit omitting portable behavior originating in Windows-oriented source.
9. Publish verified milestones incrementally into `blue-rs-dev`. Leave `main` and the upstream BlueExposure repository unchanged.

These explicit decisions take precedence over conflicting project guidance, including any blanket requirement that CGP own all application logic. Preserve the user's existing AGENTS.MD edits and unrelated work. Do not rewrite project instructions merely to remove a constraint; record and apply the user's actual decisions.

## 2. Native Codex Goal lifecycle

Use Codex's native Goal state as the authority for lifecycle and budget accounting. Repository records store implementation progress and evidence; they do not replace the Goal service or create a competing lifecycle controller. The official [Using Goals in Codex guide](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex) describes persistent objectives, continuation, and evidence-based completion. The tool descriptions available in the executing session govern exact operations.

At activation:

- Call `get_goal` before attempting `create_goal`.
- Reuse a matching active Goal, including one already created by the user's `/goal` command. Do not create duplicate or milestone Goals for the same work.
- If no unfinished Goal exists and the user has explicitly invoked this execution contract, create the reimplementation Goal. Set its objective to the complete outcome in section 1 and reference this contract. Omit `token_budget` unless the user explicitly supplies a budget.
- If an unrelated unfinished Goal exists, do not replace it. Request the user's direction about that conflict.
- Preserve the complete objective across ordinary turns, continuation turns, compaction, and status questions. A user correction updates the contract and work queue; it does not silently erase unfinished obligations.

Apply the lifecycle rules exactly:

| Situation | Required action |
| --- | --- |
| A batch or milestone finishes | Checkpoint its actual result and continue the next useful work while the Goal remains active. |
| One feature awaits information | Record the exact dependency, request the missing information, and continue independent work. |
| The user explicitly requests a pause | Use `update_goal` with `status: "paused"`, report the returned status, and stop Goal work. |
| The same genuine blocking condition repeats for at least three consecutive Goal turns, including the original turn, and no meaningful independent work remains | Use `update_goal` with `status: "blocked"`; identify the evidence and the specific input or external change required. After a blocked Goal is resumed, start a fresh three-turn blocking audit. |
| A user or system budget limit is reached | Stop substantive work, preserve the checkpoint within the remaining allowance, and report unfinished work. Do not mark the Goal complete or change its budget yourself. |
| Every acceptance condition in section 16 is satisfied | Audit the evidence, then use `update_goal` with `status: "complete"`. Report tool-provided final token usage if this was a budgeted Goal. |

Only explicit user direction authorizes pausing. Difficulty, uncertainty, an incomplete feature, a pending local decision, or a short-lived tool failure is not by itself a blocked Goal. Do not bypass a budget, create replacement Goals to reset accounting, or start daemons and automation loops to imitate continuation. Make no claim of work continuing invisibly after the session stops.

## 3. Source authority and repository unification

Execution starts from these locations, revalidated against the actual checkout:

| Role | Location |
| --- | --- |
| Unified target repository | `/home/astral/Projects/blue` |
| Target remote | `https://github.com/QuasarRay/blue.git` |
| Integration branch | `blue-rs-dev` |
| Original Blue source | `src-cpp/`, `include/`, original Python modules, original tests, and associated source/configuration files in the target repository |
| Initial BlueExposure source | `/home/astral/Projects/blueexposure` |
| Imported BlueExposure reference | `reference/blueexposure/` in the unified target repository |

Authoring-time observations on 2026-11-02: Blue HEAD was `cdc466f778d16d8997006294cd17a6f559c53068`; BlueExposure HEAD was `086a6948fa29e568cbe10f239910cdef70defa61`. Blue had a minimal Rust scaffold and an existing AGENTS.MD edit. These observations are provenance hints, not permanent execution pins or evidence of migration progress. Re-read current state before acting.

1. Inspect branch, HEAD, status, remotes, existing migration artifacts, source changes, and available tools. Preserve dirty work and staged changes. Reuse existing work instead of resetting it or creating a competing implementation.
2. Record the chosen source revisions, repository origins, configuration identities, licenses, and content digests. Identify any source changes not represented by a commit. Resolve material source-authority conflicts explicitly instead of selecting whichever revision is easiest.
3. Preserve the Original Blue source and original tests as reference. Some existing CMake entries still refer to `src/` even though the C++ source was moved to `src-cpp/`; correct build infrastructure or extraction paths without confusing the Rust scaffold with the authoritative C++ implementation.
4. Import a revision-pinned BlueExposure snapshot into `reference/blueexposure/`, including original tests, relevant build metadata, license, and provenance. Exclude nested Git metadata. Import from the actual selected source, not from an assumed remote revision. Do not modify or push the sibling upstream checkout.
5. Use one Cargo workspace rooted in Blue. Keep a coherent public `blue` API, private implementations, and the Python exposure boundary. Add internal crates only where they simplify ownership, build boundaries, or delivery. Both implementations, their source mappings, and build infrastructure belong in this repository.
6. Remove production reliance on sibling checkout paths, a BlueExposure gitlink, or Original Blue/BlueExposure C++ bodies. Reference builds may support the original test infrastructure, but a passing original implementation is not evidence that Rust works.
7. Inventory external dependencies such as Carbon core, IO, scheduler, and host interfaces. Reuse maintained ecosystem facilities and implement the Blue-owned boundary contracts. Do not expand this Goal into wholesale ports of unrelated engines, or omit Blue behavior because a dependency is missing.

Source snapshots remain immutable reference inputs. Store human-approved test adaptations separately with their justification and original identity. Any necessary source/configuration update changes provenance and invalidates affected results.

## 4. Complete inventory and one correspondence record

Inspect both libraries beyond their public headers and existing tests. Cover handwritten bodies, inline definitions, templates and specializations, macro-expanded declarations, registration tables, Python helper modules, configuration branches, private helpers, and behavior embedded in exposure or lifecycle machinery. A list of exported names or discovered tests is not the complete implementation denominator.

Use the simplest reliable extraction approach available. Reuse compiler metadata or structured parsing when useful, but do not build a general C++-to-Rust compiler as a prerequisite. Manual review and handwritten Rust are allowed. Mechanical mapping means deterministic, explicit, machine-readable correspondence; it does not require every production body to be generated. Unresolved extraction remains visible and cannot receive completion credit.

Use one versioned rule set for identifier translation and relationship mapping. Apply it uniformly to the authoritative source inventory and the actual Rust declarations. Record necessary semantic choices as explicit inputs to those rules rather than unrelated per-file naming or hierarchy decisions. The resulting mapping must identify real types, traits, fields, and methods and their actual relationships; a manifest entry, comment, or alias to a generic placeholder cannot supply a missing counterpart. This is mapping infrastructure, not an additional behavioral test suite.

Maintain one canonical machine-readable record, reusing an existing equivalent if present. Otherwise use `migration/correspondence.json`. Each source identity must distinguish repository, qualified symbol, declaration kind, and canonical signature or specialization, with source revision, configuration, path, and span recorded as provenance. Do not use line numbers alone as symbol identity or collapse overloads and duplicate test definitions.

Record at least:

- Original qualified identifier, kind, signature, declaring owner, visibility, source references, and whether it belongs to production, a test fixture, or external infrastructure.
- Original bases, implemented interfaces, override and inherited-method relationships, fields or logical state, and the responsibility of each mapped item.
- Original Python-visible module/class/member names, registration identifiers, property behavior, and relevant construction/lifecycle rules.
- Canonical Rust qualified path, item kind, inherent or trait `impl` owner, translated signature, field/state owner, and linked structural relationships.
- Representation decisions, PyO3 or ecosystem replacements, thin adapters, and auxiliary Rust infrastructure. Keep the distinction between the canonical counterpart and supporting helpers explicit.
- Implementation status, relevant compilation and original-test receipts, unresolved questions, and exact human-approved exceptions.

Use explicit dispositions such as `BLUE_SPECIFIC`, `PYO3_PROVIDED`, `NATIVE_RUST_MECHANISM`, `EXTERNAL_BOUNDARY`, `PLATFORM_DEFERRED`, and `UNRESOLVED`. A disposition is not evidence of implementation. Mixed utilities retain their Blue-specific obligations even when PyO3 or Rust replaces their generic machinery. Never classify an item merely as "legacy" and discard it. No unresolved required item may remain at acceptance.

The correspondence record supports implementation and review. It is not a formal proof, a substitute for the original tests, or permission to introduce a separate custom parity test suite. Repeated mapping information in reports must be derived from this record rather than maintained as conflicting catalogs.

## 5. Mandatory behavioral, structural, and nominal mapping rules

**Every original Blue-specific object, type, interface, method, property, and responsibility MUST have exactly one canonical Rust counterpart. Preserve the complete object and responsibility graph, not just the exposed method names.**

Apply these mappings consistently:

| Original construct | Required Rust correspondence |
| --- | --- |
| Concrete class or struct | One canonical nominal Rust type carrying the corresponding responsibility and authoritative state. Preserve the original identifier wherever legal. |
| Interface or purely abstract contract | A correspondingly named trait; preserve its method set and parent-interface edges through supertraits. |
| Concrete base class | Explicit composition and delegation preserving base identity, inherited behavior, and state ownership. Mixed state-and-contract bases may use auxiliary traits whose role is recorded. |
| Implemented interface | An explicit implementation for the corresponding Rust type, with methods owned by their correct declaring interface and override path. |
| Inherent/private method | A corresponding method on the owning Rust type or its documented implementation boundary; preserve visibility intent and responsibility. |
| Inherited method | A link to the declaring base implementation and explicit forwarding where Rust requires it. Do not create an independently evolving duplicate body in every derived type. |
| Field or logical state variable | An explicit mapping to one Rust state owner. Native representations may replace pointer/allocation bookkeeping, with every affected source role accounted for. |
| Namespace, nested type, enum, constant, or registration identity | A deterministic qualified Rust representation and preserved externally observable identity/value. |
| Generic binding or C++ language mechanism | A documented PyO3/native Rust replacement; retain any Blue-specific policy and relationships attached to it. |

Rust `impl` blocks have no independent names or inheritance hierarchy. Their canonical identity is the self type plus implemented trait, or the inherent implementation owner. Preserve the source's method ownership and interface/base graph through those real Rust constructs. Do not fabricate named or inherited `impl` blocks. See the [Rust object-model documentation](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html).

Nominal mapping must be reversible through the record:

- Keep original class, trait, method, field, and registration spelling wherever valid Rust permits. Do not mechanically replace original public names with preferred Rust naming style. Limit naming-lint allowances to the correspondence surface.
- Use raw identifiers for Rust keywords where supported. Use a fixed documented escaping rule otherwise.
- Distinguish overloads and specializations using a deterministic signature-derived identifier and record the complete original signature. Resolve any collision explicitly; never silently merge overloads.
- Map constructors to Rust construction functions and destructors to lifecycle/`Drop` implementations, with explicit source links. Map operators to native traits or named methods as appropriate. Rust-required spelling differences must follow documented rules rather than arbitrary renaming.
- Preserve Python-visible names and callable/property contracts through PyO3's supported naming and signature facilities.
- Mark convenience APIs and asynchronous adapters as additional infrastructure. They do not replace, merge, or multiply canonical source counterparts.

Preserve exact Blue-specific values, units, conversions, error behavior, callback semantics, lifecycle effects, persistence formats, and ordering requirements. Native Rust ownership and synchronization replace raw-pointer tricks, manual C++ allocation, compiler ABI layout, vtable emulation, and undefined behavior. Do not reproduce C++ machinery merely to resemble the source. If a caller-visible requirement conflicts with safe modern implementation, report the concrete conflict and continue unrelated work; do not silently change the contract.

Source-grounded examples:

- `src-cpp/BlueMemStream.h` declares `MemStream`, not a class named `BlueMemStream`. Its canonical Rust type must preserve `MemStream` and both `IBlueMemStream` and `ICacheable` relationships. `IBlueMemStream` inherits `IBlueStream`; inherited stream interfaces and the original declaring owners of their methods must also be represented. One generic stream wrapper is insufficient.
- The BlueExposure `TestMultipleInterfaces` fixture implements `ITestInterfaceOne` and `ITestInterfaceTwo`. Both relationships and each method's owner must survive the adaptation around the original test.
- The BlueExposure `TestSuperclassMethods` fixture inherits from `TestMethods`. Retain the declaring base and inheritance relationship instead of flattening copied methods into an unrelated type.

Separate the public API from private implementation cleanly. Keep canonical public types and contracts stable while allowing internal replacement without gratuitous API redesign. Private source responsibilities remain mapped; hiding internals does not erase them. Do not move unrelated original responsibilities into a generic manager, untyped entity wrapper, or oversized service merely to simplify a dependency graph.

## 6. BlueExposure and PyO3 division of responsibility

Use maintained PyO3 facilities directly for generic Python object ownership, interpreter attachment, class/module exposure, method/property binding, reference handling, ordinary conversions, exceptions, and supported Python protocols. Do not recreate a second Python binding framework or duplicate PyO3-generated plumbing.

For each replaced BlueExposure utility, identify the concrete PyO3 facility and version that supplies its role. Keep an original-named thin adapter only when the name or Blue-specific policy is part of the required contract. A generic C++ template need not survive as an empty Rust wrapper merely for appearance.

Preserve the portions PyO3 does not supply as Blue-specific behavior: class and interface discovery, registration identities, object creation rules, identity across wrappers, Blue reflection metadata, inherited/interface method exposure, copying, persistence, special argument/result conversions, callbacks, weak-reference/lifetime policy, and error semantics wherever the source defines them.

Do not assume ordinary PyO3 conversions are identical to Blue conversions. Original tests include distinctions between booleans, integers, floats, strings, overflow, and exceptions. Retain the required policy around the reused primitive. Likewise, use PyO3 inheritance facilities only where they represent the required Python contract; C++ multiple-interface relationships are not automatically the same as Python class inheritance. Preserve the Rust interface graph and the actual Python-visible relationships without inventing unsupported PyO3 capabilities.

Document replacements in the correspondence record. `PYO3_PROVIDED` does not make a relevant original test disappear. If a test exercises machinery that cannot survive modernization unchanged, apply section 12's human-decision process.

## 7. Deep and broad Bevy integration

Bevy MUST own engine execution and state integration. A plugin that registers names and then calls an independent legacy runtime does not satisfy this contract. Bevy integration is part of each implementation batch from the first working object path onward.

Resolve the newest stable Bevy release from authoritative release/package information at execution start, pin the actual version and compatible Rust toolchain, and record the resolution date. Recheck before final acceptance and update if a newer stable release is required to satisfy the user's newest-release requirement. Pin within a working batch to avoid uncontrolled dependency churn. Bevy 0.20 was the release family verified during prompt authoring on 2026-11-02; it is not a permanent version requirement. See the [Bevy 0.20 announcement](https://bevy.org/news/bevy-0-20/).

Vertical integration must connect the real path from Python/Rust entry points through canonical Blue objects and interfaces to Bevy-managed state, scheduled work, resources, callbacks, and returned results. Horizontal integration must connect the applicable subsystems to one coherent runtime.

Cover at least the following where supported by original responsibilities:

- Application/plugin initialization, class/service registration, startup, pumping, and shutdown.
- ECS resources and components, object identity, lifecycle, authoritative state, and change visibility.
- Schedules and system ordering for clocks, timers, ticks, callbacks, notifications, and resource completion.
- Bevy task execution for suitable background IO and computation, integrated with the Python concurrency boundary.
- Asset/resource loading, caches, streams, background reads, completion notifications, and resource lifetime.
- Reflection/type metadata connected to Blue's canonical object model without a competing state model.
- Persistence/copying, time, statistics, logging, and diagnostics connected to the same object and resource identities.

Do not force every object into an entity if a typed resource or owned value is the correct mapping. Do not scatter a canonical source object's responsibilities into unrelated components without explicit ownership and relationship mappings. Preserve one authoritative owner for every mutable state value; avoid duplicate Python, Blue, and Bevy copies with ad hoc synchronization.

Use explicit system ordering only where required by observable Blue contracts. Permit independent systems and Python work to run in parallel. Preserve immediate versus queued completion: original `test_events.py` requires `SendEvent` to invoke its callback before returning, while `PostEvent` completes later. A delayed Bevy command cannot silently replace a synchronous source effect.

Support headless Linux operation without a window or GPU. Integrate applicable Bevy facilities broadly; do not add unrelated rendering/gameplay scope to inflate integration claims. Existing original tests must reach the production integration path, rather than a separate test-only runtime.

## 8. Free-threaded, concurrent, parallel, async Python by default

Select and pin a current stable free-threaded CPython build supported by the maintained PyO3 release in use. Record the interpreter executable, version, build configuration, PyO3 version, extension build mode, and actual runtime GIL state. The default installation and execution path must use that configuration, not silently select a conventional GIL-enabled interpreter.

Free-threaded capability and active GIL state are different. Include their identification in runtime/test-launch infrastructure, including state after required imports. An extension can re-enable the GIL; do not accept that silently as the default. Report the responsible dependency and fix or replace it. See [Python free-threading guidance](https://docs.python.org/3.15/howto/free-threading-python.html).

Design shared objects, registries, counters, caches, callbacks, cancellation state, and lifecycle operations for real concurrent access from the outset. Use native Rust ownership and appropriate synchronization. Do not serialize all Python execution under a substitute global lock. Legitimate ordering and exclusive Bevy world mutations must be scoped to their actual state requirements.

Apply the selected PyO3 release's documented threading APIs. Attach worker threads correctly when accessing Python, use owned Python references across worker boundaries, and never treat a Python attachment token as exclusive access to shared data. Do not use unsupported `Send`/`Sync` assertions or `unsendable` objects to disguise a concurrency design failure. PyO3's [free-threading guidance](https://pyo3.rs/v0.30.1/free-threading) is a reference; revalidate APIs against the version actually selected.

Integrate Python coroutines, worker threads, and Bevy work through maintained facilities and explicit command/result ownership. Bevy owns engine scheduling; Python event loops own their coroutines. Cross-thread submissions and completions must respect loop ownership. Do not invent a second general executor if an existing maintained bridge suffices. Async concurrency alone is not parallel Python execution. See [Python asyncio threading guidance](https://docs.python.org/3.15/library/asyncio-threading.html).

Define operational behavior for submission, completion, cancellation, errors, callback dispatch, and shutdown. In particular:

- Do not hold a Bevy world borrow, Python borrow, or internal lock across an asynchronous suspension or reentrant callback where doing so violates ownership or can deadlock.
- Preserve the original synchronous method contracts. An async adapter is an additional interface with an explicit relationship to the canonical method, not permission to change its return type or timing contract.
- Avoid blocking a thread that must make progress to complete the operation being awaited. Release/detach resources according to the selected APIs around blocking work and callback boundaries.
- Propagate errors and cancellation through the correct Blue/Python contracts. Account for pending jobs and callbacks during shutdown instead of losing them or running against destroyed state.
- Replace unsafe global counters and interpreter-thread assumptions, including task identity and task-local context bookkeeping, while preserving their required externally observable behavior.

Replace legacy Stackless implementation machinery with modern Rust/Python execution mechanisms while retaining Blue-facing task identities, suspension/resumption contracts, timing, callbacks, context, and errors where required. Free-threaded Python is not by itself a replacement for Stackless tasklet semantics. If a concrete original requirement cannot be preserved by the modern implementation, present that conflict to the human supervisor rather than pretending it disappeared with the GIL.

Preserve source-defined serialization behavior, including legacy data compatibility where it remains part of Blue's contract. Supporting an old data format does not require shipping an old interpreter. Do not add Python 2 runtime support merely because the source has a `PY27_COMPATIBILITY_MODE` data path.

## 9. Maintained macros and development speed

Remove deprecated macro dependencies from production code. Prefer maintained PyO3 attributes, Rust language features, maintained ecosystem libraries, or small maintained project macros. Original macro declarations may remain in immutable reference sources and original test infrastructure; they must not become the production implementation.

BlueExposure's `include/BlueExposureMacrosDeprecated.h` is a concrete source to inventory, including its method/property mapping roles. Preserve required Blue semantics while replacing the mechanism. Renaming a deprecated macro or copying its obsolete internals is not modernization. Do not make becoming "mainstream" or gaining external adoption a project completion condition; choose the permitted maintained-replacement path.

CGP means Context-Generic Programming. It is optional. So are `proc_macro2`, `syn`/`quote`, attribute/derive/delegation macros, derive-deftly, comprende, better_comprehension, arcane, and larger extraction/generation frameworks. They are candidates, not a required dependency checklist.

Choose the approach that finishes the current verified feature batch fastest, including setup, debugging, compilation, and maintenance costs. Direct handwritten Rust is explicitly allowed. Avoid speculative frameworks, Python-like syntax for its own sake, and broad tool installation campaigns. Stop optional metaprogramming work when its cost exceeds the work it saves and use the simpler implementation. Never use this rule to reduce functionality or bypass required original tests.

For generated code, preserve reproducibility: version the input, mapping rules, and generator; regenerate intentionally; do not make production fixes only in output that the next generation will overwrite. Do not require generation for code that is faster and clearer to maintain directly.

## 10. Durable progress records and continuation loop

Reuse an existing equivalent migration record if present. Otherwise keep this minimal set under `migration/`:

- `correspondence.json`: canonical inventory, relationships, mappings, implementation status, and links to evidence/decisions.
- `work_queue.json`: one dependency-ordered queue referencing source identities, with the current batch, blockers, and next action.
- `CONTINUATION.md`: a concise current-state checkpoint that tells the next turn exactly where to resume.
- `receipts/`: compilation/original-test results and publication evidence referenced by the canonical records; not a separate acceptance database.

Keep source/dependency pins and human decisions in or linked from these records. Do not create parallel catalogs for Bevy, Python, exposure, and platform status. Generate views from the same source identities when useful.

At every continuation:

1. Read Goal state, branch/status, current pins, the checkpoint, and only the relevant queue/mapping slice. Reconcile changed files and user steering before relying on old results.
2. Select the next dependency-ready batch that advances complete behavior. Prefer completing a usable cross-layer path over creating many unrelated declarations.
3. Implement the mapped responsibilities and their real Bevy/Python integration. Keep public/private boundaries and canonical ownership intact.
4. Compile the affected production artifacts and run the relevant original tests. Fix diagnosed failures without weakening their assertions or changing the required behavior.
5. Record the implementation revision/content identity, commands, configuration, tool/dependency versions, exit codes, test selection/results, and remaining blockers. Publish a verified milestone when ready.
6. Update the queue and checkpoint with the exact next command or source responsibility. Continue while the Goal is active and useful work remains.

Checkpoint before a context handoff or budget boundary. Store durable facts in files rather than repeating large transcripts. Keep user updates concise and factual. A status request does not cancel the underlying implementation work.

Reuse successful results only while the relevant source, production code, dependencies, configuration, adapters, and tools remain current. When an identity changes, invalidate the affected results, not every unrelated receipt. Do not repeatedly run unchanged failing commands or repeat broad successful checks without a new reason. Report compilation, original tests, mapping completion, and publication separately.

## 11. Implementation sequence and complete subsystem coverage

Use dependency order without postponing Bevy or Python to the end:

1. Establish source authority, the unified workspace, initial correspondence rules, supported toolchain/interpreter configuration, and a buildable foundation.
2. Complete an initial integrated path covering object creation, identity, registration, an interface relationship, lifecycle, PyO3 exposure, and Bevy-managed state. Use relevant original tests as the validation surface.
3. Complete dependency-ready subsystem batches until all inventoried responsibilities are implemented. Each batch includes its own Python exposure and Bevy integration rather than deferring those obligations indefinitely.
4. Resolve remaining Linux obligations, run the complete applicable original suites/configurations, build/package the deliverables, and audit acceptance.

The inventory, not this illustrative list, defines full coverage: object/class metadata, discovery, interfaces and inheritance; copying and reference/lifecycle behavior; time, pumping, task scheduling and timers; callbacks and events; memory/file/network/remote streams; path handling and resource files; resource managers, caches and background loading; marshaling, black/dictionary/YAML persistence and structured rows; statistics, accumulators, logging, diagnostics, process/OS boundaries, and source-owned utility behavior.

Do not count declarations, registries, TODOs, unconditional success, empty callbacks, state-only facades, or delegates to missing functions as implementation. Optional feature flags must not hide incomplete required functionality. Platform-specific APIs require a documented Linux behavior or a concrete human decision, not an automatic omission based on a filename.

## 12. Original tests: preserve exact code and adapt infrastructure

**Use successful compilation and the original Blue and BlueExposure tests. Adapt infrastructure around those tests while preserving their exact code wherever possible. Do not add tests, rewrite assertions, weaken expectations, or remove failures merely to make the migration pass.**

Keep the original test files byte-preserved as reference, including their fixtures, data, expected values, and source identities. Treat exact test code as the default executable test material. Adapt build configuration, module search/loading, fixture registration, execution environment, Linux support, and narrowly scoped adapters around it. Adapters must invoke the actual Rust implementation and expose its real results; they may not supply expected answers or maintain a second implementation.

Original C++ tests may need infrastructure to reach Rust. Do not automatically translate their assertions into new Rust tests, rebuild a C++ production compatibility runtime, or count tests linked to Original Blue as tests of Rust. If they cannot run unchanged through a reasonable infrastructure adaptation, report the exact case and request a human decision.

For each concrete incompatibility, provide:

1. Original test identity, source path/span, assertion or dependency, and the command/configuration that exposes the issue.
2. The actual failure or code-level contradiction and the relevant Rust implementation path.
3. Whether this is an implementation defect, infrastructure gap, unavailable dependency, or a test that specifically requires obsolete behavior.
4. The smallest proposed resolution and its effect on correspondence and the accepted behavior.

Obtain explicit human permission before changing, removing, disabling, or replacing an original test. For example, a test that requires all Python execution to be single-threaded must be reported; it is not silently deleted when parallelism makes it fail. Preserve the original and record the exact approved exception. A PyO3 replacement, skipped platform, or legacy label does not itself authorize test retirement.

If a concrete scenario gives reason to believe compilation and the original tests are insufficient, explain that scenario and ask the human supervisor what additional verification to use. Do not preemptively add unit tests, parity suites, stress tests, fuzzing, mutation testing, differential campaigns, benchmarks, or formal-proof requirements. Continue independent work while a localized decision is pending.

Missing test dependencies do not justify weakening assertions or counting the tests as passed. Report the blocker and work on independent responsibilities. An original source bug or ambiguous expectation requires the same evidence-based discussion; do not fix the test merely because changing Rust is inconvenient.

## 13. Compilation, original-test accounting, and packaging

Determine the real build and original-test commands from the repository, then record them so another engineer can reproduce the result. Compile production Rust, the Bevy integration, Python extension/embedding entry points that the source requires, and necessary test adapters for Linux. Build the actual final package artifacts; `cargo check` alone does not establish a successfully built extension or release package.

Use the relevant original tests for each batch, then run the complete applicable suites and original configuration variants for final acceptance. Include source-defined data-compatibility modes where applicable. Resolve source/test discovery and runtime dependency differences without changing the test assertions. Do not demand unrelated new platform matrices outside this Linux Goal.

Record test identities using repository, source path, fixture/class, test name, and any parameter/configuration identity. Preserve a source span or occurrence discriminator when definitions collide. Distinguish static source definitions from cases actually discovered and executed by the original runner; shadowed Python definitions must not be credited as two executed tests.

Report `passed`, `failed`, `skipped`, `undiscovered`, `blocked`, and `human-approved exception` separately. Existing conditional skips remain visible. Do not introduce new skips or omit a required configuration to manufacture acceptance. Original platform-only cases may be assigned to the recorded future platform scope with a source-backed reason; portable assertions inside such cases still need a disposition.

Bind results to source and implementation identity, dependency/tool versions, interpreter/build mode, runtime GIL state, adapters, test selection, and configuration. Failed, interrupted, stale, or truncated execution does not receive a successful receipt. Environment identification and mapping records are infrastructure/provenance, not a new behavioral test suite.

Package the unified Linux Rust library/runtime and Python deliverables locally with their required licenses, build instructions, public API documentation, and maintained dependencies. Do not publish packages to registries or unrelated deployment services without authorization. GitHub milestone publication is authorized separately in section 14.

Compilation and passing original tests are the chosen verification standard. Report their results accurately and their actual coverage. Do not call them formal proof or claim they demonstrate every possible concurrency interleaving. The complete implementation and correspondence requirements still apply to behavior not directly exercised by a test.

## 14. Incremental GitHub milestone publication

Use `/home/astral/Projects/blue` and its authorized `QuasarRay/blue` remote. `blue-rs-dev` is the integration branch. Leave `main` untouched and never publish to the upstream BlueExposure repository.

Prefer scoped, verified milestone commits and pushes to `blue-rs-dev`. If protections require pull requests, create milestone branches such as `codex/blue-<milestone>` from the appropriate development base, target `blue-rs-dev`, and merge after required checks. Maintain a clear stacked dependency order where milestones depend on unmerged work. Do not use a PR to bypass branch protection or review requirements.

Check remote identity, credentials, branch protections, and Git metadata writability early. Preserve unrelated dirty/staged files and the user's AGENTS.MD edits; stage only intended milestone changes. Do not force-push, rewrite unrelated history, or remove user work to simplify publication.

Publication and necessary PR communication for these milestones are authorized when this execution contract is invoked. Do not ask for the same ordinary milestone permission repeatedly. An actual approval/tool-policy restriction or unavailable credential must be reported as that specific restriction, not disguised as a completed push.

Verify remote branch/commit state and merged PR state where applicable. Separately verify the visible checkout is on the intended development branch with the expected content. A local commit, patch, bundle, open PR, or attempted push does not equal a published and integrated milestone. If publication is blocked, preserve local work and continue useful implementation; full Goal acceptance still requires the publication outcome.

## 15. Concrete conflicts and human decisions

Ask only for missing information, approval explicitly required by this contract, or a concrete conflict that cannot be resolved within the existing authorization. Do not repeatedly ask for routine choices already settled here.

Escalate exact conflicts such as an original test requiring obsolete behavior, an observable source contract incompatible with the modern runtime, a source-authority disagreement, or missing external access. State the source/test identity, evidence, attempted approaches, smallest proposed change, consequences, and independent work that can continue.

Record decisions with the user's actual approval and affected identities. A pending question is not an approval. Do not turn the general instruction to discard legacy implementation burden into permission to discard Blue behavior, objects, interfaces, or tests. If the user authorizes a behavioral exception, preserve its history and describe the final result as parity subject to that named exception.

## 16. Final acceptance and completion report

Before completing the Goal, audit current files, correspondence records, original-test results, build artifacts, dependency pins, and remote publication against all of the following:

1. Both Blue and BlueExposure are implemented in the unified repository. Production builds do not require the sibling checkout or Original Blue/BlueExposure C++ implementation.
2. Every required original Blue-specific object, type, interface, method, property, and responsibility has exactly one canonical Rust counterpart. Behavioral, structural, and nominal correspondence are all satisfied, including names, complete base/interface relationships, method/`impl` ownership, state ownership, and public/private boundaries.
3. The complete source inventory has a justified disposition. No unresolved required source body, relationship, configuration, or responsibility remains. Every PyO3/native-Rust replacement is explicit and retains any attached Blue-specific behavior.
4. Bevy integration is operational vertically and horizontally throughout the relevant runtime. State authority, scheduling, resources, callbacks, and Python access use the real production path, with headless Linux support.
5. The selected Bevy version satisfies the newest-stable requirement at the final version check. Maintained, compatible Rust, CPython, and PyO3 versions are pinned and reproducible.
6. Free-threaded Python is the default runtime, with concurrent, parallel, async, multithreaded operation supported by the implementation. Required imports do not silently restore GIL-dependent operation. Synchronous source contracts and required callback ordering remain intact.
7. No required implementation is a placeholder, omitted method, empty callback, hidden disabled feature, or dependency on deprecated production macros. Modernization has not silently removed legacy data or scheduling behavior that remains part of the contract.
8. Production compilation and packaging succeed on Linux, and all applicable original suites/configurations succeed, subject only to individually documented human-approved exceptions. Required skipped, undiscovered, blocked, or stale results do not count as passes.
9. Completed milestones are published and integrated into `blue-rs-dev`, with current remote evidence. `main`, the upstream BlueExposure repository, and unrelated user work remain preserved.
10. The work queue and continuation record agree with the actual result. Windows/macOS obligations and human-approved exceptions are accurately separated from this Linux milestone; no broader cross-platform completion is claimed.

If any required condition is unmet, continue implementation or follow the genuine blocker policy. Do not declare completion merely because the project compiles, a subset of original tests passes, a budget is nearly exhausted, or the document is long enough.

On successful completion, call the native Goal completion tool and provide a concise report containing the implemented scope, structural/nominal mapping disposition, selected versions and Python runtime mode, compilation and original-test results with receipt locations, exact approved exceptions, published commits/PRs and visible branch, future platform obligations, and tool-reported usage when budgeted. Describe evidence as implementation review, successful compilation, and original-test results; never as formal proof.

## Launcher

The accompanying `BLUE_RS_REIMPLEMENTATION_GOAL.txt` contains this one-line command. Invoking it starts the implementation objective; authoring these files does not.

```text
/goal Reimplement Carbon Engine's Blue and BlueExposure together in Rust in /home/astral/Projects/blue, following /home/astral/Projects/blue/BLUE_RS_REIMPLEMENTATION_GOAL_PROMPT.md in full. Complete Linux parity with behavioral, structural, and nominal one-to-one correspondence, deep and broad integration with the newest stable Bevy, and free-threaded concurrent, parallel, async, multithreaded Python by default. Reuse PyO3, preserve observable Blue behavior, choose development tools for speed, compile production artifacts, pass the applicable original tests without unauthorized test changes, and publish verified milestones into blue-rs-dev. Continue through the contract's acceptance audit; record exact blockers and human-approved exceptions, and mark complete only when the full objective is satisfied.
```
