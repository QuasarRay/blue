#!/usr/bin/env python3
"""Refresh source provenance and static original-test identities.

This is inventory infrastructure, not a parity test. C++ body/relationship
extraction remains explicitly unresolved until reviewed; lexical candidates
never receive implementation credit.
"""
import ast
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECORD = ROOT / 'migration/correspondence.json'
BLUE_REV = '2b920547091a25dd3647099c654c66ead25155aa'
EXPOSURE_REV = '086a6948fa29e568cbe10f239910cdef70defa61'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def source_paths():
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BLUE_REV], cwd=ROOT, text=True).splitlines()
    for name in paths:
        p = Path(name)
        if p.parts[0] in {'src-cpp', 'include', 'python', 'python_package', 'tests', 'cmake'} or name in {'CMakeLists.txt', 'CMakePresets.json', 'vcpkg.json', 'vcpkg-configuration.json', 'LICENSE.md', 'NOTICE.md', 'README.md', 'carbon-blueConfig.cmake'}:
            yield 'blue', BLUE_REV, name
    for p in sorted((ROOT / 'reference/blueexposure').rglob('*')):
        if p.is_file():
            yield 'blueexposure', EXPOSURE_REV, p.relative_to(ROOT).as_posix()


def test_records(repo, path, text):
    if not path.endswith('.py') or '/test' not in path:
        return []
    try:
        tree = ast.parse(text, path)
    except SyntaxError as e:
        return [{'id': f'{repo}::{path}::discovery-error', 'kind': 'test_discovery_error', 'status': 'blocked', 'error': str(e)}]
    records = []
    for cls in (n for n in ast.walk(tree) if isinstance(n, ast.ClassDef)):
        occurrences = {}
        for fn in cls.body:
            if isinstance(fn, (ast.FunctionDef, ast.AsyncFunctionDef)) and fn.name.startswith('test'):
                occurrences[fn.name] = occurrences.get(fn.name, 0) + 1
                occurrence = occurrences[fn.name]
                records.append({'id': f'{repo}::{path}::{cls.name}::{fn.name}::{occurrence}', 'kind': 'original_python_test', 'repository': repo, 'path': path, 'fixture': cls.name, 'name': fn.name, 'occurrence': occurrence, 'span': [fn.lineno, fn.end_lineno], 'status': 'undiscovered', 'receipts': []})
        for record in records:
            if record['fixture'] == cls.name:
                record['shadowed'] = record['occurrence'] < occurrences[record['name']]
    return records


def main():
    data = json.loads(RECORD.read_text()) if RECORD.exists() else {'format_version': 1, 'entities': [], 'decisions': [], 'receipts': []}
    old_files = {f['id']: f for f in data.get('source_files', [])}
    old_tests = {t['id']: t for t in data.get('original_tests', [])}
    files, tests = [], []
    for repo, revision, name in source_paths():
        content = (ROOT / name).read_bytes()
        sha = digest(content)
        identity = f'{repo}::{name}'
        old = old_files.get(identity, {})
        role = 'test' if '/tests/' in '/' + name else 'production' if Path(name).suffix in {'.h', '.cpp', '.mm', '.py'} else 'infrastructure'
        files.append({'id': identity, 'repository': repo, 'revision': revision, 'path': name, 'sha256': sha, 'role': role, 'configuration': 'linux; BLUE_WITH_PYTHON=1; PY27_COMPATIBILITY_MODE pending', 'review_status': old.get('review_status', 'unreviewed') if old.get('sha256') == sha else 'unreviewed', 'extraction_status': old.get('extraction_status', 'unresolved; declarations, bodies, macros and configuration branches require review') if old.get('sha256') == sha else 'unresolved; declarations, bodies, macros and configuration branches require review'})
        for record in test_records(repo, name, content.decode('utf-8', errors='replace')):
            previous = old_tests.get(record['id'], {})
            if old.get('sha256') == sha:
                for key in ['status', 'receipts', 'runner_id']:
                    if key in previous: record[key] = previous[key]
            tests.append(record)
    data['source_files'] = files
    data['original_tests'] = tests
    data['source_pins'] = {
        'blue': {'revision': BLUE_REV, 'origin': 'https://github.com/QuasarRay/blue.git', 'initial_dirty': False, 'snapshot': 'preserved source paths in repository', 'license': 'LICENSE.md'},
        'blueexposure': {'revision': EXPOSURE_REV, 'origin': 'https://github.com/carbonengine/blueexposure.git', 'initial_dirty': False, 'snapshot': 'reference/blueexposure', 'license': 'reference/blueexposure/LICENSE.md'},
    }
    data['inventory_status'] = 'All selected files inventoried; full C++ declarations/body/relationship extraction is incomplete. File coverage is not correspondence completion.'
    data['source_digest'] = digest(json.dumps([(f['id'], f['sha256']) for f in files], separators=(',', ':')).encode())
    data['mapping_rules'] = {
        'version': 1,
        'source_identity': 'repository + qualified identifier + kind + full canonical signature; occurrence for duplicate tests; spans are provenance only',
        'identifiers': 'Preserve original spelling. Rust keyword: raw identifier where legal; otherwise append __blue. Overload: original name + __sig_ + first 12 hex of SHA256(canonical signature). Constructors: new; destructors: Drop/native field destruction.',
        'interfaces': 'Named supertraits, with methods on the actual declaring trait. Concrete bases: composition with explicit delegation.',
        'canonical_versus_adapters': 'Canonical Rust types are public source counterparts. PyO3 adapter names and ObjectHandle are auxiliary infrastructure, not extra canonical counterparts.',
        'replacement_credit': 'PyO3/native replacement requires concrete facility, version, retained Blue policy, source links and applicable original-test receipts.',
    }
    RECORD.write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps({'source_files': len(files), 'static_python_test_definitions': len(tests), 'shadowed_definitions': sum(t.get('shadowed', False) for t in tests), 'source_digest': data['source_digest'], 'complete_correspondence': False}, indent=2))


if __name__ == '__main__':
    main()
