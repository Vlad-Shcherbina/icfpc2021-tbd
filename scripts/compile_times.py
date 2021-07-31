'''
Usage:

python3 scripts/compile_times.py measure  # will take a long time
python3 scripts/compile_times.py render   # will create an HTML report
'''

import subprocess
import time
from pathlib import Path
import json
import sys
import html

RESULTS_PATH = Path('cache/results.json')

def main():
    res = subprocess.run(
        'git rev-list --first-parent main',
        shell=True, check=True, capture_output=True, universal_newlines=True)
    commits = res.stdout.splitlines()

    if sys.argv[1] == 'measure':
        measure(commits)
    elif sys.argv[1] == 'render':
        render(commits)


def render(commits):
    results = json.loads(RESULTS_PATH.read_text())
    s = []
    s.append('''
    <style>
        table {
            margin-top: 100px;
        }
        th.diag > div {
            transform: translate(5px, 0px) rotate(-30deg);
            width: 25px;
            white-space: nowrap;
        }
        td {
            /*border: solid 1px red;*/
        }
    </style>
    ''')
    s.append('<table>')
    s.append('<tr>')
    s.append('<th class="diag"><div>cargo check from scratch</div></th>')
    s.append('<th class="diag"><div>cargo build</div></th>')
    s.append('<th class="diag"><div>cargo check incremental</div></th>')
    s.append('<th class="diag"><div>cargo build incremental</div></th>')
    s.append('</tr>')
    for commit in commits:
        s.append('<tr>')
        res = subprocess.run('git log --format="%an%n%B" -1 ' + commit,
            shell=True, check=True, capture_output=True, universal_newlines=True)
        author, message = res.stdout.split('\n', 1)
        message_first_line = message.split('\n')[0]

        if commit == commits[-1]:
            cargo_toml_diff = ''
            cargo_lock_diff = ''
        else:
            cargo_toml_diff = subprocess.run(f'git diff {commit}~1 {commit} -- Cargo.toml',
                shell=True, check=True, capture_output=True, universal_newlines=True).stdout
            cargo_lock_diff = subprocess.run(f'git diff {commit}~1 {commit} -- Cargo.lock',
                shell=True, check=True, capture_output=True, universal_newlines=True).stdout

        print(author)
        print(message_first_line)
        print()

        rs = [r for r in results if r['commit'] == commit]
        print(len(rs))
        if len(rs) > 0:
            [r] = rs
            def fmt(x):
                if x is not None:
                    return f'{x:.2f}'
                else:
                    return '-'
            s.append(f'<td>{fmt(r["check"])}</td>')
            s.append(f'<td>{fmt(r["build"])}</td>')
            # s.append(f'<td>{fmt(r["test"])}</td>')
            # s.append(f'<td>{fmt(r["test2"])}</td>')
            s.append(f'<td>{fmt(r["check2"])}</td>')
            s.append(f'<td>{fmt(r["build2"])}</td>')
        else:
            for _ in range(4):
                s.append('<td></td>')

        if cargo_toml_diff:
            s.append(f'<td title="{html.escape(cargo_toml_diff)}">toml</td>')
        else:
            s.append('<td></td>')
        if cargo_lock_diff:
            s.append(f'<td title="{html.escape(cargo_lock_diff)}">lock</td>')
        else:
            s.append('<td></td>')

        link = 'https://github.com/Vlad-Shcherbina/icfpc2021-tbd/commit/' + commit
        s.append(f'<td title="{html.escape(message)}"><a href="{link}">{message_first_line}</a></td>')
        s.append(f'<td>{author}</td>')

        s.append('</tr>')

    s.append('</table>')
    p = Path('outputs/compile_times.html')
    p.write_text('\n'.join(s))
    print('see', p)


def measure(commits):
    if RESULTS_PATH.exists():
        results = json.loads(RESULTS_PATH.read_text())
    else:
        results = []

    for i, commit in enumerate(commits):
        result = dict(commit=commit)

        print(i, commit, '*' * 50)
        if any(r['commit'] == commit for r in results):
            print('skip')
            continue
        subprocess.run('git checkout ' + commit, shell=True, check=True)

        subprocess.run('cargo clean', shell=True)

        print('****** fetch')
        subprocess.run(
            'cargo fetch --locked',
            shell=True, check=True)

        print('****** check')
        result['check'] = measure_cmd('cargo check --offline')

        print('****** build')
        result['build'] = measure_cmd('cargo build --offline')

        print('****** test')
        result['test'] = measure_cmd('cargo test --offline')

        print('****** test2')
        result['test2'] = measure_cmd('cargo test --offline')

        p = Path('src/main.rs')
        p.write_text(p.read_text())

        print('****** check2')
        result['check2'] = measure_cmd('cargo check --offline')

        print('****** build2')
        result['build2'] = measure_cmd('cargo build --offline')

        print(result)
        results.append(result)
        RESULTS_PATH.write_text(json.dumps(results))


def measure_cmd(cmd):
    start = time.time()
    res = subprocess.run(cmd, shell=True)
    if res.returncode:
        return None
    return time.time() - start


if __name__ == '__main__':
    main()
