"""Pinned definitions, append-only publication, artifact and review verification."""

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile

from jsonschema import Draft202012Validator
import yaml

if __package__:
    from .validate_register import FORMATS, UniqueKeyLoader, read_yaml, local_path
else:
    from validate_register import FORMATS, UniqueKeyLoader, read_yaml, local_path

KINDS = {'VAL': ('validation', 'register/validations'), 'EXP': ('experiment', 'register/experiments'),
         'TST': ('test', 'register/tests'), 'RUN': ('execution', 'executions'),
         'EVD': ('evidence', 'evidence-manifests'), 'VDEC': ('decision', 'decisions')}


def utc_now():
    return datetime.now(timezone.utc).isoformat(timespec='microseconds').replace('+00:00', 'Z')


def instant(value):
    return datetime.fromisoformat(value.replace('Z', '+00:00'))


def git(root, *arguments):
    result = subprocess.run(['git', '-C', str(root), *arguments], capture_output=True)
    if result.returncode:
        raise ValueError('Git reference/content unavailable: ' + ' '.join(arguments[:2]))
    return result.stdout


def yaml_text(data):
    return yaml.safe_dump(data, sort_keys=False, allow_unicode=True, width=100)


def load_text(text):
    return yaml.load(text, Loader=UniqueKeyLoader)


def validate_record(root, kind, record):
    schema = json.loads((root / 'validation/schemas' / f'{kind}.schema.json').read_text())
    Draft202012Validator.check_schema(schema)
    Draft202012Validator(schema, format_checker=FORMATS).validate(record)


def file_digest(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def immutable_write(path, content):
    """Publish a fully written file with an exclusive hard link; never replace."""
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix='.publish-', dir=path.parent)
    try:
        with os.fdopen(descriptor, 'wb') as stream:
            stream.write(content.encode() if isinstance(content, str) else content)
            stream.flush()
            os.fsync(stream.fileno())
            os.fchmod(stream.fileno(), 0o444)
        os.link(temporary, path)
    finally:
        os.unlink(temporary)


