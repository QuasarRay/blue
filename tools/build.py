#!/usr/bin/env python3
"""Build the production extension against the pinned free-threaded CPython."""
import argparse
import json
import os
import shutil
import subprocess
import sys
from evidence import ROOT, identity, python_path, save_receipt, sha


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    python = python_path()
    inspect = 'import sys,json;sys.path.insert(0,"tools");from evidence import require_free_threaded;print(json.dumps(require_free_threaded()))'
    info = json.loads(subprocess.check_output([str(python), '-c', inspect], cwd=ROOT, text=True))
    command = ['cargo', 'build', '--locked']
    if args.release: command.append('--release')
    if args.offline: command.append('--offline')
    env = {**os.environ, 'PYO3_PYTHON': str(python)}
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end='')
    data = {'command': command, 'environment': {'PYO3_PYTHON': str(python)}, 'implementation': identity(), 'python': info, 'exit_code': result.returncode, 'output': result.stdout, 'artifacts': {}}
    if result.returncode == 0:
        profile = 'release' if args.release else 'debug'
        runtime = ROOT / '.runtime'
        runtime.mkdir(exist_ok=True)
        artifact = runtime / ('blue' + info['EXT_SUFFIX'])
        shutil.copy2(ROOT / f'target/{profile}/libblue.so', artifact)
        data['artifacts'][artifact.relative_to(ROOT).as_posix()] = sha(artifact)
        data['artifacts'][f'target/{profile}/libblue.rlib'] = sha(ROOT / f'target/{profile}/libblue.rlib')
    save_receipt('build', data)
    return result.returncode


if __name__ == '__main__': sys.exit(main())
