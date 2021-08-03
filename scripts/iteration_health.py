import subprocess
from pathlib import Path
import json
import random
import time
import sys
import statistics


def main():
    very_start = time.time()

    res = subprocess.run(['cargo', 'metadata', '--no-deps', '--format-version=1'],
        check=True, capture_output=True, universal_newlines=True)
    metadata = json.loads(res.stdout)
    workspace_root = metadata['workspace_root']

    src_paths = []
    for package in metadata['packages']:
        for target in package['targets']:
            src_paths.append(target['src_path'])
            src_path = Path(target['src_path']).relative_to(workspace_root)

    MODES = dict(
        check=['cargo', 'check', '--workspace'],
        build=['cargo', 'build', '--workspace'],
        test=['cargo', 'build', '--workspace', '--tests'],
    )
    for cmd in MODES.values():
        print(' '.join(cmd), file=sys.stderr)
        subprocess.run(cmd, check=True)

    REPEAT = 3
    print('measuring incremental build times...', file=sys.stderr)
    print('[' + REPEAT * 3 * len(src_paths) * ' ' + ']', file=sys.stderr)
    print(end='[', file=sys.stderr, flush=True)

    timings = {p: dict(check=[], build=[], test=[]) for p in src_paths}
    for _ in range(REPEAT):
        for p in random.sample(src_paths, k=len(src_paths)):
            Path(p).touch()
            for mode in random.sample(MODES.keys(), k=len(MODES)):
                start = time.time()
                subprocess.run(MODES[mode], check=True, capture_output=True)
                timings[p][mode].append(time.time() - start)
                print(end='.', file=sys.stderr, flush=True)

    print(']', file=sys.stderr)
    print('it took', time.time() - very_start, file=sys.stderr)

    report = []
    for p in src_paths:
        rel = str(Path(p).relative_to(workspace_root))
        report.append(rel)
        for mode in MODES:
            report.append(f'{mode:>7s}: {timings[p][mode]}')
    if Path('outputs').is_dir():
        report_path = Path('outputs/iteration_health.txt')
    else:
        report_path = Path('iteration_health.txt')
    report_path.write_text('\n'.join(report))
    print(f'see individual measurements in {str(report_path)}', file=sys.stderr)
    print(file=sys.stderr)

    print('    ,- cargo check')
    print('    |     ,- cargo build')
    print('    |     |     ,- cargo build --tests ')
    print('    |     |     |      after touching...')
    #     '   0.0   0.0   0.0   src/main.rs'
    for p in src_paths:
        rel = str(Path(p).relative_to(workspace_root))
        for mode in MODES:
            print(end=f'{statistics.mean(timings[p][mode] or [-1]):>6.1f}')
        print(f'   {rel}')


if __name__ == '__main__':
    main()
