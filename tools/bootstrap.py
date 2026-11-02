#!/usr/bin/env python3
"""Provision pinned local tooling without changing the system interpreter."""
import hashlib
import io
import platform
import subprocess
import tarfile
import urllib.request
from pathlib import Path
from evidence import ROOT, python_path

UV_VERSION = '0.13.6'
UV_SHA256 = 'f29c3e94ffabd7e6f3ba521dba169a9b7ec12100bf3257087f0f24f6123f3645'


def main():
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise SystemExit('This provisioning command targets the Linux x86_64 milestone. Provide a pinned free-threaded interpreter via BLUE_PYTHON on another host.')
    uv = ROOT / '.tools/uv/uv'
    if not uv.exists():
        url = f'https://github.com/astral-sh/uv/releases/download/{UV_VERSION}/uv-x86_64-unknown-linux-gnu.tar.gz'
        with urllib.request.urlopen(url, timeout=60) as response: content = response.read()
        if hashlib.sha256(content).hexdigest() != UV_SHA256: raise SystemExit('uv archive digest mismatch')
        uv.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(fileobj=io.BytesIO(content)) as archive:
            with archive.extractfile('uv-x86_64-unknown-linux-gnu/uv') as source:
                uv.write_bytes(source.read())
        uv.chmod(0o755)
    observed = subprocess.check_output([str(uv), '--version'], text=True).strip()
    if observed.split()[:2] != ['uv', UV_VERSION]: raise SystemExit('Unexpected uv version: ' + observed)
    subprocess.run([str(uv), 'python', 'install', '3.15.2t', '--install-dir', str(ROOT / '.tools/python')], cwd=ROOT, check=True)
    subprocess.run([str(python_path()), '-c', 'import sys; sys.path.insert(0,"tools"); from evidence import require_free_threaded; print(require_free_threaded())'], cwd=ROOT, check=True)


if __name__ == '__main__': main()
