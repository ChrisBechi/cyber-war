// Shared filesystem/path inputs. No expected output is computed here.
import { request } from './common-contracts.mjs';

export const filesystemCommands = ['readlink', 'realpath', 'mkdir', 'rmdir', 'ln'];
export const specialNames = [
  'two words',
  'tab\tname',
  'line\nname',
  'back\\slash',
  "apost'rophe",
  'double"quote',
  '-dash',
  'ação',
];
export function pathFixture() {
  const home = '/home/kali/';
  return {
    files: { [home + 'file']: 'shared\n', [home + 'tree/child/data']: 'data\n' },
    directories: [home + 'empty', home + 'tree/child', home + 'denied'],
    modes: { [home + 'denied']: 0 },
    hardlinks: { [home + 'hard']: home + 'file' },
    symlinks: Object.fromEntries(
      Object.entries({
        link: 'file',
        absolute: home + 'file',
        dot: './tree/../file',
        dirlink: 'tree/child',
        broken: 'absent',
        brokenparent: 'absent/child',
        chain: 'link',
        loop: 'loop2',
        loop2: 'loop',
        self: 'self',
        'tree/up': '../file',
        slashfile: 'file/',
        slashdir: 'tree/',
        ...Object.fromEntries(specialNames.map((n) => [n, './tree/../file'])),
      }).map(([k, v]) => [home + k, v]),
    ),
  };
}
export const pathOperands = [
  'file',
  'hard',
  'empty',
  'tree',
  'tree/child/data',
  'link',
  'absolute',
  'dot',
  'dirlink',
  'dirlink/..',
  'dirlink/../data',
  'dirlink/../../file',
  'broken',
  'brokenparent',
  'chain',
  'loop',
  'loop/../file',
  'self',
  'slashfile',
  'slashdir',
  'link/',
  'empty/',
  'file/',
  'file/.',
  'file/..',
  'absent',
  'absent/',
  'absent/child',
  'absent/../file',
  'empty/../file',
  'tree//child/../child/data',
  'denied/child',
  'denied/../file',
  '.',
  '..',
  '',
  '/home/kali/link',
  '//home///kali/link',
  '///',
  'tree/up',
];