def immutable_copy(source, destination):
    """Retain large artifacts without loading them into memory or replacing files."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix='.publish-', dir=destination.parent)
    try:
        with os.fdopen(descriptor, 'wb') as output, source.open('rb') as stream:
            shutil.copyfileobj(stream, output, length=1024 * 1024)
            output.flush()
            os.fsync(output.fileno())
            os.fchmod(output.fileno(), 0o444)
        os.link(temporary, destination)
    finally:
        os.unlink(temporary)


def load_records(root):
    records = {}
    for kind, directory in KINDS.values():
        for path in sorted((root / 'validation' / directory).glob('*.yaml')):
            record = read_yaml(path)
            validate_record(root, kind, record)
            if path.stem != record['id']:
                raise ValueError('Registry filename must match the record identity')
            if record['id'] in records:
                raise ValueError(f"Duplicate record: {record['id']}")
            records[record['id']] = record
    return records


def definition(record):
    return {key: value for key, value in record.items()
            if key not in {'lifecycle', 'current_decision_id', 'prior_decision_ids', 'revision_history'}}


class PinnedDefinitions:
    def __init__(self, root):
        self.root = root.resolve()
        self.cache = {}

    def commit(self, value):
        if not re.fullmatch('[0-9a-f]{40}', value):
            raise ValueError('An exact 40-character Git commit is required')
        resolved = git(self.root, 'rev-parse', '--verify', value + '^{commit}').decode().strip()
        if resolved != value:
            raise ValueError('Git reference must identify the exact commit')
        return value

    def blob(self, commit, path):
        self.commit(commit)
        local_path(self.root, path)  # reject paths escaping the repository
        return git(self.root, 'show', f'{commit}:{path}')

    def record(self, reference):
        identity, revision, commit = reference['id'], reference['revision'], reference['registry_commit']
        key = identity, revision, commit
        if key in self.cache:
            return self.cache[key]
        prefix = identity.split('-')[0]
        if prefix not in {'VAL', 'EXP', 'TST'}:
            raise ValueError(f'Not a versioned definition: {identity}')
        kind, folder = KINDS[prefix]
        record = load_text(self.blob(commit, f'validation/{folder}/{identity}.yaml'))
        if record['revision'] != revision:
            plural = {'VAL': 'validations', 'EXP': 'experiments', 'TST': 'tests'}[prefix]
            record = load_text(self.blob(commit, f'validation/revisions/{plural}/{identity}/r{revision}.yaml'))
        if record['id'] != identity or record['revision'] != revision:
            raise ValueError(f'Pinned definition does not match {identity} r{revision}')
        schema = json.loads(self.blob(commit, f'validation/schemas/{kind}.schema.json'))
        Draft202012Validator(schema, format_checker=FORMATS).validate(record)
        self.cache[key] = record
        return record


def reference(identity, revision, commit):
    return {'id': identity, 'revision': revision, 'registry_commit': commit}


def artifact_path(root, evidence, artifact_map=None, artifact_root=None):
    identity = evidence['id']
    if artifact_map and identity in artifact_map:
        return Path(artifact_map[identity]).resolve()
    uri = evidence['artifact']['uri']
    if not uri.startswith('repo://'):
        raise ValueError(f'{identity}: external artifact needs an explicit local mirror in --artifact-map')
    return local_path((artifact_root or root).resolve(), uri.removeprefix('repo://'))


def verify_artifact(root, evidence, artifact_map=None, artifact_root=None):
    path = artifact_path(root, evidence, artifact_map, artifact_root)
    if not path.is_file():
        raise ValueError(f"{evidence['id']}: artifact is unavailable: {path}")
    expected = evidence['artifact']
    if path.stat().st_size != expected['size_bytes'] or file_digest(path) != expected['sha256']:
        raise ValueError(f"{evidence['id']}: artifact size/SHA-256 mismatch")
    return path


def verify_evidence(root, item, resolver, records, artifact_map=None, artifact_root=None):
    validate_record(root, 'evidence', item)
    verify_artifact(root, item, artifact_map, artifact_root)
    for support in item['supports']:
        resolver.record(support['validation'])
    producer = item['provenance']['execution_id']
    if producer and (producer not in records or not producer.startswith('RUN-')):
        raise ValueError(f"{item['id']}: missing producing run")


def aggregate(results, aborted=False, source_changed=False, infrastructure_error=False):
    if aborted:
        return 'aborted'
    outcomes = [case['outcome'] for case in results]
    if source_changed or infrastructure_error or 'error' in outcomes:
        return 'error'
    if 'failed' in outcomes:
        return 'failed'
    if not outcomes or 'skipped' in outcomes:
        return 'incomplete'
    return 'passed'


def verify_run(root, run, resolver, records, artifact_map=None, artifact_root=None):
    validate_record(root, 'execution', run)
    if instant(run['ended_at']) < instant(run['started_at']):
        raise ValueError('Execution ends before it starts')
    local_path(root, run['working_directory'])
    experiment = resolver.record(run['experiment'])
    if experiment['lifecycle']['state'] not in {'ready', 'running', 'reviewing', 'completed'}:
        raise ValueError('Execution references an experiment that was not ready')
    commit = run['experiment']['registry_commit']
    if run['source']['registry_commit'] != commit:
        raise ValueError(f"{run['id']}: inconsistent registry commits")
    resolver.commit(run['source']['implementation_commit'])
    expected = {test['id']: test['revision'] for test in experiment['tests']}
    actual = [case['test']['id'] for case in run['tests']]
    if len(actual) != len(set(actual)) or set(actual) != set(expected):
        raise ValueError(f"{run['id']}: every declared test needs one outcome, including omitted/skipped cases")
    for case in run['tests']:
        ref = case['test']
        if ref['registry_commit'] != commit or ref['revision'] != expected[ref['id']]:
            raise ValueError(f"{run['id']}: test revision does not match the pinned experiment")
        test = resolver.record(ref)
        if test['experiment'] != {'id': experiment['id'], 'revision': experiment['revision']}:
            raise ValueError(f"{run['id']}: test ownership differs from the pinned experiment")
        if case['outcome'] == 'skipped':
            if case['exit_code'] is not None:
                raise ValueError('Skipped cases cannot have an exit code')
        else:
            if test['execution_binding']['state'] != 'bound':
                raise ValueError(f"{ref['id']}: observed test method was unbound at the pinned revision")
            if test['execution_type'] == 'automated':
                if case['command'] != shlex.split(test['execution_binding']['entrypoint']):
                    raise ValueError(f"{ref['id']}: command differs from the pinned binding")
                resolver.blob(run['source']['implementation_commit'], test['execution_binding']['implementation_path'])
                if case['outcome'] == 'passed' and case['exit_code'] != 0:
                    raise ValueError('Automated pass requires exit code zero')
                if case['outcome'] == 'failed' and (case['exit_code'] is None or case['exit_code'] == 0):
                    raise ValueError('Automated assertion failure requires a nonzero exit code')
            elif case['command'] or case['exit_code'] is not None:
                raise ValueError('Manual observations must not claim an automated command/exit code')
            if not case['evidence_ids']:
                raise ValueError(f"{ref['id']}: an observed case needs retained evidence")
        if not set(case['evidence_ids']) <= set(run['evidence_ids']):
            raise ValueError('Case evidence must be listed in the run')
        for identity in case['evidence_ids']:
            item = records.get(identity)
            if item is None:
                raise ValueError(f'Missing case evidence: {identity}')
            supported = {(s['validation']['id'], s['validation']['revision'], s['validation']['registry_commit']) for s in item['supports']}
            required = {(v['id'], v['revision'], commit) for v in test['validations']}
            if not required <= supported:
                raise ValueError(f'{identity}: evidence scope does not cover its test applicability')
    expected_outcome = aggregate(run['tests'], run['outcome'] == 'aborted', run['source']['changed_during_run'], bool(run['tool_errors']))
    if run['outcome'] != expected_outcome:
        raise ValueError(f"{run['id']}: aggregate outcome does not match observations")
    for identity in run['evidence_ids']:
        item = records.get(identity)
        if item is None:
            raise ValueError(f'Missing run evidence: {identity}')
        if item['provenance']['execution_id'] not in {None, run['id']}:
            raise ValueError(f'{identity}: producing run differs from linked observation')
        verify_evidence(root, item, resolver, records, artifact_map, artifact_root)
    for item in run['inputs']:
        if not item['reference'].startswith('repo://'):
            raise ValueError('Input references must pin repository content; retain external inputs as evidence first')
        blob = resolver.blob(run['source']['implementation_commit'], item['reference'].removeprefix('repo://'))
        if hashlib.sha256(blob).hexdigest() != item['sha256']:
            raise ValueError('Input hash differs from the pinned implementation content')
    for lockfile in run['source']['dependency_lockfiles']:
        blob = resolver.blob(run['source']['implementation_commit'], lockfile['path'])
        if hashlib.sha256(blob).hexdigest() != lockfile['sha256']:
            raise ValueError('Dependency lockfile hash differs from the pinned content')
    if run['source']['dirty']:
        patch = records.get(run['source']['patch_evidence_id'])
        if not patch or patch['kind'] != 'source_patch' or patch['id'] not in run['evidence_ids']:
            raise ValueError('Dirty observations require a retained source-patch evidence item')


def read_artifact_map(path):
    if not path:
        return {}
    path = Path(path).resolve()
    mapping = read_yaml(path)
    if not isinstance(mapping, dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in mapping.items()):
        raise ValueError('Artifact map must map EVD IDs to local file paths')
    return {identity: str((path.parent / value).resolve()) for identity, value in mapping.items()}


def verify_decision(root, decision, resolver, records, artifact_map=None, artifact_root=None, require_current=False):
    validate_record(root, 'decision', decision)
    val = resolver.record(decision['validation'])
    current = records.get(val['id'])
    current_scope = require_current or (current is not None and current.get('current_decision_id') == decision['id'])
    if current_scope and (current is None or definition(current) != definition(val)):
        raise ValueError('Decision does not apply to the current validation definition')
    if decision['reviewer'].strip().lower() in {'', 'pending', 'unassigned', 'reviewer', 'codex', 'automation'}:
        raise ValueError('A named human reviewer attestation is required')
    if instant(decision['reviewed_at']) > instant(utc_now()):
        raise ValueError('Review timestamp is in the future')
    selected = []
    for identity in decision['execution_ids']:
        run = records.get(identity)
        if run is None or not identity.startswith('RUN-'):
            raise ValueError(f'Missing decision execution: {identity}')
        verify_run(root, run, resolver, records, artifact_map, artifact_root)
        if instant(run['ended_at']) > instant(decision['reviewed_at']):
            raise ValueError('Review predates its supporting execution')
        investigation = resolver.record(run['experiment'])
        if val['id'] not in {ref['id'] for ref in investigation['validates']}:
            raise ValueError('Decision cites an execution unrelated to its validation')
        selected.append(run)
    for identity in decision['evidence_ids']:
        if identity not in records or not identity.startswith('EVD-'):
            raise ValueError(f'Missing decision evidence: {identity}')
        item = records[identity]
        verify_evidence(root, item, resolver, records, artifact_map, artifact_root)
        if not any(s['validation'] == decision['validation'] for s in item['supports']):
            raise ValueError(f'{identity}: evidence does not support this validation revision/commit')
    if decision['disposition'] not in {'accepted', 'conditional'}:
        return
    for value in [decision['rationale'], decision['coverage_assessment'], *[decision['applicability'][key] for key in ('workload', 'hardware', 'dependencies')]]:
        if value.strip().lower() in {'', 'pending', 'unassigned', 'tbd'}:
            raise ValueError('Positive decision needs a concrete rationale, coverage and operating envelope')
    if decision['disposition'] == 'conditional' and not (decision['applicability']['limitations'] or decision['applicability']['exclusions']):
        raise ValueError('Conditional acceptance must state an explicit limitation or exclusion')
    if any(finding['invalidates_conclusion'] for finding in decision['unresolved_findings']):
        raise ValueError('Positive decision cannot retain an invalidating finding')
    assessments = decision['evidence_assessment']
    if len({a['requirement'] for a in assessments}) != len(assessments) or {a['requirement'] for a in assessments} != set(val['required_evidence']):
        raise ValueError('Every required evidence obligation needs an explicit assessment')
    for assessment in assessments:
        if not assessment['rationale'].strip():
            raise ValueError('Evidence assessments need a substantive rationale')
        if not set(assessment['evidence_ids']) <= set(decision['evidence_ids']):
            raise ValueError('Assessed evidence is not in the decision')
    registry_commit = decision['validation']['registry_commit']
    implementation_commit = decision['applicability']['implementation_commit']
    expected_tests = {ref['id']: ref['revision'] for ref in val['coverage']['tests'] if ref['mandatory']}
    expected_exps = {ref['id']: ref['revision'] for ref in val['coverage']['experiments'] if ref['mandatory']}
    if not expected_tests or not expected_exps:
        raise ValueError('Positive closure needs declared mandatory test/experiment coverage')
    seen = {}
    for run in selected:
        if run['source']['implementation_commit'] != implementation_commit or run['experiment']['registry_commit'] != registry_commit:
            raise ValueError('Positive decision mixes source/registry applicability boundaries')
        if run['source']['dirty'] or run['source']['changed_during_run'] or run['outcome'] in {'error', 'aborted'}:
            raise ValueError('Positive decision needs clean, valid execution observations')
        if run['experiment']['id'] not in expected_exps or run['experiment']['revision'] != expected_exps[run['experiment']['id']]:
            raise ValueError('Decision execution is outside mandatory experiment coverage')
        for case in run['tests']:
            if case['test']['id'] in expected_tests and case['test']['revision'] == expected_tests[case['test']['id']] and case['outcome'] != 'skipped':
                key = case['test']['id']
                if key not in seen or instant(run['ended_at']) > seen[key][0]:
                    seen[key] = instant(run['ended_at']), case['outcome'], run
    if set(seen) != set(expected_tests) or any(value[1] != 'passed' for value in seen.values()):
        raise ValueError('Positive decision lacks passing observations for every mandatory applicable test')
    for identity, (_, _, run) in seen.items():
        case = next(case for case in run['tests'] if case['test']['id'] == identity)
        if not set(case['evidence_ids']) <= set(decision['evidence_ids']):
            raise ValueError('Decision must retain the supporting artifacts for each mandatory passing case')
    for exp_id, revision in expected_exps.items():
        pinned = resolver.record(reference(exp_id, revision, registry_commit))
        if not any(case['test']['id'] in expected_tests for run in selected if run['experiment']['id'] == exp_id for case in run['tests'] if case['outcome'] == 'passed'):
            raise ValueError(f'{pinned["id"]}: mandatory experiment has no passing applicable observation')
    # Do not cherry-pick an earlier pass over a later contradictory run in the
    # same implementation, definition, input and environment boundary.
    for run in records.values():
        if not run['id'].startswith('RUN-') or run['source']['implementation_commit'] != implementation_commit or run['experiment']['registry_commit'] != registry_commit:
            continue
        for case in run['tests']:
            prior = seen.get(case['test']['id'])
            if not prior or case['outcome'] not in {'failed', 'error'}:
                continue
            chosen = prior[2]
            same_context = run['environment'] == chosen['environment'] and run['inputs'] == chosen['inputs'] and run['random_seeds'] == chosen['random_seeds']
            within_review = current_scope or instant(run['ended_at']) <= instant(decision['reviewed_at'])
            if same_context and within_review and instant(run['ended_at']) >= prior[0]:
                raise ValueError('A later contradictory observation invalidates the selected pass')


def publish_run(root, run, evidence, artifact_map=None, artifact_root=None, resume=False):
    records = load_records(root)
    new = {item['id']: item for item in evidence}
    if len(new) != len(evidence) or (not resume and (run['id'] in records or set(new) & set(records))):
        raise ValueError('RUN/EVD IDs cannot be overwritten or reused')
    for identity, item in {**new, run['id']: run}.items():
        if identity in records and records[identity] != item:
            raise ValueError('Resume cannot change an already published record')
    if not set(new) <= set(run['evidence_ids']):
        raise ValueError('Unlinked evidence must not be published with the run')
    candidate = {**records, **new, run['id']: run}
    resolver = PinnedDefinitions(root)
    verify_run(root, run, resolver, candidate, artifact_map, artifact_root)
    for item in evidence:
        if item['id'] not in records:
            immutable_write(root / 'validation/evidence-manifests' / f"{item['id']}.yaml", yaml_text(item))
    if run['id'] not in records:
        immutable_write(root / 'validation/executions' / f"{run['id']}.yaml", yaml_text(run))
