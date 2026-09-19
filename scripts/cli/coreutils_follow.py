"""DEV-only schema-2 long-lived reference protocol inside bubblewrap.

Waits synchronize on kernel blocking state and explicit output byte barriers.
The deadline bounds a failure; elapsed time is never accepted as evidence.
"""
import os
from pathlib import Path, PurePosixPath
import select
import signal
import subprocess
import sys
import time


def path(value):
    p = PurePosixPath(value)
    if not p.is_absolute() or not p.is_relative_to('/home/kali') or '..' in p.parts:
        raise ValueError('Mutation outside isolated fixture')
    return str(p)


def mutate(step):
    target = path(step['path'])
    kind = step['operation']
    if kind in {'write', 'append'}:
        with open(target, 'ab' if kind == 'append' else 'wb') as stream:
            stream.write(bytes.fromhex(step['hex']))
    elif kind == 'truncate':
        with open(target, 'r+b') as stream:
            stream.truncate(step['size'])
    elif kind == 'rename':
        os.replace(target, path(step['target']))
    elif kind == 'unlink':
        os.unlink(target)
    elif kind == 'chmod':
        os.chmod(target, step['mode'])
    elif kind == 'mkdir':
        os.mkdir(target)
    elif kind == 'hardlink':
        os.link(target, path(step['target']))
    elif kind == 'symlink':
        os.symlink(step['target'], target)
    else:
        raise ValueError('Unknown mutation operation')


def execute(req, binary, env):
    interaction = req['interaction']
    if interaction['schemaVersion'] != 2:
        raise ValueError('Unknown follow protocol version')
    io = req.get('io') or {}
    writers = {name: subprocess.Popen([sys.executable, '-c', 'import sys; sys.stdin.buffer.read()'], stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
               for name in interaction.get('writers', [])}
    argv = req['argv'][:]
    for name, writer in writers.items():
        argv = [arg.replace('{' + name + '}', str(writer.pid)) for arg in argv]
    opened = []
    stdin = subprocess.DEVNULL
    if io.get('stdinPath'):
        stdin = os.open(path(io['stdinPath']), os.O_RDONLY)
        opened.append(stdin)
    stdout = subprocess.PIPE
    if io.get('stdoutPath'):
        stdout = os.open(path(io['stdoutPath']), os.O_WRONLY | os.O_CREAT | (os.O_APPEND if io.get('append') else os.O_TRUNC), 0o666)
        opened.append(stdout)
    if io.get('closedConsumer'):
        reader, stdout = os.pipe()
        os.close(reader)
        opened.append(stdout)
    child = subprocess.Popen([req['invocation'], *argv], executable=binary,
                             env=env, stdin=stdin, stdout=stdout, stderr=subprocess.PIPE,
                             restore_signals=True)
    for fd in opened:
        os.close(fd)
    streams = {s.fileno(): (s, bytearray()) for s in [child.stdout, child.stderr] if s}
    out = streams[child.stdout.fileno()][1] if child.stdout else bytearray()
    err = streams[child.stderr.fileno()][1]
    live = set(streams)
    observations = []
    deadline = time.monotonic() + 4

    def drain(timeout=0):
        ready, _, _ = select.select(list(live), [], [], timeout)
        for fd in ready:
            data = os.read(fd, 65536)
            if data:
                streams[fd][1].extend(data)
            else:
                live.remove(fd)
        if len(out) > 4 * 1024 * 1024 or len(err) > 65536:
            raise RuntimeError('Follow output limit exceeded')

    def observe():
        drain()
        observations.append({'stdoutHex': out.hex(), 'stderrHex': err.hex(), 'running': child.poll() is None})

    def wait_blocked():
        while child.poll() is None:
            drain()
            channel = Path(f'/proc/{child.pid}/wchan').read_text()
            if any(token in channel for token in ['poll', 'select', 'nanosleep', 'read', 'wait_woken']):
                drain()
                return
            if time.monotonic() >= deadline:
                raise RuntimeError(f'Follow wait barrier timed out: {channel}')
            select.select([], [], [], .001)
        raise RuntimeError('Follow process exited before wait barrier')

    try:
        for step in interaction['steps']:
            kind = step['kind']
            if kind == 'mutate':
                mutate(step)
            elif kind == 'mutateBatch':
                wait_blocked()
                os.kill(child.pid, signal.SIGSTOP)
                stopped, status = os.waitpid(child.pid, os.WUNTRACED)
                if stopped != child.pid or not os.WIFSTOPPED(status):
                    raise RuntimeError('Could not suspend the fixture consumer')
                try:
                    for mutation in step['steps']:
                        if mutation['kind'] != 'mutate':
                            raise ValueError('Batch accepts mutations only')
                        mutate(mutation)
                finally:
                    os.kill(child.pid, signal.SIGCONT)
            elif kind == 'stopWriter':
                writer = writers[step['writer']]
                if step.get('signal'):
                    os.kill(writer.pid, getattr(signal, step['signal']))
                else:
                    writer.stdin.close()
                writer.wait(timeout=max(.01, deadline - time.monotonic()))
            elif kind == 'await':
                while len(out) < step.get('stdoutBytes', 0) or len(err) < step.get('stderrBytes', 0):
                    remaining = deadline - time.monotonic()
                    if remaining <= 0 or not live:
                        raise RuntimeError(f'Follow output barrier timed out: {out!r}, {err!r}')
                    drain(remaining)
                observe()
            elif kind == 'wait':
                wait_blocked()
                observe()
            elif kind == 'signal':
                wait_blocked()
                os.kill(child.pid, getattr(signal, step['signal']))
            else:
                raise ValueError('Unknown follow step')
        while live:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise RuntimeError('Follow did not terminate at protocol end')
            drain(remaining)
        child.wait(timeout=max(.01, deadline - time.monotonic()))
        code = child.returncode
        return {'stdoutHex': out.hex(), 'stderrHex': err.hex(),
                'exitCode': code if code >= 0 else 128 - code,
                'termination': {'kind': 'exit', 'code': code} if code >= 0 else {'kind': 'signal', 'signal': signal.Signals(-code).name},
                'observations': observations,
                'endpoints': {'stdin': 'file' if io.get('stdinPath') else 'null',
                              'stdout': 'file' if io.get('stdoutPath') else 'closed-pipe' if io.get('closedConsumer') else 'pipe', 'stderr': 'pipe'}}
    finally:
        if child.poll() is None:
            child.kill()
        child.wait()
        for writer in writers.values():
            if writer.poll() is None:
                writer.kill()
            writer.wait()
