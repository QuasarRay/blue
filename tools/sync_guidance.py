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
    check.add_argument('--commit', action='store_true', help='Commit only changed guidance mirrors (for CI)')
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
    if args.commit and missing:
        subprocess.run(['git', 'add', '--', *missing], cwd=ROOT, check=True)
        subprocess.run(['git', '-c', 'user.name=github-actions[bot]', '-c', 'user.email=41898282+github-actions[bot]@users.noreply.github.com', 'commit', '--only', '-m', 'Synchronize project guidance', '--', *missing], cwd=ROOT, check=True)
    print(f'{len(directories)} guidance directories; {len(missing)} differences' + (' repaired' if not args.check else ''))


if __name__ == '__main__': main()
