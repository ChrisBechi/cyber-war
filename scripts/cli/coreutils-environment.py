"""DEV only. Prepare the locked Linux reference environment; never gameplay.

Windows uses the dedicated CyberWar-GNU97 WSL2 distribution. Linux/macOS use
Docker with the same checksum-pinned rootfs and signed APKs, without latest tags.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / 'content/cli-compatibility/coreutils-environment.json'
CACHE = ROOT / '.tools/coreutils-reference'
DISTRO = 'CyberWar-GNU97'


def run(args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def wsl_path(path):
    p = str(path.resolve()).replace('\\', '/')
    if len(p) < 3 or p[1] != ':':
        raise ValueError('WSL reference requires a local drive path')
    return '/mnt/' + p[0].lower() + p[2:]


def download(row, urls):
    path = CACHE / row['file']
    if not path.exists():
        for url in urls:
            try:
                with urllib.request.urlopen(url, timeout=60) as response:
                    data = response.read(128 * 1024 * 1024 + 1)
                break
            except urllib.error.HTTPError as error:
                if error.code != 404:
                    raise
        else:
            raise RuntimeError(f'Pinned package unavailable: {row["file"]}; do not substitute another version')
        if hashlib.sha256(data).hexdigest() != row['sha256']:
            raise RuntimeError(f'Download checksum mismatch: {row["file"]}')
        path.write_bytes(data)
    if hashlib.sha256(path.read_bytes()).hexdigest() != row['sha256']:
        raise RuntimeError(f'Cache checksum mismatch: {path}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['prepare', 'run'])
    args, extra = parser.parse_known_args()
    lock = json.loads(LOCK.read_text(encoding='utf-8'))
    baseline = json.loads((ROOT / 'content/cli-compatibility/manifest.json').read_text(encoding='utf-8'))['software']['coreutils']['referenceVersion']
    if baseline != lock['coreutilsVersion']:
        parser.error('Reference environment baseline differs from manifest')
    windows = platform.system() == 'Windows'
    tag = 'cyber-war-coreutils:' + hashlib.sha256(LOCK.read_bytes()).hexdigest()[:16]
    if args.action == 'prepare':
        if extra:
            parser.error('Unexpected prepare arguments')
        CACHE.mkdir(parents=True, exist_ok=True)
        download(lock['base'], [lock['base']['url']])
        for row in lock['packages']:
            download(row, [url + row['file'] for url in lock['repositories']])
        package_files = [row['file'] for row in lock['packages']]
        if windows:
            found = subprocess.check_output(['wsl.exe', '--list', '--quiet']).decode('utf-16-le').split()
            if DISTRO not in found:
                run(['wsl.exe', '--import', DISTRO, str(CACHE / 'wsl'), str(CACHE / lock['base']['file']), '--version', '2'])
            run(['wsl.exe', '-d', DISTRO, '--exec', '/sbin/apk', 'add', '--no-network', *[wsl_path(CACHE / p) for p in package_files]])
            run(['wsl.exe', '-d', DISTRO, '--exec', '/usr/bin/basename', '--version'])
        else:
            # Build context contains only the locked artifacts, never the repository.
            dockerfile = 'FROM scratch\nADD ' + lock['base']['file'] + ' /\n'
            dockerfile += '\n'.join('COPY ' + p + ' /packages/' + p for p in package_files)
            dockerfile += '\nRUN /sbin/apk add --no-network /packages/*.apk && /bin/rm -r /packages\nWORKDIR /repo\n'
            (CACHE / 'Dockerfile').write_text(dockerfile, encoding='utf-8')
            (CACHE / '.dockerignore').write_text('*\n!Dockerfile\n!' + lock['base']['file'] + '\n' + ''.join('!' + p + '\n' for p in package_files), encoding='utf-8')
            run(['docker', 'build', '--network=none', '-t', tag, str(CACHE)])
        print('Prepared locked reference:', lock['id'])
    elif windows:
        run(['wsl.exe', '-d', DISTRO, '--exec', '/usr/bin/python3', wsl_path(ROOT / 'scripts/cli/coreutils-reference.py'), *extra])
    else:
        output = ROOT / 'artifacts/coreutils-reference-output'
        output.mkdir(parents=True, exist_ok=True)
        destination = [] if '--verify' in extra else ['--output-dir', '/output']
        run(['docker', 'run', '--rm', '--network=none', '--read-only', '--cap-add=SYS_ADMIN', '--security-opt=seccomp=unconfined', '--security-opt=apparmor=unconfined', '--memory=768m', '--cpus=2', '--pids-limit=128', '--tmpfs=/tmp:rw,nosuid,size=128m', '-v', str(ROOT) + ':/repo:ro', '-v', str(output) + ':/output:rw', tag, '/usr/bin/python3', '/repo/scripts/cli/coreutils-reference.py', *destination, *extra])


if __name__ == '__main__':
    main()
