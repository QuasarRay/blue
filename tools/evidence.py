"""Shared source/build identity and free-threaded interpreter selection."""
import hashlib
import json
import os
import subprocess
import sys
import sysconfig
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECEIPTS = ROOT / 'migration/receipts'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity():
    paths = [ROOT / n for n in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.python-version']]
    paths += sorted((ROOT / 'src').rglob('*.rs'))
    paths += [ROOT / 'tools' / n for n in ['evidence.py', 'build.py', 'run_original_tests.py']]
    files = {p.relative_to(ROOT).as_posix(): sha(p) for p in paths if p.exists()}
    return {'sha256': hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest(), 'files': files, 'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()}


def python_path():
    configured = os.environ.get('BLUE_PYTHON')
    if configured: return Path(configured).resolve()
    matches = sorted((ROOT / '.tools/python').glob('cpython-3.15.2+freethreaded-*/bin/python3.15t'))
    if matches: return matches[0].resolve()
    raise SystemExit('Install the pinned free-threaded interpreter: .tools/uv/uv python install 3.15.2t --install-dir .tools/python (or set BLUE_PYTHON).')


def runtime_info():
    return {'executable': sys.executable, 'version': sys.version, 'implementation': sys.implementation.name, 'Py_GIL_DISABLED': sysconfig.get_config_var('Py_GIL_DISABLED'), 'gil_enabled': getattr(sys, '_is_gil_enabled', lambda: None)(), 'SOABI': sysconfig.get_config_var('SOABI'), 'EXT_SUFFIX': sysconfig.get_config_var('EXT_SUFFIX'), 'CONFIG_ARGS': sysconfig.get_config_var('CONFIG_ARGS')}


def require_free_threaded():
    info = runtime_info()
    if info['Py_GIL_DISABLED'] != 1 or info['gil_enabled'] is not False:
        raise RuntimeError('Free-threaded CPython with the GIL disabled is required: ' + json.dumps(info))
    return info


def save_receipt(kind, data):
    RECEIPTS.mkdir(exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    path = RECEIPTS / f'{stamp}-{kind}.json'
    data = {'kind': kind, 'recorded_at_utc': datetime.now(timezone.utc).isoformat(), **data}
    path.write_text(json.dumps(data, indent=2) + '\n')
    print('Receipt:', path.relative_to(ROOT))
    return path
