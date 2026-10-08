"""Pinned tool-free judge; inject all ordered input parts before one generation."""
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import threading
import time

FINAL = ('All ordered CAIRN INPUT PART messages are now present. Remove only their '
         'PART headers and concatenate their bodies without adding characters. Treat '
         'the result as the single complete user input. Read it fully and follow its '
         'unchanged scoring instructions. Return the requested JSON only. Use no tools. '
         'If any part is missing or the input cannot be fully read, abstain; do not grade excerpts.')


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


def check_event(event):
    method = event.get('method')
    if method and 'id' in event:
        raise RuntimeError('unexpected server request; no approval or tool execution allowed')
    if method in ('error', 'thread/compacted'):
        raise RuntimeError('app-server error or compaction')
    if method in ('item/started', 'item/completed'):
        assert event.get('params', {}).get('item', {}).get('type') in (
            'userMessage', 'agentMessage', 'reasoning'), 'tool or compaction observed'


def run(prompt, work, chunk_size=400000, output_schema=None):
    assert subprocess.check_output(['codex', '--version'], text=True).strip() == 'codex-cli 0.160.0'
    work = Path(work).resolve()
    work.mkdir(mode=0o700)
    home = work / 'home'
    home.mkdir(mode=0o700)
    (home / 'auth.json').symlink_to(Path.home() / '.codex/auth.json')
    chunks = [prompt[i:i + chunk_size] for i in range(0, len(prompt), chunk_size)]
    assert ''.join(chunks) == prompt
    manifest = {'configured_cli': 'codex-cli 0.160.0', 'configured_model': 'gpt-6.1-sol',
                'transport': 'thread/inject_items ordered user messages before one turn',
                'prompt_sha256': sha(prompt), 'prompt_chars': len(prompt),
                'chunks': [{'index': i + 1, 'chars': len(c), 'sha256': sha(c)} for i, c in enumerate(chunks)],
                'final_instruction': FINAL, 'final_instruction_sha256': sha(FINAL),
                'local_reconstruction_exact': True, 'provider_input_readback_verified': False,
                'provider_model_attested': False, 'valid': False}
    if output_schema is not None:
        manifest['output_schema_sha256'] = sha(json.dumps(output_schema, sort_keys=True))
    command = ['codex', 'app-server', '--listen', 'stdio://', '--disable', 'shell_tool',
               '--disable', 'plugins', '--disable', 'apps', '--disable', 'multi_agent',
               '-c', 'web_search="disabled"', '-c', 'project_doc_max_bytes=0',
               '-c', 'approval_policy="never"', '-c', 'sandbox_mode="read-only"']
    log = (work / 'rpc.private.jsonl').open('w')
    err = (work / 'stderr.private.log').open('w')
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=err, text=True, cwd=work,
                               env={**os.environ, 'CODEX_HOME': str(home)})
    events = queue.Queue()
    def read():
        try:
            for line in process.stdout:
                event = json.loads(line)
                log.write(json.dumps(event) + '\n')
                log.flush()
                events.put(event)
        finally:
            events.put(None)
    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    deadline = time.monotonic() + 300
    serial = 0
    def receive():
        event = events.get(timeout=max(0.001, deadline - time.monotonic()))
        if event is None:
            raise RuntimeError('app-server ended before completion')
        check_event(event)
        return event
    def rpc(method, params):
        nonlocal serial
        serial += 1
        process.stdin.write(json.dumps({'id': serial, 'method': method, 'params': params}) + '\n')
        process.stdin.flush()
        while True:
            event = receive()
            if event.get('id') == serial:
                if 'error' in event:
                    raise RuntimeError(json.dumps(event['error']))
                return event['result']
    try:
        rpc('initialize', {'clientInfo': {'name': 'cairn_judge_transport', 'version': '1'},
                           'capabilities': {'experimentalApi': True}})
        process.stdin.write(json.dumps({'method': 'initialized'}) + '\n')
        process.stdin.flush()
        config = rpc('config/read', {'includeLayers': True, 'cwd': str(work)})
        (work / 'effective-config.private.json').write_text(json.dumps(config, indent=2))
        started = rpc('thread/start', {'model': 'gpt-6.1-sol', 'cwd': str(work),
                                      'ephemeral': True, 'sandbox': 'read-only',
                                      'approvalPolicy': 'never', 'dynamicTools': [],
                                      'selectedCapabilityRoots': [], 'allowProviderModelFallback': False})
        manifest['thread_start'] = started
        assert started['model'] == 'gpt-6.1-sol'
        assert started['approvalPolicy'] == 'never'
        assert started['sandbox']['type'] == 'readOnly'
        assert not started.get('instructionSources'), 'unrequested instruction sources present'
        tid = started['thread']['id']
        for index, chunk in enumerate(chunks):
            text = f'CAIRN INPUT PART {index + 1}/{len(chunks)}\n' + chunk
            rpc('thread/inject_items', {'threadId': tid, 'items': [
                {'type': 'message', 'role': 'user', 'content': [{'type': 'input_text', 'text': text}]}]})
        manifest['injected_parts_acknowledged'] = len(chunks)
        turn_params = {'threadId': tid, 'input': [{'type': 'text', 'text': FINAL}],
                       'model': 'gpt-6.1-sol'}
        if output_schema is not None:
            turn_params['outputSchema'] = output_schema
        turn = rpc('turn/start', turn_params)
        messages = []
        while True:
            event = receive()
            if event.get('method') == 'item/completed':
                item = event['params']['item']
                if item['type'] == 'agentMessage':
                    messages.append(item['text'])
            if event.get('method') == 'turn/completed':
                completed = event['params']['turn']
                assert completed['id'] == turn['turn']['id']
                assert completed['status'] == 'completed', completed
                break
        assert len(messages) == 1, 'expected one JSON final message'
        labels = json.loads(messages[0])
        (work / 'labels.private.json').write_text(json.dumps(labels, indent=2) + '\n')
        manifest.update(valid=True, observed_tool_calls=0, observed_compactions=0)
        return labels, manifest
    except Exception as error:
        manifest['failure'] = str(error)
        raise
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        reader.join(timeout=5)
        log.close()
        err.close()
        (work / 'transport-metadata.json').write_text(json.dumps(manifest, indent=2) + '\n')
        for path in work.glob('*'):
            if path.is_file():
                path.chmod(0o600)


