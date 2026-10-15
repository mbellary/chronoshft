#!/usr/bin/env python3
"""Capture bound automated tests without overwriting observations or evidence."""

import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import signal
import subprocess
import sys
import time

if __package__:
    from .validate_register import ROOT, read_yaml, local_path
    from .validation_runtime import (PinnedDefinitions, aggregate, file_digest, git, immutable_copy, immutable_write,
        publish_run, reference, utc_now, validate_record, yaml_text)
else:
    from validate_register import ROOT, read_yaml, local_path
    from validation_runtime import (PinnedDefinitions, aggregate, file_digest, git, immutable_copy, immutable_write,
        publish_run, reference, utc_now, validate_record, yaml_text)


def process(argv, cwd, stdout, stderr, timeout, environment=None):
    stdout.parent.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    code, error, aborted, details = None, False, False, ''
    with stdout.open('wb') as out, stderr.open('wb') as err:
        child = None
        try:
            child = subprocess.Popen(argv, cwd=cwd, stdout=out, stderr=err, env=environment, start_new_session=(os.name == 'posix'))
            code = child.wait(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as exc:
            aborted = isinstance(exc, KeyboardInterrupt)
            error, details = True, 'Execution interrupted' if aborted else 'Execution timed out'
            if child is not None:
                if os.name == 'posix':
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    child.kill()
                child.wait()
            err.write((details + '\n').encode())
        except OSError as exc:
            error, details = True, str(exc)
            err.write((details + '\n').encode())
    return code, error, aborted, max(0, time.monotonic() - started), details


def cpu_description():
    path = Path('/proc/cpuinfo')
    if path.is_file():
        for line in path.read_text().splitlines():
            if line.lower().startswith('model name'):
                return line.split(':', 1)[1].strip()
    return platform.processor() or platform.machine() or 'CPU identifier unavailable (recorded deviation required)'


def capture(root, plan, shared_ci=False):
    root = root.resolve()
    plan = deepcopy(plan)
    checkout_commit = git(root, 'rev-parse', 'HEAD').decode().strip()
    if plan['implementation_commit'] == 'HEAD':
        plan['implementation_commit'] = checkout_commit
    if plan['experiment']['registry_commit'] == 'HEAD':
        plan['experiment']['registry_commit'] = checkout_commit
    validate_record(root, 'capture-plan', plan)
    resolver = PinnedDefinitions(root)
    experiment = resolver.record(plan['experiment'])
    commit = plan['experiment']['registry_commit']
    implementation = resolver.commit(plan['implementation_commit'])
    if git(root, 'rev-parse', 'HEAD').decode().strip() != implementation:
        raise ValueError('Implementation commit must be the current checkout HEAD')
    source_paths = ['.', ':(exclude)validation/executions/**', ':(exclude)validation/evidence-manifests/**',
                    ':(exclude)validation/decisions/**', ':(exclude)validation/artifacts/**']
    if git(root, 'status', '--porcelain', '--untracked-files=all', '--', *source_paths).strip():
        raise ValueError('Capture requires clean committed source; commit framework/bindings/inputs before a real run')
    if experiment['lifecycle']['state'] not in {'ready', 'running', 'reviewing', 'completed'}:
        raise ValueError('Pinned experiment is not ready for execution')
    cwd = local_path(root, plan['working_directory'])
    if not cwd.is_dir():
        raise ValueError('Declared working directory does not exist')
    tests = [resolver.record(reference(ref['id'], ref['revision'], commit)) for ref in experiment['tests']]
    if any(test['mandatory'] and test['execution_binding']['state'] != 'bound' for test in tests):
        raise ValueError('All mandatory methods must be bound before experiment execution')
    selected = {item['id']: item for item in plan['tests']}
    expected = {test['id']: test for test in tests}
    if len(selected) != len(plan['tests']) or not set(selected) <= set(expected):
        raise ValueError('Capture plan has duplicate or foreign test identities')
    for identity, item in selected.items():
        test = expected[identity]
        if shared_ci and test['classification'] == 'performance':
            raise ValueError('Shared CI cannot execute performance acceptance tests; use a controlled environment')
        if test['execution_type'] != 'automated' or item['revision'] != test['revision']:
            raise ValueError('Capture plans select exact automated test revisions; use register_execution.py for manual observations')
        if test['execution_binding']['state'] != 'bound' or item['command'] != shlex.split(test['execution_binding']['entrypoint']):
            raise ValueError(f'{identity}: command does not match the pinned binding')
        resolver.blob(implementation, test['execution_binding']['implementation_path'])
        for artifact in item['artifacts']:
            local_path(cwd, artifact['path'])
    if shared_ci:
        plan['deviations'] = [*plan['deviations'], 'Shared CI environment; not controlled performance acceptance evidence']
    inputs = []
    for item in plan['inputs']:
        path = local_path(root, item['path'])
        data = path.read_bytes()
        if data != resolver.blob(implementation, item['path']):
            raise ValueError('Inputs must match their committed source bytes')
        inputs.append({'name': item['name'], 'sha256': hashlib.sha256(data).hexdigest(), 'reference': 'repo://' + item['path']})
    lockfiles = []
    for relative in plan['dependency_lockfiles']:
        data = local_path(root, relative).read_bytes()
        if data != resolver.blob(implementation, relative):
            raise ValueError('Lockfile differs from its committed source')
        lockfiles.append({'path': relative, 'sha256': hashlib.sha256(data).hexdigest()})
    run_id = plan['run_id']
    if (root / 'validation/executions' / f'{run_id}.yaml').exists():
        raise ValueError('Run IDs cannot be reused')
    bundle = root / 'evidence' / run_id
    bundle.mkdir(parents=True, exist_ok=False)
    (bundle / 'capture-state.json').write_text(json.dumps({'state': 'capturing', 'run_id': run_id}, indent=2) + '\n')
    immutable_write(bundle / 'plan.yaml', yaml_text(plan))
    evidence = []
    started = utc_now()
    validations = experiment['validates']
    command_environment = {name: os.environ[name] for name in ('PATH', 'HOME', 'SYSTEMROOT', 'WINDIR', 'TMPDIR', 'TEMP', 'TMP', 'CARGO_HOME', 'RUSTUP_HOME') if name in os.environ}
    command_environment.update({'LC_ALL': 'C', 'TZ': 'UTC'})
    command_environment.update(plan['environment_variables'])

    def retain(path, kind, scope, applicability):
        digest = file_digest(path)
        size = path.stat().st_size
        repository = plan['retention'] == 'repository' and size <= plan['max_repository_artifact_bytes']
        if repository:
            relative = Path('validation/artifacts/sha256') / digest
            target = root / relative
            if target.exists():
                if target.stat().st_size != size or file_digest(target) != digest:
                    raise ValueError('Existing content-addressed artifact has been modified')
            else:
                try:
                    immutable_copy(path, target)
                except FileExistsError:
                    if target.stat().st_size != size or file_digest(target) != digest:
                        raise ValueError('Concurrent content-addressed artifact differs from its digest')
        else:
            relative = path.relative_to(root)
        identity = f"EVD-{run_id.removeprefix('RUN-')}-{len(evidence) + 1:03}"
        evidence.append({'schema_version': 1, 'id': identity, 'kind': kind,
            'provenance': {'execution_id': run_id, 'description': scope, 'source_version': implementation, 'retrieved_at': utc_now()},
            'artifact': {'uri': 'repo://' + relative.as_posix(), 'sha256': digest, 'size_bytes': size,
                         'media_type': 'application/json' if path.suffix == '.json' else 'text/plain' if path.suffix in {'.log', '.yaml'} else 'application/octet-stream',
                         'retention_location': 'Repository content-addressed artifact' if repository else 'Run bundle; publish and supply a local mirror for verification',
                         'access': 'Repository checkout' if repository else 'Retained run bundle or explicit artifact mirror'},
            'supports': [{'validation': reference(val['id'], val['revision'], commit), 'scope': scope} for val in applicability], 'supersedes': None})
        path.chmod(0o444)
        return identity

    retain(bundle / 'plan.yaml', 'other', 'Exact predeclared capture plan; not a technical conclusion', validations)
    version_out, version_err = bundle / 'toolchain/stdout.log', bundle / 'toolchain/stderr.log'
    version_code, version_error, aborted, _, version_details = process(plan['toolchain_command'], root, version_out, version_err, plan['timeout_seconds'], command_environment)
    retain(version_out, 'other', 'Actual toolchain command output', validations)
    retain(version_err, 'other', 'Actual toolchain command error output', validations)
    tool_errors = []
    if version_error or version_code != 0:
        tool_errors.append('Toolchain probe failed: ' + (version_details or str(version_code)))
    environment = {'toolchain': (version_out.read_text(errors='replace') + version_err.read_text(errors='replace')).strip() or 'Toolchain probe produced no version output',
                   'os': platform.platform(), 'cpu': cpu_description(), 'configuration': plan['configuration'] + '\nProcess environment: ' + json.dumps(command_environment, sort_keys=True), 'deviations': plan['deviations']}
    results, source_changed, stop = [], False, bool(tool_errors) or aborted
    for test in tests:
        identity = test['id']
        case = {'test': reference(identity, test['revision'], commit), 'outcome': 'skipped', 'details': 'Not selected in this capture plan',
                'command': selected[identity]['command'] if identity in selected else [], 'exit_code': None, 'duration_seconds': 0, 'evidence_ids': []}
        if identity not in selected or stop:
            if stop:
                case['details'] = 'Not launched after interruption, infrastructure error or source/input change'
            results.append(case)
            continue
        item = selected[identity]
        out, err = bundle / identity / 'stdout.log', bundle / identity / 'stderr.log'
        code, error, interrupted, duration, details = process(item['command'], cwd, out, err, plan['timeout_seconds'], command_environment)
        aborted = aborted or interrupted
        outcome = 'error' if error else 'passed' if code == 0 else 'failed' if code in item['failure_exit_codes'] else 'error'
        case.update(outcome=outcome, details=details or f'Bound process exited with code {code}', exit_code=code, duration_seconds=duration)
        case['evidence_ids'] = [retain(out, 'test_report', f'{identity} standard output; observation, not automatic closure evidence', test['validations']),
                                retain(err, 'test_report', f'{identity} standard error', test['validations'])]
        for index, artifact in enumerate(item['artifacts'], 1):
            try:
                source = local_path(cwd, artifact['path'])
                destination = bundle / identity / f'artifact-{index:03}'
                immutable_copy(source, destination)
                case['evidence_ids'].append(retain(destination, artifact['kind'], f"{identity} declared output {artifact['path']}", test['validations']))
            except (OSError, ValueError) as exc:
                case['outcome'] = 'error'
                case['details'] += f'; required artifact unavailable: {exc}'
        tracked_changed = bool(git(root, 'diff', 'HEAD', '--', *source_paths).strip())
        try:
            input_changed = any(file_digest(local_path(root, value['reference'].removeprefix('repo://'))) != value['sha256'] for value in inputs)
        except OSError:
            input_changed = True
        source_changed = source_changed or tracked_changed or input_changed
        if source_changed:
            tool_errors.append('Tracked source or a pinned input changed during execution')
        if case['outcome'] == 'error':
            tool_errors.append(f"{identity}: {case['details']}")
        results.append(case)
        stop = source_changed or aborted
    ended = utc_now()
    run = {'schema_version': 1, 'id': run_id, 'started_at': started, 'ended_at': ended,
           'experiment': plan['experiment'], 'working_directory': plan['working_directory'], 'tests': results,
           'outcome': aggregate(results, aborted, source_changed, bool(tool_errors)),
           'procedure': ['Working directory: ' + plan['working_directory'], 'Toolchain probe: ' + shlex.join(plan['toolchain_command'])] + [shlex.join(case['command']) for case in results if case['outcome'] != 'skipped'],
           'source': {'implementation_commit': implementation, 'registry_commit': commit, 'dirty': False, 'changed_during_run': source_changed, 'patch_evidence_id': None, 'dependency_lockfiles': lockfiles},
           'environment': environment, 'inputs': inputs, 'random_seeds': plan['random_seeds'], 'instrumentation': plan['instrumentation'],
           'evidence_ids': [], 'failures': [f"{case['test']['id']}: {case['details']}" for case in results if case['outcome'] == 'failed'],
           'skips': [f"{case['test']['id']}: {case['details']}" for case in results if case['outcome'] == 'skipped'],
           'tool_errors': tool_errors, 'supersedes': None}
    report_id = f"EVD-{run_id.removeprefix('RUN-')}-{len(evidence) + 1:03}"
    run['evidence_ids'] = [item['id'] for item in evidence] + [report_id]
    immutable_write(bundle / 'results.json', json.dumps({'run': run, 'capture_plan': plan}, indent=2) + '\n')
    retained_report = retain(bundle / 'results.json', 'test_report', 'Structured outcomes and reproduction metadata; review required', validations)
    if retained_report != report_id:
        raise ValueError('Structured report identity differs from the final manifest')
    immutable_write(bundle / 'manifests' / f'{run_id}.yaml', yaml_text(run))
    for item in evidence:
        immutable_write(bundle / 'manifests/evidence' / f"{item['id']}.yaml", yaml_text(item))
    publish_run(root, run, evidence)
    (bundle / 'capture-state.json').write_text(json.dumps({'state': 'published', 'run_id': run_id, 'outcome': run['outcome']}, indent=2) + '\n')
    return run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--run-id', help='unique identity override, recorded in the resolved pre-execution plan')
    parser.add_argument('--committed-plan', action='store_true', help='require a tracked plan under validation/execution-plans at HEAD')
    parser.add_argument('--shared-ci', action='store_true', help='reject performance-class cases on shared CI')
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        plan_path = args.plan.resolve()
        if args.committed_plan:
            if not plan_path.is_relative_to(root / 'validation/execution-plans'):
                raise ValueError('CI requires a committed plan under validation/execution-plans')
            if git(root, 'show', 'HEAD:' + plan_path.relative_to(root).as_posix()) != plan_path.read_bytes():
                raise ValueError('The plan differs from the committed source')
        plan = read_yaml(plan_path)
        if args.run_id:
            plan['run_id'] = args.run_id
        run = capture(root, plan, shared_ci=args.shared_ci)
    except Exception as exc:
        print(f'Capture failed: {exc}. Retain any reserved run bundle; do not reuse its RUN ID.', file=sys.stderr)
        return 1
    print(f"Recorded {run['id']}: {run['outcome']}. Evidence retained; no validation decision created.")
    return 0 if run['outcome'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
