# Rust implementation on Linux

This migration is in progress. See migration/CONTINUATION.md and the canonical
migration/correspondence.json for implemented scope and remaining obligations.
The original Blue C++ source and tests remain reference inputs. BlueExposure's
exact selected snapshot is under reference/blueexposure; production builds have
no dependency on that C++ implementation or on the sibling checkout.

The selected batch uses Rust 1.98.1, Bevy 0.20.1 and PyO3 0.30.1. The default
Python path uses CPython 3.15.2's free-threaded build with the GIL disabled.
The extension rejects a conventional interpreter or a runtime with an active GIL.

Provision pinned uv 0.13.6 and the free-threaded interpreter locally:

```sh
python3 tools/bootstrap.py
```

The local uv binary is a development tool, not a production dependency. An
equivalent separately provisioned free-threaded interpreter may be selected
with BLUE_PYTHON. Build and run the unchanged original memory-stream tests:

```sh
python3 tools/build.py --offline
python3 tools/run_original_tests.py tests/python/bluetests/test/test_memstream.py
python3 tools/run_original_tests.py tests/python/bluetests/test/test_events.py
```

Omit --offline when fetching the pinned Cargo dependencies for the first time.
Use tools/build.py --release for optimized library and extension artifacts.
Build and original-test receipts include content hashes, complete command output,
interpreter configuration, GIL state and exact original-test identities. These
receipts credit only their actual batch; they are not full migration acceptance.

BlueRuntime and BluePlugin own the real headless Bevy world. A Python MemStream
is a handle to its canonical Rust MemStream stored in that world. World locks
cover membership/lookup; byte operations lock only that object. PyO3 attachment
is released around native operations. No window or GPU is required.

Regenerate source provenance with tools/inventory.py and the reviewed stream
mapping with tools/record_stream_mapping.py. Neither operation is a behavioral
test. Unreviewed C++ bodies remain visible. Root AGENTS.MD is preserved and
tools/sync_guidance.py mirrors it into owned source directories, excluding
immutable reference inputs, external gitlinks and tool/build caches.

Original licenses are LICENSE.md and reference/blueexposure/LICENSE.md, with
additional notices in NOTICE.md. Windows and macOS support are future work;
portable source behavior remains part of the Linux migration.
