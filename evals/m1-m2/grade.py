"""Same frozen predicates, fresh isolated calibrated Codex judge per packet."""
import argparse, json, os, subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ["CAIRN_M2_OUT"])
if ROOT.resolve().is_relative_to(HERE.parents[1]):
    raise ValueError("Private judgments must be outside the repository")
RULES = (HERE / "judge-instructions.md").read_text()
RUBRIC = json.loads((HERE / "calibration-v4b.json").read_text())["predicates"]
PROTOCOL = "\n".join((HERE / ("protocol-" + version + ".md")).read_text()
                     for version in ("v4", "v8", "v9", "v10"))

def grade(packet, out):
    identity = packet.stem
    dest = out / (identity + '.json')
    if dest.exists():
        return
    home = out / (identity + '.home')
    home.mkdir(mode=0o700)
    (home / 'auth.json').symlink_to(Path.home() / '.codex/auth.json')
    prompt = RULES + '\nCALIBRATED PREDICATES\n' + json.dumps(RUBRIC) + '\nFROZEN PROTOCOL\n' + PROTOCOL + '\nCOMPLETE PACKET\n' + packet.read_text()
    raw, err, labels_file = [out / (identity + suffix) for suffix in ('.private.out', '.private.err', '.labels.private.json')]
    command = ['codex', 'exec', '-C', str(out), '-m', 'gpt-6.1-sol', '--json',
               '--ephemeral', '--ignore-user-config', '--ignore-rules', '--skip-git-repo-check',
               '--sandbox', 'read-only', '--disable', 'shell_tool', '--disable', 'plugins',
               '--disable', 'apps', '--disable', 'multi_agent', '--config', 'web_search="disabled"',
               '--output-last-message', str(labels_file), '-']
    result = {'id': identity, 'judge_valid': False, 'configured_model': 'gpt-6.1-sol', 'provider_model_attested': False}
    try:
        cli = subprocess.check_output(['codex', '--version'], text=True).strip()
        result['configured_cli'] = cli
        assert cli == 'codex-cli 0.160.0', 'calibrated CLI changed'
        with raw.open('w') as stdout, err.open('w') as stderr:
            process = subprocess.run(command, input=prompt, text=True, cwd=out,
                                     env={**os.environ, 'CODEX_HOME': str(home)}, stdout=stdout,
                                     stderr=stderr, timeout=300)
        result['judge_status'] = process.returncode
        if process.returncode != 0:
            raise RuntimeError('judge process failed')
        events = [json.loads(line) for line in raw.read_text().splitlines()]
        assert not any(e.get('type') == 'error' for e in events)
        assert not any(e.get('item', {}).get('type') in ('command_execution', 'mcp_tool_call', 'web_search') for e in events)
        labels = json.loads(labels_file.read_text())
        assert labels['id'] == identity
        assert set(labels['completion']) == {'control', 'treatment'}
        assert all(v in ('yes', 'no', 'unknown') for v in labels['completion'].values())
        assert labels['useful_application'] in ('yes', 'no', 'unknown', None)
        assert isinstance(labels['delivery']['complete'], bool)
        claims = labels['delivery']['claims']
        assert len({c['id'] for c in claims}) == len(claims)
        assert all(c[k] in ('yes', 'no', 'unknown') for c in claims for k in ('relevant', 'supported'))
        assert labels['harm']['high_impact'] in ('yes', 'no', 'unknown')
        assert isinstance(labels['harm']['lower_impact'], bool)
        assert all(labels['repeated'][a] is None or type(labels['repeated'][a]) is int and labels['repeated'][a] >= 0 for a in ('control', 'treatment'))
        result.update(labels, judge_valid=True, tool_calls=0)
    except Exception as error:
        result['failure'] = type(error).__name__
    dest.write_text(json.dumps(result, indent=2) + '\n')
    for file in (dest, raw, err, labels_file):
        if file.exists():
            file.chmod(0o600)
    print(identity, 'judge_valid', result['judge_valid'], flush=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--packets', type=Path, default=ROOT / 'packets')
    parser.add_argument('--out', type=Path, default=ROOT / 'judgments')
    parser.add_argument('--jobs', type=int, default=1)
    parser.add_argument('--retry-failed', action='store_true')
    args = parser.parse_args()
    args.out.mkdir(mode=0o700, exist_ok=True)
    if args.out.stat().st_mode & 0o077:
        raise ValueError("Judgment output directory must be private (0700)")
    if args.retry_failed:
        for packet in args.packets.glob('*.json'):
            result = args.out / (packet.stem + '.json')
            if not result.exists() or json.loads(result.read_text()).get('judge_valid'):
                continue
            attempts = args.out / 'failed-attempts'
            attempts.mkdir(mode=0o700, exist_ok=True)
            index = 1
            while (attempts / (packet.stem + '-' + str(index))).exists():
                index += 1
            preserved = attempts / (packet.stem + '-' + str(index))
            preserved.mkdir(mode=0o700)
            for path in args.out.glob(packet.stem + '.*'):
                path.rename(preserved / path.name)
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        list(pool.map(lambda packet: grade(packet, args.out), sorted(args.packets.glob('*.json'))))

if __name__ == '__main__':
    main()
