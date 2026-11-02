#!/usr/bin/env python3
"""Load exact original Python test files against the built Rust extension.

Only loading/environment infrastructure is adapted. No assertions, fixtures or
expected results are translated or rewritten.
"""
import argparse
import importlib
import io
import json
import os
import sys
import unittest
from pathlib import Path
from evidence import ROOT, identity, python_path, require_free_threaded, save_receipt, sha


class RecordedResult(unittest.TextTestResult):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.cases = []
    def addSuccess(self, test):
        super().addSuccess(test)
        self.cases.append({'runner_id': test.id(), 'status': 'passed'})
    def addFailure(self, test, err):
        super().addFailure(test, err)
        self.cases.append({'runner_id': test.id(), 'status': 'failed'})
    def addError(self, test, err):
        super().addError(test, err)
        self.cases.append({'runner_id': test.id(), 'status': 'blocked' if isinstance(test, unittest.loader._FailedTest) else 'failed'})
    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        self.cases.append({'runner_id': test.id(), 'status': 'skipped', 'reason': reason})


def main():
    python = python_path()
    if Path(sys.executable).resolve() != python:
        os.execv(str(python), [str(python), str(Path(__file__).resolve()), *sys.argv[1:]])
    parser = argparse.ArgumentParser()
    parser.add_argument('paths', nargs='+', help='Original test files, relative to the repository')
    args = parser.parse_args()
    before = require_free_threaded()
    sys.path[:0] = [str(ROOT / '.runtime'), str(ROOT / 'tests/python'), str(ROOT / 'reference/blueexposure/tests/python'), str(ROOT / 'python')]
    blue = importlib.import_module('blue')
    extension = Path(blue.__file__).resolve()
    if extension.parent != ROOT / '.runtime': raise RuntimeError(f'Unexpected Blue implementation loaded: {extension}')
    suite = unittest.TestSuite()
    source_files = {}
    mapping = json.loads((ROOT / 'migration/correspondence.json').read_text())
    selected = []
    for value in args.paths:
        path = (ROOT / value).resolve()
        relative = path.relative_to(ROOT).as_posix()
        file_record = next((f for f in mapping['source_files'] if f['path'] == relative and f['role'] == 'test'), None)
        if not file_record or sha(path) != file_record['sha256']: raise RuntimeError(f'Original test provenance mismatch: {relative}')
        base = ROOT / ('reference/blueexposure/tests/python' if relative.startswith('reference/') else 'tests/python')
        module_name = '.'.join(path.relative_to(base).with_suffix('').parts)
        selected.append((relative, module_name))
        source_files[relative] = sha(path)
        suite.addTests(unittest.defaultTestLoader.loadTestsFromName(module_name))
    after_imports = require_free_threaded()
    output = io.StringIO()
    result = unittest.TextTestRunner(stream=output, verbosity=2, resultclass=RecordedResult).run(suite)
    print(output.getvalue(), end='')
    after_run = require_free_threaded()
    for case in result.cases:
        for relative, module_name in selected:
            if case['runner_id'].startswith(module_name + '.'):
                fixture, name = case['runner_id'][len(module_name) + 1:].rsplit('.', 1)
                candidates = [t for t in mapping['original_tests'] if t.get('path') == relative and t.get('fixture') == fixture and t.get('name') == name and not t.get('shadowed')]
                if len(candidates) == 1: case['source_id'] = candidates[0]['id']
    path = save_receipt('original-tests', {'command': [str(python), 'tools/run_original_tests.py', *args.paths], 'implementation': identity(), 'source_files': source_files, 'extension': {'path': str(extension.relative_to(ROOT)), 'sha256': sha(extension)}, 'python_before': before, 'python_after_imports': after_imports, 'python_after_run': after_run, 'cases': result.cases, 'tests_run': result.testsRun, 'exit_code': 0 if result.wasSuccessful() else 1, 'output': output.getvalue()})
    for case in result.cases:
        for record in mapping['original_tests']:
            if record['id'] == case.get('source_id'):
                record.update(status=case['status'], runner_id=case['runner_id'])
                record['receipts'].append(path.relative_to(ROOT).as_posix())
    mapping['receipts'].append(path.relative_to(ROOT).as_posix())
    (ROOT / 'migration/correspondence.json').write_text(json.dumps(mapping, indent=2) + '\n')
    return 0 if result.wasSuccessful() else 1


if __name__ == '__main__': sys.exit(main())
