"""DEV-only controlled descriptors/PTY/signals inside the reference namespace."""
import json
import array
import fcntl
import os
from pathlib import Path
import select
import signal
import subprocess
import termios
import time


def execute(req, binary, env):
    io = req.get('io') or {}
    interaction = req.get('interaction')
    opened = []
    stdin = subprocess.PIPE
    stdout = subprocess.PIPE
    stderr = subprocess.PIPE
    master = None
    monitor = None
    if interaction and not io.get('stdinPath'):
        master, slave = os.openpty()
        attrs = termios.tcgetattr(slave)
        attrs[3] = (attrs[3] | termios.ICANON) & ~(termios.ECHO | termios.ECHONL)
        attrs[0] &= ~(termios.INLCR | termios.IGNCR)
        attrs[0] |= termios.ICRNL
        termios.tcsetattr(slave, termios.TCSANOW, attrs)
        stdin = slave
        monitor = os.dup(slave)
        opened.append(slave)
    if io.get('stdinPath'):
        stdin = os.open(io['stdinPath'], os.O_RDONLY)
        opened.append(stdin)
    if io.get('stdoutPath'):
        stdout = os.open(io['stdoutPath'], os.O_WRONLY | os.O_CREAT | (os.O_APPEND if io.get('append') else os.O_TRUNC), 0o666)
        opened.append(stdout)
    if io.get('stderrPath'):
        stderr = os.open(io['stderrPath'], os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o666)
        opened.append(stderr)
    if io.get('closedConsumer'):
        reader, stdout = os.pipe()
        os.close(reader)
        opened.append(stdout)
    # Popen's exec-error handshake completes before signal delivery is allowed.
    child = subprocess.Popen([req['invocation'], *req['argv']], executable=binary,
                             env=env, stdin=stdin, stdout=stdout, stderr=stderr, restore_signals=True)
    for fd in opened:
        os.close(fd)
    if interaction and any(s.get('waitFor') == 'output' for s in interaction['steps']):
        # Match the shared virtual 64 KiB consumer window explicitly.
        fcntl.fcntl(child.stdout.fileno(), fcntl.F_SETPIPE_SZ, 65536)
    observations = []
    collected = bytearray()
    deadline = time.monotonic() + 4
    try:
        if interaction:
            for step in interaction['steps']:
                kind = step['kind']
                if kind == 'write':
                    data = bytes.fromhex(step['hex'])
                    while data:
                        data = data[os.write(master, data):]
                elif kind == 'eof':
                    os.write(master, b'\x04')
                elif kind == 'expect':
                    expected = bytes.fromhex(step['stdoutHex'])
                    while len(collected) < len(expected):
                        remaining = deadline - time.monotonic()
                        if remaining <= 0 or not select.select([child.stdout], [], [], remaining)[0]:
                            raise RuntimeError('TTY stdout barrier timed out')
                        part = os.read(child.stdout.fileno(), 4096)
                        if not part:
                            raise RuntimeError('TTY exited before stdout barrier')
                        collected.extend(part)
                    if collected != expected:
                        raise RuntimeError(f'TTY barrier differs: {collected.hex()} != {expected.hex()}')
                    observations.append({'stdoutHex': collected.hex(), 'running': child.poll() is None})
                elif kind == 'signal':
                    if step.get('waitFor') == 'output':
                        while True:
                            if child.poll() is not None:
                                raise RuntimeError('Process exited before output barrier')
                            channel = Path(f'/proc/{child.pid}/wchan').read_text()
                            syscall = Path(f'/proc/{child.pid}/syscall').read_text().split()
                            if 'pipe_write' in channel or (len(syscall) > 1 and syscall[0] in {'1', '20'} and int(syscall[1], 16) == 1):
                                break
                            if time.monotonic() > deadline:
                                raise RuntimeError('Output backpressure barrier timed out')
                            select.select([], [], [], .001)
                        ignored = next(int(line.split()[1], 16) for line in Path(f'/proc/{child.pid}/status').read_text().splitlines() if line.startswith('SigIgn:'))
                        number = getattr(signal, step['signal'])
                        os.kill(child.pid, number)
                        if not ignored & (1 << (number - 1)):
                            # Reaping before draining prevents a blocked write from
                            # racing with the terminating signal after pipe space opens.
                            child.wait(timeout=max(.01, deadline - time.monotonic()))
                        continue
                    # Synchronize on a blocked kernel read, not an arbitrary delay.
                    while True:
                        if child.poll() is not None:
                            raise RuntimeError('Process exited before signal barrier')
                        channel = Path(f'/proc/{child.pid}/wchan').read_text()
                        syscall = Path(f'/proc/{child.pid}/syscall').read_text().split()
                        # Locked x86_64: read=0, readv=19. musl fread uses readv;
                        # WSL may expose wchan=0 even while /proc/syscall shows
                        # the blocked stdin read. Do not replace this with a sleep.
                        queued = array.array('i', [0])
                        fcntl.ioctl(monitor, termios.FIONREAD, queued, True)
                        while child.stdout and select.select([child.stdout], [], [], 0)[0]:
                            part = os.read(child.stdout.fileno(), 65536)
                            if not part:
                                break
                            collected.extend(part)
                        input_wait = 'read' in channel or (len(syscall) > 1 and syscall[0] in {'0', '19'} and int(syscall[1], 16) == 0)
                        # tee -p uses iopoll on stdin/stdout before read. Both endpoints
                        # are controlled here; the output consumer remains open.
                        input_wait |= req['command'] == 'tee' and (channel.endswith('sys_poll') or (syscall and syscall[0] == '7'))
                        if queued[0] == 0 and input_wait:
                            break
                        if time.monotonic() > deadline:
                            raise RuntimeError(f'Process did not reach read barrier: wchan={channel}, syscall={syscall}')
                        select.select([], [], [], 0.001)
                    os.kill(child.pid, getattr(signal, step['signal']))
                else:
                    raise ValueError('Unknown reference interaction')
            out, err = child.communicate(timeout=max(0.1, deadline - time.monotonic()))
        else:
            out, err = child.communicate(bytes.fromhex(req['stdinHex']) if stdin == subprocess.PIPE else None, timeout=4)
        collected.extend(out or b'')
        code = child.returncode
        termination = {'kind': 'exit', 'code': code} if code >= 0 else {'kind': 'signal', 'signal': signal.Signals(-code).name}
        return {'stdoutHex': collected.hex(), 'stderrHex': (err or b'').hex(),
                'exitCode': code if code >= 0 else 128 - code, 'termination': termination,
                'observations': observations,
                'endpoints': {'stdin': 'file' if io.get('stdinPath') else 'pty-canonical-echo-off' if interaction else 'pipe',
                              'stdout': 'file' if io.get('stdoutPath') else 'closed-pipe' if io.get('closedConsumer') else 'pipe',
                              'stderr': 'file' if io.get('stderrPath') else 'pipe'}}
    finally:
        if child.poll() is None:
            child.kill()
        child.wait()
        if master is not None:
            os.close(master)
        if monitor is not None:
            os.close(monitor)
