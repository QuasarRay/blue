#!/usr/bin/env python3
"""Mirror root guidance into owned source directories; preserve frozen inputs."""
import argparse
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXCLUDED = {'.git', '.tools', '.runtime', '.venv', 'target', 'dist', 'vendor', '__pycache__'}


def main():
    check = argparse.ArgumentParser()
    check.add_argument('--check', action='store_true')
    args = check.parse_args()
    content = (ROOT / 'AGENTS.MD').read_bytes()
    directories = {ROOT}
    for p in ROOT.rglob('*'):
        relative = p.relative_to(ROOT)
        if any(part in EXCLUDED for part in relative.parts) or relative.parts[:2] == ('reference', 'blueexposure'):
            continue
        if p.is_dir(): directories.add(p)
    missing = []
    for directory in sorted(directories):
        p = directory / 'AGENTS.MD'
        if not p.exists() or p.read_bytes() != content:
            missing.append(str(p.relative_to(ROOT)))
            if not args.check: p.write_bytes(content)
    if args.check and missing: raise SystemExit('\n'.join(missing))
    print(f'{len(directories)} guidance directories; {len(missing)} differences' + (' repaired' if not args.check else ''))


if __name__ == '__main__': main()