export function filesystemRequests(command) {
  const tests = [];
  const make = (id, argv, extra = {}) =>
    tests.push(request(command, id, argv, { fixture: pathFixture(), ...extra }));
  for (const kind of ['help', 'version']) make('common-' + kind, ['--' + kind]);
  for (const [id, argv] of Object.entries({
    'missing-operands': [],
    'unknown-short': ['-?'],
    'unknown-long': ['--no-such-option'],
    'unknown-utf8': ['-é'],
    'option-help-value': ['--help=x'],
    'options-double-dash': ['--', '-dash'],
    'options-help-immediate': ['--help', '--invalid'],
    'options-version-immediate': ['--version', '--invalid'],
  }))
    make(id, argv);
  if (command === 'readlink') {
    for (const [m, flags] of [[], ['-v'], ['-fv'], ['-ev'], ['-mv']].entries())
      for (const [i, path] of pathOperands.entries()) make(`paths-${m}-${i}`, [...flags, path]);
    for (const [i, path] of specialNames.entries()) {
      make(`special-filenames-${i}`, ['--', path]);
      make(`special-errors-${i}`, ['-v', path + '/bad']);
    }
    for (const [i, flags] of [
      [],
      ['-n'],
      ['-z'],
      ['-nz'],
      ['-vn'],
      ['-vqs'],
      ['-sv'],
      ['-vs'],
      ['-fme'],
      ['-emf'],
      ['-efm'],
      ['--canonicalize'],
      ['--canonicalize-existing'],
      ['--canonicalize-missing'],
      ['--no-newline'],
      ['--quiet'],
      ['--silent'],
      ['--verbose'],
      ['--zero'],
      ['--can'],
      ['--canon=x'],
      ['--no'],
      ['--ver'],
      ['--verb'],
    ].entries()) {
      make(`options-single-${i}`, [...flags, 'broken']);
      make(`multiple-operands-${i}`, [...flags, 'link', 'file', 'broken', 'absolute']);
    }
    make('options-permutation', ['link', '-z', 'absolute']);
    make('options-posix', ['link', '-z', 'absolute'], {
      env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' },
    });
    make('invocation-basename', ['-v', 'file'], { invocation: 'readlink' });
    make('invocation-absolute', ['-v', 'file']);
    make('integration-pipe', ['-mz', 'link', 'missing'], { transport: 'pipe' });
    make('integration-redirect', ['-v', 'link', 'file'], {
      io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
    });
    for (const n of [254, 255, 256])
      for (const flag of ['-v', '-fv', '-ev', '-mv'])
        make(`component-limit-${n}-${flag.slice(1).toLowerCase()}`, [flag, 'x'.repeat(n)]);
    // GNU follows arbitrarily long acyclic chains; VFS traversal stays bounded.
    const fixture = pathFixture();
    for (let n = 0; n < 45; n++)
      fixture.symlinks['/home/kali/c' + n] = n === 44 ? 'file' : 'c' + (n + 1);
    for (const flag of ['-v', '-fv', '-ev', '-mv'])
      make('symlink-chain-45-' + flag.slice(1).toLowerCase(), [flag, 'c0'], { fixture });
    for (const [i, target] of ['grow/child', './', '/', 'dirlink/../child/data'].entries()) {
      const fixture = pathFixture();
      fixture.symlinks['/home/kali/grow'] = target;
      // GNU canonicalization of grow -> grow/child exhausts the locked sandbox
      // (SIGKILL/timeout), so only its finite raw read belongs to the oracle corpus.
      // The shared VFS property test separately proves bounded cycle rejection.
      for (const mode of i === 0 ? ['-v'] : ['-v', '-fv', '-ev', '-mv'])
        make(`symlink-expansion-${i}-${mode.slice(1)}`, [mode, 'grow'], { fixture });
    }
  } else if (command === 'realpath') {
    for (const [m, flags] of [[], ['-e'], ['-m'], ['-s'], ['-L'], ['-ms']].entries())
      for (const [i, path] of pathOperands.entries()) make(`paths-${m}-${i}`, [...flags, path]);
    for (const [i, path] of specialNames.entries()) {
      make(`special-filenames-${i}`, ['--', path]);
      make(`special-errors-${i}`, [path + '/bad']);
    }
    for (const [i, flags] of [
      ['-eL'],
      ['-es'],
      ['-mL'],
      ['-P'],
      ['-LP'],
      ['-PL'],
      ['-sL'],
      ['-Ls'],
      ['-sP'],
      ['-Ps'],
      ['-em'],
      ['-me'],
      ['-q'],
      ['-z'],
      ['-eqz'],
      ['--logical'],
      ['--physical'],
      ['--strip'],
      ['--no-symlinks'],
      ['--canonicalize-existing'],
      ['--canonicalize-missing'],
      ['--quiet'],
      ['--zero'],
      ['--relative'],
      ['--relative-to'],
      ['--canonicalize'],
      ['--relative-to=x', '--help'],
    ].entries())
      make(`options-multiple-${i}`, [...flags, 'dirlink/..', 'broken', 'loop']);
    for (const [i, base] of [
      '.',
      'tree',
      'tree/child',
      'dirlink',
      'absent',
      'file',
      '',
      'loop',
    ].entries())
      for (const [m, flags] of [[], ['-e'], ['-m'], ['-s']].entries())
        for (const option of ['relative-to', 'relative-base'])
          make(`relative-${option}-${i}-${m}`, [
            ...flags,
            `--${option}=${base}`,
            '.',
            'tree/child/data',
            'file',
            'absent',
          ]);
    for (const [i, [to, base]] of [
      ['tree/child', 'tree'],
      ['tree', 'tree/child'],
      ['empty', 'tree'],
      ['.', '/'],
      ['dirlink', 'tree'],
      ['tree/child', 'dirlink'],
      ['missing/child', 'missing'],
    ].entries())
      for (const [m, flags] of [[], ['-e'], ['-m'], ['-L'], ['-s']].entries())
        make(`relative-pair-${i}-${m}`, [
          ...flags,
          '--relative-to=' + to,
          '--relative-base=' + base,
          'tree',
          'tree/child',
          'tree/child/data',
          'file',
        ]);
    make('relative-duplicate', [
      '--relative-to=missing',
      '--relative-to=tree',
      '--relative-base=/',
      '--relative-base=tree',
      'tree/child',
    ]);
    make('relative-missing-option-arg', ['--relative-to']);
    make('relative-missing-base-arg', ['--relative-base']);
    make('relative-quiet-error', ['-eq', '--relative-to=absent', 'file']);
    make('relative-missing-operands', ['--relative-to=absent']);
    for (const [i, path] of [
      'absent/child',
      'absent/./child',
      'absent/child/..',
      'absent/child/../file',
      'absent/child/.',
      'file/./child',
      'absent/child/',
      'absent/child/./..',
    ].entries())
      make('lexical-regression-' + i, ['-s', path]);
    make('options-permutation', ['link', '-z', 'absolute']);
    make('options-posix', ['link', '-z', 'absolute'], {
      env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' },
    });
    make('invocation-basename', ['file', 'missing/child'], { invocation: 'realpath' });
    make('integration-pipe', ['-mz', 'link', 'missing'], { transport: 'pipe' });
    make('integration-redirect', ['link', 'file', 'missing/child'], {
      io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
    });
    for (const n of [254, 255, 256])
      for (const flag of ['-e', '-m', '-s', '-L'])
        make(`component-limit-${n}-${flag.slice(1).toLowerCase()}`, [flag, 'x'.repeat(n)]);
  } else if (command === 'mkdir') {
    const paths = [
      ...pathOperands,
      'new',
      'new/child',
      'new/child/deep',
      'tree/new',
      'new//child///',
      'new/../other',
      'new/./child',
      'dirlink/new',
      'new/../file/child',
      'new/a/../../file/child',
    ];
    for (const [m, flags] of [[], ['-p'], ['-pv']].entries())
      for (const [i, path] of paths.entries()) make(`paths-${m}-${i}`, [...flags, path]);
    const modes = [
      '0',
      '000',
      '700',
      '0755',
      '0777',
      '0000',
      '4755',
      '2750',
      '1777',
      '07777',
      'u=rwx,g=rx,o=',
      'u=rw',
      'a=',
      'a+rwx',
      'u+s,g+s,o+t',
      'u=rwX,g=u,o=g',
      'g=u',
      'o=u',
      'u=rwx,g=u-w,o=g',
      '+w',
      '-w',
      '=rx',
      'a-X',
      'u+X,g-X,o+r',
      'u=rw+x',
      'a+st',
      '+x',
      'u=,g=,o=',
      'u-s,g-s',
      '00700',
    ];
    for (const [i, mode] of modes.entries()) {
      for (const mask of [0, 0o022, 0o077, 0o777])
        make(`modes-${i}-${mask}`, ['-m', mode, 'new'], { process: { umask: mask } });
      make(`parents-mode-${i}`, ['-pm' + mode, 'new/parent/leaf']);
    }
    for (const [i, mode] of [
      '+700',
      '-700',
      '=700',
      '=00700',
      '+4000',
      '-4000',
      'u+',
      'g-',
      'a+X',
      'u+t',
      'g+t',
      'o+s',
      'u==rw',
      'u=ug',
      'u=urw',
      'u=gu',
    ].entries()) {
      const fixture = pathFixture();
      fixture.modes['/home/kali/tree'] = 0o2775;
      make('mode-grammar-' + i, ['-m', mode, 'tree/new'], { fixture });
    }
    for (const [i, mode] of [
      '700',
      '0700',
      '00700',
      '07777',
      'g-s',
      'u-s',
      'o-t',
      'a=rx',
    ].entries()) {
      const fixture = pathFixture();
      fixture.modes['/home/kali/tree'] = 0o2775;
      make('setgid-parent-' + i, ['-m', mode, 'tree/new'], { fixture });
    }
    for (const [i, mode] of [
      '8',
      '888',
      '10000',
      '-1',
      'u',
      'u+y',
      'z+r',
      'u+r,',
      ',u+r',
      'u+r,,g+w',
      'u+☃',
      '',
    ].entries())
      make('invalid-mode-' + i, ['--mode=' + mode, 'new', 'other']);
    for (const [i, argv] of [
      ['-m'],
      ['--mode'],
      ['--mo'],
      ['-m', 'bad'],
      ['-m', 'bad', '--help'],
      ['-m', 'bad', '-m', '700', 'new'],
      ['-m700', '-m', 'bad', 'new'],
      ['--parents', '--verbose', 'new/child'],
      ['new', '-pv', 'new/child'],
      ['-pv', 'new', 'file', 'other'],
      ['-pm000', 'new', 'new/child', 'other'],
      ['-Z', 'new'],
      ['--context', 'new'],
      ['--context=x', 'new'],
      ['--context=', 'new'],
      ['--context=x', '--help'],
      ['--context=x', '--bad'],
      ['--context=x'],
      ['--context=x', '-m', 'bad', 'new'],
      ['-Zx', 'new'],
      ['--parents=x', 'new'],
      ['--ver'],
      ['--verbose=x', 'new'],
    ].entries())
      make('options-' + i, argv);
    for (const [i, name] of specialNames.entries()) {
      const fixture = pathFixture();
      delete fixture.symlinks['/home/kali/' + name];
      make('special-filenames-' + i, ['-v', '--', name], { fixture });
    }
    for (const mask of [0o077, 0o777])
      make('parents-umask-' + mask, ['-pv', 'new/child'], { process: { umask: mask } });
    make('invocation-basename', ['-v', 'new'], { invocation: 'mkdir' });
    make('options-posix', ['new', '-pv', 'other'], { env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
    make('integration-pipe', ['-pv', 'new/child'], { transport: 'pipe' });
    make('integration-redirect', ['-v', 'new', 'file', 'other'], {
      io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
    });
    for (const n of [254, 255, 256]) make('component-limit-' + n, ['-pv', 'new/' + 'x'.repeat(n)]);
    make('permissions-parent-root', ['-pv', 'denied/new'], { process: { actor: 'root', uid: 0 } });
  } else if (command === 'rmdir') {
    for (const [i, argv] of [
      ['-pv', '/home/kali/empty'],
      ['-pv', '--ignore-fail-on-non-empty', '/home/kali/empty'],
      ['-pv', '//home///kali/empty/'],
      ['-pv', 'empty/../empty'],
      ['-pv', 'nest/a/b/..'],
      ['-pv', './empty'],
    ].entries()) {
      const fixture = pathFixture();
      fixture.directories.push('/home/kali/nest/a/b');
      // The oracle home is a bind mount (EBUSY), unlike the virtual home.
      // Preserve those exploratory observations as an explicit environment boundary.
      if (![0, 2].includes(i)) make('ancestry-regression-' + i, argv, { fixture });
    }
    for (const [i, path] of ['/home/kali/tree/empty', '//home///kali/tree/empty/'].entries()) {
      const fixture = pathFixture();
      fixture.directories.push('/home/kali/tree/empty');
      make('absolute-ancestry-' + i, ['-pv', path], { fixture });
    }
    for (const [i, mode] of [0o555, 0o111, 0o000].entries())
      for (const [j, ignore] of [false, true].entries()) {
        const fixture = pathFixture();
        fixture.files['/home/kali/restricted/child/file'] = 'content';
        fixture.modes['/home/kali/restricted'] = mode;
        make(
          `nonempty-permission-regression-${i}-${j}`,
          [...(ignore ? ['--ignore-fail-on-non-empty'] : []), '-v', 'restricted/child'],
          { fixture },
        );
      }
    const paths = [
      ...pathOperands,
      'dirlink/',
      'broken/',
      'loop/',
      'self/',
      'slashdir/',
      'empty/.',
      'empty/..',
      '/',
      '//',
      'denied',
      'denied/',
      'tree/child',
      'nest/a/b',
      'nest//a///b///',
      './nest/a/b',
      'nest/a/./b',
      'nest/a/b/../b',
      'empty-link',
      'empty-link/',
      'empty-link/..',
      'nest-link/b',
    ];
    for (const [m, flags] of [
      [],
      ['-pv'],
      ['--ignore-fail-on-non-empty'],
      ['-pv', '--ignore-fail-on-non-empty'],
    ].entries()) {
      for (const [i, path] of paths.entries()) {
        const fixture = pathFixture();
        fixture.directories.push('/home/kali/nest/a/b');
        fixture.symlinks['/home/kali/empty-link'] = 'empty';
        fixture.symlinks['/home/kali/nest-link'] = 'nest/a';
        make(`paths-${m}-${i}`, [...flags, path], { fixture });
      }
    }
    for (const [i, argv] of [
      ['-v', 'empty', 'missing', 'tree', 'denied'],
      ['-pv', 'nest/a/b', 'empty'],
      ['--parents', '--verbose', 'nest/a/b'],
      ['--path', 'nest/a/b'],
      ['--pa', 'nest/a/b'],
      ['--par', 'nest/a/b'],
      ['--ignore', 'tree'],
      ['--ignore-fail-on-non-empty=x', 'tree'],
      ['--parents=x', 'empty'],
      ['--verbose=x', 'empty'],
      ['--ver'],
      ['-i', 'tree'],
      ['empty', '-v', 'tree'],
      ['-p', 'empty', 'empty'],
      ['-pv', 'empty-link/'],
      ['-v', '--ignore-fail-on-non-empty', 'missing', 'file', 'tree', 'empty'],
    ].entries()) {
      const fixture = pathFixture();
      fixture.directories.push('/home/kali/nest/a/b');
      fixture.symlinks['/home/kali/empty-link'] = 'empty';
      make('options-' + i, argv, { fixture });
    }
    for (const [i, name] of specialNames.entries()) {
      const fixture = pathFixture();
      delete fixture.symlinks['/home/kali/' + name];
      fixture.directories.push('/home/kali/' + name);
      make('special-filenames-' + i, ['-v', '--', name], { fixture });
      make('special-errors-' + i, ['-v', '--', name]);
    }
    for (const [i, mode] of [0o000, 0o111, 0o444, 0o555, 0o755, 0o1777].entries()) {
      const fixture = pathFixture();
      fixture.directories.push('/home/kali/restricted/child');
      fixture.modes['/home/kali/restricted'] = mode;
      for (const [m, flags] of [[], ['-p'], ['--ignore-fail-on-non-empty']].entries()) {
        make(`permissions-${i}-${m}`, [...flags, 'restricted/child', 'restricted'], { fixture });
        make(`permissions-nonempty-${i}-${m}`, [...flags, 'restricted'], { fixture });
      }
    }
    make('options-posix', ['empty', '-v', 'tree'], { env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
    make('invocation-basename', ['-v', 'empty', 'tree'], { invocation: 'rmdir' });
    make('integration-pipe', ['-v', 'empty', 'tree'], { transport: 'pipe' });
    make('integration-redirect', ['-v', 'empty', 'tree'], {
      io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
    });
    for (const n of [254, 255, 256]) make('component-limit-' + n, ['-pv', 'x'.repeat(n)]);
  } else if (command === 'ln') {
    for (const [i, version] of ['0', '0001', '9'.repeat(80)].entries()) {
      const fixture = pathFixture();
      fixture.files['/home/kali/hard.~' + version + '~'] = 'backup';
      make('backup-number-boundary-' + i, ['-bv', 'file', 'hard'], { fixture });
    }
    for (const n of [254, 255]) {
      const fixture = pathFixture(),
        name = 'x'.repeat(n);
      fixture.files['/home/kali/' + name] = 'old';
      make('backup-name-boundary-' + n, ['-bv', 'file', name], { fixture });
      for (const strategy of ['simple', 'numbered'])
        make(`backup-name-${strategy}-${n}`, ['-v', '--backup=' + strategy, 'file', name], {
          fixture,
        });
    }
    make('interactive-stdin-directory', ['-iv', 'file', 'hard'], {
      stdin: null,
      io: { stdinPath: '/home/kali/empty' },
    });
    for (const [i, argv] of [
      ['--backup=bad', 'file', 'new'],
      ['-S', 'x/y', 'file', 'new'],
      ['--backup=bad', 'file', 'empty'],
      ['-t', 'missing', '--help'],
      ['-t', 'file', '--help'],
      ['-t', 'empty', '-t', 'tree', '--help'],
      ['-T', 'file'],
      ['-s', '-b', '', 'file'],
      ['-s', '-b', 'file', 'file'],
      ['-b', '--suffix=~', 'hard', 'file'],
    ].entries())
      make('ordering-regression-' + i, argv);
    for (const [i, flags] of [
      ['-fv'],
      ['-iv'],
      ['-bv'],
      ['-fbv'],
      ['-fv', '--backup=numbered'],
      ['-sfv'],
    ].entries()) {
      const fixture = pathFixture();
      fixture.files['/home/kali/a/data'] = 'A';
      fixture.files['/home/kali/b/data'] = 'B';
      make('collision-' + i, [...flags, 'a/data', 'b/data', 'empty'], { fixture, stdin: 'y\ny\n' });
    }
    for (const n of [4095, 4096, 4097])
      make('target-byte-limit-' + n, ['-s', 'x'.repeat(n), 'new']);
    make('directory-root-attempt', ['-d', 'tree', 'new'], { process: { actor: 'root', uid: 0 } });
    for (const [m, flags] of [[], ['-s'], ['-L'], ['-sr']].entries())
      for (const [i, path] of pathOperands.entries())
        make(`sources-${m}-${i}`, [...flags, '--', path, 'new']);
    const destinations = [
      'new',
      'file',
      'hard',
      'empty',
      'tree',
      'dirlink',
      'broken',
      'loop',
      'link',
      'empty/',
      'file/',
      'absent/child',
      'denied/new',
      'tree/new',
      'new/',
      'dirlink/',
      '.',
      '',
      '/home/kali/new',
    ];
    for (const [m, flags] of [[], ['-s'], ['-f'], ['-sf'], ['-nf'], ['-T'], ['-sfT']].entries())
      for (const [i, dest] of destinations.entries())
        make(`destinations-${m}-${i}`, [...flags, 'file', dest]);
    for (const [i, argv] of [
      ['file'],
      ['tree/child/data'],
      ['-s', 'missing'],
      ['-s', 'tree'],
      ['-s', ''],
      ['file', 'tree/child/data', 'empty'],
      ['-sv', 'file', 'missing', 'tree/child/data', 'empty'],
      ['-v', 'file', 'missing', 'tree/child/data', 'empty'],
      ['-v', 'file', 'file', 'empty'],
      ['-fv', 'file', 'file', 'empty'],
      ['-sv', 'file', 'file', 'empty'],
      ['-sfv', 'file', 'file', 'empty'],
      ['-v', 'file', 'hard', 'empty'],
      ['-vf', 'file', 'hard', 'empty'],
      ['-t', 'empty', 'file', 'hard'],
      ['--target-directory=dirlink', 'file', 'hard'],
      ['-t', 'file', 'file'],
      ['-t', 'missing', 'file'],
      ['-t', 'loop', 'file'],
      ['-t', 'denied', 'file'],
      ['-t', 'empty'],
      ['-t', 'empty', '-t', 'tree', 'file'],
      ['-Tt', 'empty', 'file'],
      ['-T', 'file', 'hard', 'empty'],
      ['-r', 'file', 'new'],
      ['-sPr', 'link', 'new'],
      ['-sL', 'link', 'new'],
      ['-LP', 'link', 'new'],
      ['-PL', 'link', 'new'],
      ['-d', 'tree', 'new'],
      ['-F', 'tree', 'new'],
      ['--directory', 'tree', 'new'],
      ['--symbolic', '--verbose', 'file', 'new'],
      ['--no-dereference', '-f', 'file', 'dirlink'],
      ['--no-target-directory', 'file', 'empty'],
      ['--logical', 'link', 'new'],
      ['--physical', 'link', 'new'],
      ['--relative', '-s', 'link', 'new'],
      ['--interactive', 'file', 'hard'],
      ['--force', 'file', 'hard'],
      ['-t'],
      ['--target-directory'],
      ['-S'],
      ['--suffix'],
      ['--ver'],
      ['--no'],
      ['--sym=x', 'file', 'new'],
      ['--backup=bad', '--help'],
      ['--backup=bad'],
      ['-S', 'x/y', '--help'],
      ['--backup=', 'file', 'hard'],
      ['file', '-s', 'new'],
      ['--', '-dash', 'new'],
    ].entries())
      make('options-' + i, argv);
    for (const [i, [from, to]] of [
      ['file', 'file'],
      ['file', './file'],
      ['file', 'hard'],
      ['link', 'link'],
      ['link', 'file'],
      ['file', 'link'],
      ['missing', 'file'],
      ['loop', 'file'],
      ['tree', 'file'],
    ].entries())
      for (const [j, flags] of [['-fv'], ['-sfv'], ['-bv'], ['-sbv']].entries())
        make(`same-file-${i}-${j}`, [...flags, from, to]);
    for (const [i, source] of [
      'file',
      'dirlink/../data',
      'broken',
      'loop',
      'missing/child',
      '/home/kali/tree/child/data',
    ].entries())
      for (const [j, dest] of ['new', 'empty/new', 'tree/child/new', 'dirlink/new'].entries())
        make(`relative-${i}-${j}`, ['-srv', source, dest]);
    for (const [i, control] of [
      'none',
      'off',
      'numbered',
      't',
      'existing',
      'nil',
      'simple',
      'never',
      'n',
      'e',
      's',
      'bad',
    ].entries())
      for (const [j, backups] of [
        [],
        ['file~'],
        ['file.~1~', 'file.~3~', 'file.~002~'],
      ].entries()) {
        const fixture = pathFixture();
        for (const b of backups) fixture.files['/home/kali/' + b] = 'old backup';
        make(`backup-${i}-${j}`, ['-sv', '--backup=' + control, 'missing', 'file'], { fixture });
      }
    for (const [i, argv] of [
      ['-bSv', '.bak', 'file', 'hard'],
      ['-b', '-S.bak', 'file', 'hard'],
      ['--suffix=.bak', 'file', 'hard'],
      ['-b', '--suffix=', 'file', 'hard'],
      ['-b', '--suffix=x/y', 'file', 'hard'],
      ['-b', '-S/', 'file', 'hard'],
      ['-b', '-S.', 'file', 'hard'],
      ['--backup=numbered', '-S.bak', 'file', 'hard'],
      ['-b', '--backup=none', 'file', 'hard'],
      ['--backup=none', '-b', 'file', 'hard'],
    ].entries())
      make('backup-options-' + i, argv);
    for (const [i, env] of [
      { VERSION_CONTROL: 'numbered' },
      { VERSION_CONTROL: 'bad' },
      { SIMPLE_BACKUP_SUFFIX: '.bak' },
      { SIMPLE_BACKUP_SUFFIX: 'x/y' },
    ].entries())
      make('backup-environment-' + i, ['-bv', 'file', 'hard'], { env: { LC_ALL: 'C', ...env } });
    for (const [i, answer] of [
      '',
      'y',
      'y\n',
      'Y\n',
      'yes\n',
      'Yes\n',
      'n\n',
      'N\n',
      'no\n',
      ' y\n',
      '\ny\n',
      '1\n',
      'sim\n',
      'yep\n',
      '\0y\n',
    ].entries())
      for (const [j, flags] of [['-iv'], ['-siv'], ['-biv']].entries())
        make(`interactive-${i}-${j}`, [...flags, 'file', 'hard'], { stdin: answer });
    for (const [i, argv] of [
      ['-if', 'file', 'hard'],
      ['-fi', 'file', 'hard'],
      ['-iv', 'file', 'missing'],
      ['-iv', 'missing', 'file'],
      ['-iv', '-T', 'file', 'tree'],
      ['-si', 'file', 'file'],
      ['-i', 'file', 'file'],
    ].entries())
      make('interactive-order-' + i, argv, { stdin: 'y\n' });
    const tty = { isTTY: true, columns: 80, rows: 24, ansiSupport: true, interactive: true };
    for (const [i, steps] of [
      [{ kind: 'write', hex: '790a' }, { kind: 'eof' }],
      [{ kind: 'write', hex: '6e0a' }, { kind: 'eof' }],
      [{ kind: 'eof' }],
      [{ kind: 'signal', signal: 'SIGINT' }],
      [{ kind: 'signal', signal: 'SIGTERM' }],
    ].entries())
      make('tty-' + i, ['-iv', 'file', 'hard'], {
        stdin: null,
        tty,
        interaction: { schemaVersion: 1, steps },
      });
    for (const [i, name] of specialNames.entries()) {
      make('special-filenames-' + i, ['-sv', '--', name, 'new']);
      make('special-errors-' + i, ['-v', '--', 'file', name]);
    }
    for (const [i, mode] of [0, 0o111, 0o444, 0o555, 0o755].entries()) {
      const fixture = pathFixture();
      fixture.modes['/home/kali/empty'] = mode;
      for (const [j, flags] of [[], ['-s']].entries())
        make(`permissions-${i}-${j}`, [...flags, 'file', 'empty/new'], { fixture });
    }
    for (const n of [254, 255, 256]) {
      make('component-target-' + n, ['-sv', 'x'.repeat(n), 'new']);
      make('component-destination-' + n, ['-sv', 'file', 'x'.repeat(n)]);
    }
    make('options-posix', ['file', '-s', 'new'], { env: { LC_ALL: 'C', POSIXLY_CORRECT: '1' } });
    make('invocation-basename', ['-iv', 'file', 'hard'], { invocation: 'ln', stdin: 'y\n' });
    make('integration-pipe', ['-iv', 'file', 'hard'], { stdin: 'y\n', transport: 'pipe' });
    make('integration-redirect', ['-sv', 'file', 'new'], {
      io: { stdoutPath: '/home/kali/out', stderrPath: '/home/kali/errors' },
    });
    const fixture = pathFixture();
    fixture.files['/home/kali/answer'] = 'y\n';
    make('integration-stdin-file', ['-iv', 'file', 'hard'], {
      fixture,
      stdin: null,
      io: { stdinPath: '/home/kali/answer' },
    });
    make('integration-closed-consumer', ['-sv', 'file', 'new'], { io: { closedConsumer: true } });
  } else throw new Error('Command generator not implemented: ' + command);
  return tests;
}
