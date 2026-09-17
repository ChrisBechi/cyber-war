"""DEV-only boundary evidence; never used to certify tail or run gameplay.

Capture/verify GNU follow-by-name across append and rotation in an isolated
namespace. Output barriers, not startup sleeps, determine fixture mutations.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import tempfile
import time

ARGV = ['-n1', '--follow=name', '--retry', '--sleep-interval=0.01',
        '--max-unchanged-stats=1', 'log']


def worker():
    Path('log').write_bytes(b'first\n')
    process = subprocess.Popen(['/usr/bin/tail', *ARGV], stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               env={'LC_ALL': 'C', 'PATH': '/usr/bin:/bin'})
    output = bytearray()
    observations = []

    def barrier(label, expected):
        deadline = time.monotonic() + 4
        while len(output) < len(expected):
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([process.stdout], [], [], remaining)[0]:
                raise RuntimeError(f'GNU follow barrier timed out: {label}')
            data = os.read(process.stdout.fileno(), 4096)
            if not data:
                raise RuntimeError('GNU tail ended before cancellation')
            output.extend(data)
        assert bytes(output) == expected
        assert process.poll() is None
        observations.append({'after': label, 'stdoutHex': output.hex(), 'running': True})

    try:
        barrier('initial', b'first\n')
        with Path('log').open('ab') as target:
            target.write(b'second\n')
        barrier('append', b'first\nsecond\n')
        # Atomic replacement avoids a race between missing/reappeared and
        # replaced diagnostics. Preserve the old inode under the rotated name.
        os.link('log', 'log.1')
        Path('replacement').write_bytes(b'third\n')
        os.replace('replacement', 'log')
        barrier('rename-and-recreate', b'first\nsecond\nthird\n')
        process.terminate()
        rest, errors = process.communicate(timeout=2)
        output.extend(rest)
        assert process.returncode == -15
        return {'argv': ARGV, 'stdinHex': '', 'observations': observations,
                'stdoutHex': output.hex(), 'stderrHex': errors.hex(),
                'termination': {'signal': 'SIGTERM', 'returncode': -15},
                'files': {name: Path(name).read_bytes().hex() for name in ['log', 'log.1']}}
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--capture', action='store_true')
    action.add_argument('--verify', action='store_true')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    if sys.platform != 'linux' or not shutil.which('bwrap'):
        parser.error('Requires the locked GNU Linux environment and bubblewrap')
    root = Path(__file__).resolve().parents[2]
    lock_path = root / 'content/cli-compatibility/coreutils-environment.json'
    lock = json.loads(lock_path.read_text())
    sha = lambda data: hashlib.sha256(data).hexdigest()
    binary_hash = sha(Path('/usr/bin/tail').read_bytes())
    # Alpine's locked coreutils multicall binary contains both head and tail.
    assert binary_hash == lock['binaryHashes']['head']
    version = subprocess.check_output(['/usr/bin/tail', '--version'], env={'LC_ALL': 'C'}, timeout=5).splitlines()[0].decode()
    assert version == f"tail (GNU coreutils) {lock['coreutilsVersion']}"

    def capture():
        with tempfile.TemporaryDirectory(prefix='cw-follow-') as directory:
            Path(directory).chmod(0o777)
            cmd = ['bwrap', '--unshare-all', '--die-with-parent', '--new-session', '--uid', '1000', '--gid', '1000']
            for name in ['/usr', '/bin', '/lib', '/lib64']:
                if Path(name).exists():
                    cmd += ['--ro-bind', name, name]
            cmd += ['--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
                    '--bind', directory, '/work', '--chdir', '/work',
                    '--ro-bind', str(Path(__file__).resolve()), '/probe.py',
                    '--clearenv', '--setenv', 'LC_ALL', 'C', '--',
                    '/usr/bin/python3', '/probe.py', '--worker']
            return json.loads(subprocess.check_output(cmd, timeout=15, env={'PATH': '/usr/bin:/bin'}))

    first, second = capture(), capture()
    assert first == second, 'Non-reproducible follow boundary evidence'
    evidence = {'schemaVersion': 1, 'provenance': 'GNU_BOUNDARY_PROBE',
                'certifiesExecutable': False, 'command': 'tail', 'version': version,
                'environment': lock['id'], 'locale': 'C', 'isolation': 'bubblewrap/unshare-all',
                'network': 'disabled', 'binaryHash': binary_hash,
                'lockHash': sha(lock_path.read_text().encode()),
                'probeHash': sha(Path(__file__).read_text().encode()),
                'reproducible': True, 'result': first}
    target = Path(args.output)
    if args.verify:
        previous = json.loads(target.read_text())
        previous.pop('capturedAt')
        assert previous == evidence, 'Boundary reproduction differs; original unchanged'
    else:
        if target.exists():
            parser.error('Capture target already exists; choose a new output path')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps({**evidence, 'capturedAt': datetime.now(timezone.utc).isoformat()}, indent=2) + '\n')
    print('tail boundary: append + rotation + SIGTERM, two identical GNU 9.7 runs; ' + ('verified' if args.verify else 'captured'))


if __name__ == '__main__':
    if sys.argv[1:] == ['--worker']:
        print(json.dumps(worker()))
    else:
        main()