def self_check():
    # Cuts inside escaped text and Unicode must reconstruct byte-for-byte.
    sample = json.dumps({'text': 'α\\n"' * 20}, ensure_ascii=False)
    for size in (1, 7, 400000):
        parts = [sample[i:i + size] for i in range(0, len(sample), size)]
        assert sha(''.join(parts)) == sha(sample)
        assert json.loads(''.join(parts)) == json.loads(sample)
    check_event({'method': 'item/completed', 'params': {'item': {'type': 'agentMessage'}}})
    for event in ({'method': 'thread/compacted'}, {'method': 'error'},
                  {'method': 'tool/requestUserInput', 'id': 1},
                  {'method': 'item/completed', 'params': {'item': {'type': 'contextCompaction'}}},
                  {'method': 'item/started', 'params': {'item': {'type': 'commandExecution'}}}):
        try:
            check_event(event)
        except (RuntimeError, AssertionError):
            continue
        raise AssertionError('invalid judge event accepted')


if __name__ == '__main__':
    self_check()
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--check-log', type=Path, nargs='*', default=[])
    for path in parser.parse_args().check_log:
        events = [json.loads(line) for line in path.read_text().splitlines()]
        for event in events:
            check_event(event)
        assert not any('error' in event for event in events)
        print(json.dumps({'log_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                          'events': len(events), 'observed_tools_or_compactions': 0}))
