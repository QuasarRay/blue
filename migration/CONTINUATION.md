# Blue Rust continuation

The native Goal is active. Its attachment reference resolves to the complete objective in BLUE_RS_REIMPLEMENTATION_GOAL_PROMPT.md, byte-identical to the activation attachment. Reuse it; do not create a duplicate or mark it complete for a batch.

## Source authority and checkout

- Blue reference revision: 2b920547091a25dd3647099c654c66ead25155aa. Initial checkout was clean. Original src-cpp/, include/, Python modules and tests remain unchanged.
- BlueExposure reference: 086a6948fa29e568cbe10f239910cdef70defa61, imported by git archive into reference/blueexposure without nested Git metadata. Source files are immutable. The sibling upstream checkout remains untouched.
- Integration branch: blue-rs-dev; main must remain 713f0ee394f12da2f8973f42e51a371ea60360d9.
- Published foundation: 8bae0e237d3a0017efe6afa3aac8519db80ef3c8, tree 38fd15a9f46138ec36c0e58fb64cbae401d703aa. Remote and visible checkout were verified. Published event milestone: 4c9903e818935036e97486dce2026641346b48ff, tree 1a609946b48965a1f87d4a722b58a553f452fa24. Remote and visible checkout were verified. Inspect current HEAD and publication receipts before acting.
- Shell git has no HTTPS credentials. The connected GitHub Git Data API works. Publication is authorized: create_tree against current remote base, require equality to local git write-tree, create_commit, then update_ref with expected_sha and force=false. Fetch and fast-forward the visible checkout. Branch is unprotected; repository rulesets were empty at preflight. Do not force-push or touch main/upstream BlueExposure.

## Current implementation and evidence

Pinned Rust 1.98.1, Bevy 0.20.1 (newest stable verified 2026-11-02), PyO3 0.30.1, async-channel 2.5.0 and free-threaded CPython 3.15.2. uv 0.13.6 installs python-build-standalone 20261026 locally; tools/bootstrap.py verifies the pinned uv archive. System /usr/bin/python3 is conventional 3.14.8 and serves only as a launcher. Default extension import and test infrastructure require Py_GIL_DISABLED=1 and runtime GIL=false.

Production paths now include canonical MemStream with named IBlueMemStream/IBlueStream/ICacheable relationships and explicit Carbon-core boundaries; one canonical state behind each Bevy object handle; registration/live counts; initial BlueWrapper/PythonKlass decoration; actual BlueTestEvents; BlueOS reentrancy and pumping; and BluePyOS/Synchro task/event work. Python callbacks never run under world/object locks. Native synchronous tasks have their own OS stacks, admitted by Bevy, with async-channel completions retained on Bevy tasks. Yield suspends until a subsequent pump without exhausting Bevy's bounded async pool. SendEvent runs immediately with a block trap; PostEvent queues the actual callback. Finalizers run after world membership removal and after releasing world locks.

Event milestone GitHub CI: https://github.com/QuasarRay/blue/actions/runs/41393864893 (success). Documentation/progress-only commits are excluded from runtime CI triggers to avoid repeating unchanged checks.

Latest production release build: migration/receipts/20261102T052030923694Z-build.json. Latest unchanged original tests: migration/receipts/20261102T052151226455Z-original-tests.json. All six cases pass (4 test_memstream.py, 2 test_events.py) against that optimized extension. GIL disabled before imports, after imports, and after execution. Earlier receipts are historical; use implementation content hashes before reuse. No extra behavioral tests, stress/parity/proof campaigns, or original-test edits were introduced. Linux package/wheel delivery and complete-suite execution remain pending.

Reproduce the current batch:

```sh
python3 tools/bootstrap.py
python3 tools/build.py --offline --release
python3 tools/run_original_tests.py tests/python/bluetests/test/test_memstream.py tests/python/bluetests/test/test_events.py
```

## Required next work

1. Event milestone is published and its publication receipt is recorded. GitHub CI run 41393864893 completed successfully, including the build, unchanged original tests, and guidance synchronization; inspect remote and visible checkout before choosing the next publication parent. Preserve all current work.
2. Complete the registry/object-model batch. Read src-cpp/BlueTestHelpers.h and .cpp and _Blue.cpp plus original test_blue.py and test_blueexposure.py. BlueTestHelperAttributes has string/unicode/bool/i32/u32/f32/f64/i64/u64/shared-string fields and typed vector/dict/structure-list state; implement its real responsibilities, not a count-only helper. Original test_blue.py is not credited yet. Map complete IBlueClasses methods, registration IDs/interfaces, ownership, wrapper identity and native construction policies.
3. Complete original BlueExposure fixtures with real TestMultipleInterfaces traits and TestSuperclassMethods composition/inherited ownership. Imported Python tests must remain exact. Keep fixture C++ definitions as authority; do not translate Python expectations into production answers.
4. Continue the full source inventory and dependency-ready subsystems. correspondence.json inventories 459 selected files and 547 static Python test definitions, including three shadowed definitions. Full C++ declaration/body/macro/configuration/relationship extraction remains UNRESOLVED. The 50 current mapping entries are a partial reviewed slice, not a complete denominator or full parity credit.

Broad type/method gaps remain explicit: full BlueOS/IBlueOS and BluePyOS/IBluePython graphs; complete decorator hooks, protocols, persistence and native identity; source task context labels, cancellation, shutdown and asyncio-loop bridging; all streams/resources/persistence/time/stats/OS/network/diagnostic utilities; original C++ test infrastructure; complete Linux packaging and original suite/configuration runs. Do not claim full concurrent/async functionality merely from the present worker architecture or six tests.

## Governing decisions

The execution contract makes CGP/metaprogramming optional; original tests plus compilation are the chosen verification standard; new behavioral tests and original-test changes require an explicit human decision. No such exception has been approved. Linux is the complete current target; Windows/macOS obligations remain recorded. Safe Rust replaces C++ allocator/pointer/undefined behavior. Root AGENTS.MD remains byte-preserved. tools/sync_guidance.py mirrors exact text through owned directories, with CI automatic updates; immutable BlueExposure inputs, external gitlinks and build/tool caches are excluded under the source-preservation requirements.

No human decision is currently pending and there is no global blocker. Check Goal, git status, source pins and the current queue at every continuation, then continue dependency-ready implementation. The Goal remains incomplete.
