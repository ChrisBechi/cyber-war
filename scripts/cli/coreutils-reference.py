"""DEV GNU reference capture/verify. Linux + bubblewrap only; no host fallback.

Each case runs twice in a fresh sandbox with no network or developer files.
Only system binaries/libraries, this worker and its request are read-only.
Normal --verify never changes evidence. --capture and --replace are explicit.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone

ROOT = Path('/') if sys.argv[1:2] == ['--worker'] else Path(__file__).resolve().parents[2]
LOCK_PATH = 'content/cli-compatibility/coreutils-environment.json'
HARNESS_PATH = 'scripts/cli/coreutils-reference.py'


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def source_hash(path):
    return sha((ROOT / path).read_text(encoding='utf-8').replace('\r\n', '\n').encode())


def request(case):
    # Same projection as referenceRequest in coreutils-reference.mjs.
    return {'id': case['id'], 'command': case['command'],
            'invocation': case.get('invocation', '/usr/bin/' + case['command']),
            'argv': case['argv'], 'stdinHex': case.get('stdinHex', (case.get('stdin') or '').encode().hex()),
            'env': case['env'], 'cwd': case['cwd'], 'fixture': {k: case.get('fixture', {}).get(k, [] if k in ['directories', 'setup'] else {}) for k in ['files', 'bytes', 'directories', 'modes', 'setup']},
            'process': case.get('process'), 'transport': case.get('transport', 'direct')}


def limits():
    import resource
    resource.setrlimit(resource.RLIMIT_FSIZE, (16 * 1024 * 1024, 16 * 1024 * 1024))
    resource.setrlimit(resource.RLIMIT_CPU, (4, 4))
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024, 512 * 1024 * 1024))


def fixture_path(root, name):
    path = PurePosixPath(name)
    try:
        parts = path.relative_to('/home/kali').parts
    except ValueError as error:
        raise ValueError('Reference fixture paths must be within /home/kali') from error
    if '..' in parts or not parts:
        raise ValueError('Invalid fixture path')
    return root.joinpath(*parts)


def snapshot(root):
    result, groups = {}, {}
    for path in sorted(root.rglob('*')):
        st = path.lstat()
        row = {'inodeGroup': groups.setdefault(st.st_ino, len(groups)), 'mode': st.st_mode & 0o7777, 'nlink': st.st_nlink}
        if path.is_symlink():
            row.update(type='symlink', target=os.readlink(path))
        elif path.is_file():
            row.update(type='file', contentHex=path.read_bytes().hex(), size=st.st_size)
        else:
            row.update(type='directory')
        result[str(path.relative_to(root))] = row
    return result


def environment(req):
    process = req.get('process') or {}
    if process.get('environment') is not None:
        entries = process['environment']
    else:
        user = process.get('actor', 'kali')
        env = {'HOME': '/root' if user == 'root' else '/home/kali', 'HOSTNAME': 'lifeos',
               'LANG': 'C.UTF-8', 'LOGNAME': user,
               'PATH': '/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
               'PWD': req['cwd'], 'SHELL': '/bin/bash', 'USER': user, **req['env']}
        entries = [[k, v] for k, v in sorted(env.items())]
    env = dict(entries)
    if env.get('LC_ALL') != 'C':
        raise ValueError('Every GNU case must explicitly use LC_ALL=C')
    return env


def worker(path):
    req = json.loads(Path(path).read_text())
    env = environment(req)
    argv = [req['invocation'], *req['argv']]
    if req['transport'] == 'redirect':
        fd = os.open('/home/kali/reference-output', os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o666)
        os.dup2(fd, 1)
        os.close(fd)
    elif req['transport'] == 'pipe':
        reader, writer = os.pipe()
        pid = os.fork()
        if pid:
            os.close(writer)
            while data := os.read(reader, 4096):
                view = memoryview(data)
                while view:
                    view = view[os.write(1, view):]
            os.close(reader)
            _, status = os.waitpid(pid, 0)
            os._exit(os.waitstatus_to_exitcode(status))
        os.close(reader)
        os.dup2(writer, 1)
        os.close(writer)
    binary = shutil.which(req['command'], path='/usr/bin:/bin')
    if not binary:
        raise RuntimeError('Missing locked GNU executable')
    os.execve(binary, argv, env)


def capture(case, bwrap):
    if case.get('script') or case['fixture'].get('setup') or case.get('inputEvents'):
        raise ValueError(f"{case['id']}: scripts/setup/events are project-only; use structured transport/identity")
    req = request(case)
    environment(req)
    process = req.get('process') or {}
    uid = process.get('uid', 1000)
    username = process.get('username', process.get('actor', 'kali'))
    with tempfile.TemporaryDirectory(prefix='cyber-war-coreutils-') as tmp:
        sandbox = Path(tmp)
        sandbox.chmod(0o755)
        home, etc = sandbox / 'home', sandbox / 'etc'
        home.mkdir(mode=0o777)
        home.chmod(0o777)
        etc.mkdir()
        (etc / 'passwd').write_text('' if username is None else f'{username}:x:{uid}:{uid}::/home/kali:/bin/sh\n')
        (etc / 'group').write_text(f'kali:x:{uid}:\n')
        for path in req['fixture']['directories']:
            fixture_path(home, path).mkdir(parents=True, exist_ok=True)
        for field in ['files', 'bytes']:
            for path, data in req['fixture'][field].items():
                target = fixture_path(home, path)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bytes.fromhex(data) if field == 'bytes' else data.encode())
        for path, mode in req['fixture']['modes'].items():
            fixture_path(home, path).chmod(mode)
        request_file = sandbox / 'request.json'
        request_file.write_text(canonical(req), encoding='utf-8')
        before = snapshot(home)
        cmd = [bwrap, '--unshare-all', '--die-with-parent', '--new-session', '--uid', str(uid), '--gid', str(uid)]
        for path in ['/usr', '/bin', '/lib', '/lib64']:
            if Path(path).exists():
                cmd += ['--ro-bind', path, path]
        cmd += ['--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
                '--bind', str(home), '/home/kali', '--ro-bind', str(etc), '/etc',
                '--ro-bind', str(Path(__file__).resolve()), '/reference/worker.py',
                '--ro-bind', str(request_file), '/reference/request.json',
                '--chdir', req['cwd'], '--clearenv', '--setenv', 'LC_ALL', 'C',
                '--', '/usr/bin/python3', '/reference/worker.py', '--worker', '/reference/request.json']
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            completed = subprocess.run(cmd, input=bytes.fromhex(req['stdinHex']), stdout=stdout, stderr=stderr,
                                       timeout=5, check=False, preexec_fn=limits, env={'PATH': '/usr/bin:/bin', 'LC_ALL': 'C'})
            stdout.seek(0)
            stderr.seek(0)
            out, err = stdout.read(4 * 1024 * 1024 + 1), stderr.read(65537)
            if len(out) > 4 * 1024 * 1024 or len(err) > 65536:
                raise RuntimeError('Reference output limit exceeded')
            if completed.returncode < 0 or err.startswith(b'bwrap:') or b'Traceback (most recent call last)' in err:
                raise RuntimeError(f'Sandbox/worker failed for {case["id"]}: {err[:1000]!r}')
        return {'id': req['id'], 'request': req, 'requestDigest': sha(canonical(req).encode()),
                'stdoutHex': out.hex(), 'stderrHex': err.hex(), 'exitCode': completed.returncode,
                'before': before, 'after': snapshot(home)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--capture', action='store_true')
    action.add_argument('--verify', action='store_true')
    parser.add_argument('--replace', action='store_true')
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument('--command')
    scope.add_argument('--wave', choices=['foundation'])
    parser.add_argument('--output-dir')
    parser.add_argument('--probe', help='DEV inputs; writes observations outside approved goldens')
    args = parser.parse_args()
    if args.replace and not args.capture:
        parser.error('--replace requires --capture')
    if sys.platform != 'linux' or not shutil.which('bwrap'):
        parser.error('DEV capture requires Linux and bubblewrap; Windows gameplay has no such dependency')
    manifest = json.loads((ROOT / 'content/cli-compatibility/manifest.json').read_text())
    version = manifest['software']['coreutils']['referenceVersion']
    config = json.loads((ROOT / 'content/cli-compatibility/coreutils.json').read_text())
    lock = json.loads((ROOT / LOCK_PATH).read_text())
    if lock['coreutilsVersion'] != version:
        parser.error('Reference environment baseline mismatch')
    names = [args.command] if args.command else [k for k, v in config['commands'].items() if v['area'] == args.wave]
    if any(n not in config['commands'] for n in names):
        parser.error('Unknown executable')
    cases = json.loads(Path(args.probe).read_text()) if args.probe else sum([json.loads(p.read_text()) for p in sorted((ROOT / 'tests/cli/compat/pilot').glob('*.json'))], [])
    cases = [c for c in cases if c['softwareId'] == 'coreutils' and c['command'] in names]
    if not cases:
        parser.error('No selected cases')
    binaries = {}
    for name in sorted({c['command'] for c in cases}):
        located = shutil.which(name, path='/usr/bin:/bin')
        if not located:
            parser.error(f'Missing GNU executable: {name}')
        binary = Path(located)
        result = subprocess.run([str(binary), '--version'], capture_output=True, timeout=5, check=True, env={'LC_ALL': 'C'})
        first = result.stdout.decode().splitlines()[0]
        if first != f'{name} (GNU coreutils) {version}':
            parser.error(f'Reference version mismatch: {first}; expected {version}')
        binaries[name] = {'versionLine': first, 'sha256': sha(binary.read_bytes())}
        if name in lock.get('binaryHashes', {}) and binaries[name]['sha256'] != lock['binaryHashes'][name]:
            parser.error(f'Locked GNU binary hash mismatch: {name}')
    output = Path(args.output_dir) if args.output_dir else ROOT / f'tests/cli/gnu/coreutils/{version}'
    if args.probe and not args.output_dir:
        parser.error('Probes require a separate --output-dir')
    env = {'id': lock['id'], 'lockHash': source_hash(LOCK_PATH), 'os': 'Linux', 'osRelease': Path('/etc/os-release').read_text(),
           'architecture': platform.machine(), 'isolation': 'bubblewrap/unshare-all', 'network': 'disabled',
           'locale': 'C', 'binaryHashes': binaries, 'packages': subprocess.check_output(['/sbin/apk', 'info', '-v'], stderr=subprocess.DEVNULL, text=True).splitlines()}
    for name in names:
        selected = [c for c in cases if c['command'] == name]
        target = output / f'{name}.json'
        if args.capture and target.exists() and not args.replace:
            parser.error(f'{target} exists; explicit --replace required')
        rows = []
        for case in selected:
            first, second = capture(case, shutil.which('bwrap')), capture(case, shutil.which('bwrap'))
            if first != second:
                raise RuntimeError(f'Non-reproducible GNU reference: {case["id"]}')
            first['reproducible'] = True
            rows.append(first)
        payload = {'schemaVersion': 2, 'provenance': 'GNU_PROBE' if args.probe else 'GNU_REFERENCE', 'version': version,
                   'command': name, 'locale': 'C', 'capturedAt': datetime.now(timezone.utc).isoformat(),
                   'harnessHash': source_hash(HARNESS_PATH), 'environment': {**env, 'binaryHashes': {name: binaries[name]}}, 'cases': rows}
        if args.verify:
            original = json.loads(target.read_text())
            for key in ['schemaVersion', 'provenance', 'version', 'command', 'locale', 'harnessHash', 'environment', 'cases']:
                if original[key] != payload[key]:
                    raise RuntimeError(f'Reference reproduction differs: {name}/{key}; golden unchanged')
        else:
            output.mkdir(parents=True, exist_ok=True)
            target.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        print(f'{name}: {len(rows)} GNU {version} cases, two identical runs; {"verified" if args.verify else "captured"}')


if __name__ == '__main__':
    if sys.argv[1:2] == ['--worker']:
        worker(sys.argv[2])
    else:
        main()
