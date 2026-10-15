"""Isolated synthetic Git fixtures; these tests create no project RUN/EVD/VDEC records."""

from copy import deepcopy
from contextlib import redirect_stdout, redirect_stderr
from io import StringIO
from pathlib import Path
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import yaml

from scripts.capture_execution import capture
from scripts.check_validation_history import check_history, automatic_base
from scripts.review_validation import prepare, main as review_main
from scripts.validation_runtime import (PinnedDefinitions, aggregate, file_digest, immutable_write,
    load_records, publish_run, reference, utc_now, verify_artifact, verify_decision, verify_run, yaml_text)
from scripts.validate_register import ROOT


class RuntimeFixtures(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        shutil.copytree(ROOT / 'validation/schemas', self.root / 'validation/schemas')
        for folder in ['register/validations', 'register/experiments', 'register/tests', 'executions', 'evidence-manifests', 'decisions']:
            (self.root / 'validation' / folder).mkdir(parents=True, exist_ok=True)
        self.val = yaml.safe_load((ROOT / 'validation/register/validations/VAL-012.yaml').read_text())
        self.val['objective'] = 'Synthetic fixture only; not a ChronosHFT acceptance conclusion'
        self.val['required_evidence'] = ['Synthetic fixture observation coverage']
        self.val['coverage']['experiments'] = [{'id': 'EXP-001', 'revision': 2, 'mandatory': True}]
        self.val['coverage']['tests'] = [{'id': f'TST-{n:03}', 'revision': 1, 'mandatory': True} for n in range(1, 5)]
        for state, previous in [('ready', 'planned'), ('running', 'ready'), ('reviewing', 'running')]:
            self.val['lifecycle']['history'].append({'from': previous, 'to': state, 'date': '2026-10-15', 'actor': 'fixture', 'reason': 'Synthetic fixture only'})
        self.val['lifecycle']['state'] = 'reviewing'
        self.write('validation/register/validations/VAL-012.yaml', self.val)
        self.exp = yaml.safe_load((ROOT / 'validation/register/experiments/EXP-001.yaml').read_text())
        manual = self._testMethodName.startswith('test_manual')
        if manual:
            self.exp['method']['kind'] = 'documentary'
        self.exp['lifecycle']['state'] = 'ready'
        self.exp['lifecycle']['history'].append({'from': 'proposed', 'to': 'ready', 'date': '2026-10-15', 'actor': 'fixture', 'reason': 'Synthetic fixture bindings'})
        self.write('validation/register/experiments/EXP-001.yaml', self.exp)
        self.commands = {}
        for n in range(1, 5):
            identity = f'TST-{n:03}'
            test = yaml.safe_load((ROOT / f'validation/register/tests/{identity}.yaml').read_text())
            self.commands[identity] = [sys.executable, 'fixture.py', identity]
            test['execution_binding'] = {'state': 'bound', 'implementation_path': 'fixture.py', 'entrypoint': shlex.join(self.commands[identity])}
            if manual:
                test['execution_type'] = 'manual'
                test['execution_binding'] = {'state': 'bound', 'implementation_path': None, 'entrypoint': 'Synthetic source review checklist'}
            self.write(f'validation/register/tests/{identity}.yaml', test)
        (self.root / '.gitignore').write_text('evidence/\n__pycache__/\n')
        (self.root / 'fixture.txt').write_text('Synthetic input fixture\n')
        (self.root / 'fixture.py').write_text(
            'import os,sys,time\nmode=os.environ.get("FIXTURE_MODE", "passed")\n'
            'print("Synthetic fixture only",sys.argv[1],mode,flush=True)\n'
            'if mode=="timeout": time.sleep(2)\n'
            'if mode=="mutate": open("fixture.txt","w").write("changed")\n'
            'if mode=="alternating":\n'
            ' from pathlib import Path\n'
            ' marker=Path("evidence")/("fixture-marker-"+sys.argv[1])\n'
            ' if marker.exists(): sys.exit(1)\n'
            ' marker.parent.mkdir(exist_ok=True);marker.write_text("synthetic fixture state")\n'
            'sys.exit(1 if mode=="failed" else 2 if mode=="tool_error" else 0)\n')
        self.git('init', '-q')
        self.git('config', 'user.name', 'Synthetic Fixture')
        self.git('config', 'user.email', 'fixture@example.invalid')
        self.commit = self.commit_all()

    def git(self, *arguments):
        result = subprocess.run(['git', '-C', str(self.root), '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', *arguments], capture_output=True, check=True)
        return result.stdout.decode().strip()

    def commit_all(self):
        self.git('add', '.')
        self.git('commit', '-qm', 'Synthetic runtime fixture')
        return self.git('rev-parse', 'HEAD')

    def write(self, relative, data):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(yaml_text(data))

    def plan(self, mode='passed', selected=None, run_id='RUN-fixture-001', retention='repository'):
        selected = selected if selected is not None else list(self.commands)
        return {'schema_version': 1, 'run_id': run_id, 'experiment': reference('EXP-001', 2, self.commit),
                'implementation_commit': self.commit, 'working_directory': '.', 'toolchain_command': [sys.executable, '--version'],
                'configuration': 'Synthetic regression fixture; never product evidence', 'deviations': [],
                'inputs': [{'name': 'synthetic fixture', 'path': 'fixture.txt'}], 'dependency_lockfiles': [], 'random_seeds': ['fixture-seed'],
                'instrumentation': 'Synthetic stdout/stderr capture', 'timeout_seconds': 0.25 if mode == 'timeout' else 5,
                'retention': retention, 'max_repository_artifact_bytes': 100000,
                'environment_variables': {'FIXTURE_MODE': mode},
                'tests': [{'id': identity, 'revision': 1, 'command': self.commands[identity], 'failure_exit_codes': [1], 'artifacts': []} for identity in selected]}

    def decision(self, run):
        return {'schema_version': 1, 'id': 'VDEC-001', 'validation': reference('VAL-012', 3, self.commit),
                'disposition': 'accepted', 'rationale': 'Synthetic fixture attestation only', 'execution_ids': [run['id']],
                'evidence_ids': run['evidence_ids'], 'coverage_assessment': 'All four synthetic fixture cases',
                'evidence_assessment': [{'requirement': self.val['required_evidence'][0], 'evidence_ids': run['evidence_ids'], 'rationale': 'Synthetic fixture coverage'}],
                'unresolved_findings': [], 'applicability': {'implementation_commit': self.commit, 'workload': 'Synthetic fixture',
                'hardware': 'Fixture host', 'dependencies': 'Pinned synthetic Python fixture', 'exclusions': ['No product conclusion'], 'limitations': ['Unit fixture only']},
                'reviewer': 'Synthetic Human Reviewer', 'reviewed_at': utc_now(), 'architecture_spec_actions': ['No product adoption; unit fixture only'], 'follow_up': [], 'supersedes': None}

    def verify(self, run):
        records = load_records(self.root)
        verify_run(self.root, run, PinnedDefinitions(self.root), records)
        return records

    def test_passing_capture_preserves_reproduction_and_integrity(self):
        run = capture(self.root, self.plan())
        self.assertEqual(run['outcome'], 'passed')
        records = self.verify(run)
        self.assertEqual(len(run['tests']), 4)
        self.assertIn('Process environment:', run['environment']['configuration'])
        self.assertTrue(run['evidence_ids'])
        self.assertTrue(all(case['exit_code'] == 0 and case['evidence_ids'] for case in run['tests']))
        self.assertEqual(records['VAL-012']['current_decision_id'], None)
        self.assertFalse(list((self.root / 'validation/decisions').glob('*.yaml')))

    def test_head_pins_resolve_before_execution(self):
        plan = self.plan()
        plan['implementation_commit'] = plan['experiment']['registry_commit'] = 'HEAD'
        run = capture(self.root, plan)
        self.assertEqual(run['source']['implementation_commit'], self.commit)
        self.assertEqual(run['experiment']['registry_commit'], self.commit)
        self.assertEqual(plan['implementation_commit'], 'HEAD')

    def test_failure_is_registered_with_logs(self):
        run = capture(self.root, self.plan('failed'))
        self.assertEqual(run['outcome'], 'failed')
        self.assertEqual(len(run['failures']), 4)
        self.verify(run)

    def test_omitted_cases_are_skipped_not_passed(self):
        run = capture(self.root, self.plan(selected=['TST-001']))
        self.assertEqual(run['outcome'], 'incomplete')
        self.assertEqual(sum(case['outcome'] == 'skipped' for case in run['tests']), 3)
        self.verify(run)

    def test_capture_reports_actual_order_for_reordered_partial_selection(self):
        run = capture(self.root, self.plan(selected=['TST-004', 'TST-001']))
        self.assertEqual(run['procedure'][2:], [shlex.join(self.commands['TST-001']), shlex.join(self.commands['TST-004'])])
        self.assertEqual([case['test']['id'] for case in run['tests'] if case['outcome'] != 'skipped'], ['TST-001', 'TST-004'])
        self.verify(run)

    def test_repeated_bound_commands_are_preserved_in_the_procedure(self):
        path = self.root / 'validation/register/tests/TST-002.yaml'
        test = yaml.safe_load(path.read_text())
        self.commands['TST-002'] = list(self.commands['TST-001'])
        test['execution_binding']['entrypoint'] = shlex.join(self.commands['TST-002'])
        self.write('validation/register/tests/TST-002.yaml', test)
        self.commit = self.commit_all()
        run = capture(self.root, self.plan(selected=['TST-001', 'TST-002']))
        self.assertEqual(run['procedure'][2:], [shlex.join(self.commands['TST-001'])] * 2)
        self.verify(run)

    def test_timeout_is_error_with_retained_logs(self):
        run = capture(self.root, self.plan('timeout', selected=['TST-001']))
        self.assertEqual(run['outcome'], 'error')
        self.assertIn('timed out', run['tests'][0]['details'])
        self.verify(run)

    def test_tool_exit_is_not_an_assertion_failure(self):
        run = capture(self.root, self.plan('tool_error', selected=['TST-001']))
        self.assertEqual(run['tests'][0]['outcome'], 'error')
        self.assertTrue(run['tool_errors'])
        self.verify(run)

    def test_source_mutation_blocks_success(self):
        run = capture(self.root, self.plan('mutate'))
        self.assertTrue(run['source']['changed_during_run'])
        self.assertEqual(run['outcome'], 'error')
        self.assertEqual(sum(case['outcome'] == 'skipped' for case in run['tests']), 3)
        self.verify(run)

    def test_interrupt_preserves_aborted_observation(self):
        calls = 0
        def interrupted(argv, cwd, out, err, timeout, environment):
            nonlocal calls
            calls += 1
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text('Synthetic version' if calls == 1 else 'Synthetic interruption')
            err.write_text('')
            return (0, False, False, 0.01, '') if calls == 1 else (None, True, True, 0.01, 'Synthetic interruption')
        with patch('scripts.capture_execution.process', side_effect=interrupted):
            run = capture(self.root, self.plan())
        self.assertEqual(run['outcome'], 'aborted')
        self.verify(run)

    def test_ids_cannot_be_overwritten(self):
        run = capture(self.root, self.plan())
        before = (self.root / f"validation/executions/{run['id']}.yaml").read_bytes()
        with self.assertRaisesRegex(ValueError, 'cannot be reused'):
            capture(self.root, self.plan())
        self.assertEqual(before, (self.root / f"validation/executions/{run['id']}.yaml").read_bytes())

    def test_unbound_or_uncommitted_source_cannot_run(self):
        path = self.root / 'fixture.txt'
        path.write_text('Uncommitted input')
        with self.assertRaisesRegex(ValueError, 'clean committed source'):
            capture(self.root, self.plan())
        self.assertFalse((self.root / 'evidence').exists())

    def test_command_must_match_pinned_binding(self):
        plan = self.plan()
        plan['tests'][0]['command'] = [sys.executable, '-c', 'print("unbound")']
        with self.assertRaisesRegex(ValueError, 'does not match'):
            capture(self.root, plan)

    def test_artifact_hash_and_size_tampering_is_detected(self):
        run = capture(self.root, self.plan())
        records = load_records(self.root)
        item = records[run['tests'][0]['evidence_ids'][0]]
        path = self.root / item['artifact']['uri'].removeprefix('repo://')
        path.chmod(0o644)
        path.write_bytes(b'corrupted')
        with self.assertRaisesRegex(ValueError, 'mismatch'):
            verify_artifact(self.root, item)

    def test_external_evidence_requires_a_local_mirror(self):
        run = capture(self.root, self.plan())
        records = load_records(self.root)
        item = deepcopy(records[run['tests'][0]['evidence_ids'][0]])
        path = self.root / item['artifact']['uri'].removeprefix('repo://')
        item['artifact']['uri'] = 'https://artifacts.example.invalid/immutable-fixture'
        with self.assertRaisesRegex(ValueError, 'local mirror'):
            verify_artifact(self.root, item)
        self.assertEqual(verify_artifact(self.root, item, {item['id']: str(path)}), path)

    def test_partial_publication_can_resume_identical_records_only(self):
        run = capture(self.root, self.plan())
        records = load_records(self.root)
        items = [records[identity] for identity in run['evidence_ids']]
        (self.root / f"validation/executions/{run['id']}.yaml").unlink()
        (self.root / f"validation/evidence-manifests/{items[-1]['id']}.yaml").unlink()
        publish_run(self.root, run, items, resume=True)
        self.verify(run)
        changed = deepcopy(run)
        changed['procedure'].append('Changed historical procedure')
        with self.assertRaisesRegex(ValueError, 'cannot change'):
            publish_run(self.root, changed, items, resume=True)

    def test_complete_fixture_decision_checks_without_registering(self):
        run = capture(self.root, self.plan())
        verify_decision(self.root, self.decision(run), PinnedDefinitions(self.root), load_records(self.root), require_current=True)
        self.assertFalse(list((self.root / 'validation/decisions').glob('*.yaml')))

    def test_partial_experiment_cannot_close_validation(self):
        run = capture(self.root, self.plan(selected=['TST-001']))
        with self.assertRaisesRegex(ValueError, 'every mandatory'):
            verify_decision(self.root, self.decision(run), PinnedDefinitions(self.root), load_records(self.root), require_current=True)

    def test_required_evidence_and_invalidating_findings_block_acceptance(self):
        run = capture(self.root, self.plan())
        candidate = self.decision(run)
        candidate['evidence_assessment'] = []
        with self.assertRaisesRegex(ValueError, 'explicit assessment'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)
        candidate = self.decision(run)
        candidate['unresolved_findings'] = [{'description': 'Synthetic invalidating finding', 'severity': 'critical', 'invalidates_conclusion': True}]
        with self.assertRaisesRegex(ValueError, 'invalidating'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)

    def test_conditional_decision_needs_limits_and_follow_up(self):
        run = capture(self.root, self.plan())
        candidate = self.decision(run)
        candidate['disposition'] = 'conditional'
        candidate['follow_up'] = [{'action': 'Synthetic follow-up', 'owner': 'Synthetic reviewer', 'review_condition': 'Synthetic scope change'}]
        candidate['applicability']['limitations'] = []
        candidate['applicability']['exclusions'] = []
        with self.assertRaisesRegex(ValueError, 'explicit limitation'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)
        candidate['applicability']['limitations'] = ['Synthetic fixture scope only']
        verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)

    def test_blank_reviewer_and_review_before_observations_are_rejected(self):
        run = capture(self.root, self.plan())
        candidate = self.decision(run)
        candidate['reviewer'] = '   '
        with self.assertRaisesRegex(ValueError, 'named human reviewer'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)
        candidate = self.decision(run)
        candidate['reviewed_at'] = '2000-01-01T00:00:00Z'
        with self.assertRaisesRegex(ValueError, 'predates'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), load_records(self.root), require_current=True)

    def test_toolchain_failure_records_error_without_claiming_tests_ran(self):
        plan = self.plan()
        plan['toolchain_command'] = [sys.executable, '-c', 'raise SystemExit(2)']
        run = capture(self.root, plan)
        self.assertEqual(run['outcome'], 'error')
        self.assertTrue(all(case['outcome'] == 'skipped' for case in run['tests']))
        self.verify(run)

    def test_bundle_artifacts_can_be_verified_from_an_explicit_mirror(self):
        run = capture(self.root, self.plan(retention='bundle'))
        records = load_records(self.root)
        mirror = self.root / 'evidence/retained-mirror'
        bundle = self.root / 'evidence' / run['id']
        shutil.copytree(bundle, mirror / 'evidence' / run['id'])
        item = records[run['tests'][0]['evidence_ids'][0]]
        original = self.root / item['artifact']['uri'].removeprefix('repo://')
        original.unlink()
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            verify_artifact(self.root, item)
        verify_run(self.root, run, PinnedDefinitions(self.root), records, artifact_root=mirror)

    def test_historical_decision_survives_new_definition_but_is_not_current(self):
        run = capture(self.root, self.plan())
        records = load_records(self.root)
        records['VAL-012']['revision'] = 4
        records['VAL-012']['objective'] = 'Changed synthetic question'
        verify_decision(self.root, self.decision(run), PinnedDefinitions(self.root), records)
        with self.assertRaisesRegex(ValueError, 'current validation definition'):
            verify_decision(self.root, self.decision(run), PinnedDefinitions(self.root), records, require_current=True)

    def test_review_package_remains_unapproved_and_outside_registry(self):
        capture(self.root, self.plan())
        output = self.root / 'evidence/review-draft.yaml'
        draft = prepare(self.root, 'VAL-012', 'VDEC-001', output)
        self.assertTrue(draft['draft'])
        self.assertIsNone(draft['decision']['reviewer'])
        self.assertIsNone(draft['decision']['disposition'])
        self.assertFalse(list((self.root / 'validation/decisions').glob('*.yaml')))

    def test_manual_review_observations_can_be_imported_without_fabricated_commands(self):
        timestamp = utc_now()
        artifact = self.root / 'evidence/fixture-review.txt'
        artifact.parent.mkdir()
        artifact.write_text('Synthetic reviewer checklist and observed fixture results; no product approval.\n')
        evidence = {'schema_version': 1, 'id': 'EVD-manual-fixture', 'kind': 'documentary',
                    'provenance': {'execution_id': 'RUN-manual-fixture', 'description': 'Synthetic manual fixture', 'source_version': self.commit, 'retrieved_at': timestamp},
                    'artifact': {'uri': 'repo://evidence/fixture-review.txt', 'sha256': file_digest(artifact), 'size_bytes': artifact.stat().st_size,
                                 'media_type': 'text/plain', 'retention_location': 'Synthetic fixture bundle', 'access': 'Local fixture'},
                    'supports': [{'validation': reference('VAL-012', 3, self.commit), 'scope': 'Synthetic manual observations'}], 'supersedes': None}
        run = {'schema_version': 1, 'id': 'RUN-manual-fixture', 'started_at': timestamp, 'ended_at': timestamp,
               'experiment': reference('EXP-001', 2, self.commit), 'working_directory': '.',
               'tests': [{'test': reference(identity, 1, self.commit), 'outcome': 'passed', 'details': 'Synthetic checklist comparison',
                          'command': [], 'exit_code': None, 'duration_seconds': 0, 'evidence_ids': [evidence['id']]} for identity in self.commands],
               'outcome': 'passed', 'procedure': ['Synthetic independent source/checklist comparison'],
               'source': {'implementation_commit': self.commit, 'registry_commit': self.commit, 'dirty': False, 'changed_during_run': False, 'patch_evidence_id': None, 'dependency_lockfiles': []},
               'environment': {'toolchain': 'Manual source checklist', 'os': 'Documentary fixture', 'cpu': 'No runtime performance claim', 'configuration': 'Synthetic review', 'deviations': []},
               'inputs': [], 'random_seeds': [], 'instrumentation': 'Manual fixture review', 'evidence_ids': [evidence['id']], 'failures': [], 'skips': [], 'tool_errors': [], 'supersedes': None}
        publish_run(self.root, run, [evidence])
        self.verify(run)
        self.assertFalse(list((self.root / 'validation/decisions').glob('*.yaml')))

    def test_later_contradictory_observation_blocks_cherry_picked_acceptance(self):
        first = capture(self.root, self.plan('alternating'))
        candidate = self.decision(first)
        second = capture(self.root, self.plan('alternating', run_id='RUN-fixture-002'))
        self.assertEqual((first['outcome'], second['outcome']), ('passed', 'failed'))
        records = load_records(self.root)
        # A historically reviewed conclusion remains historical; selecting it
        # for current acceptance must account for later contrary observations.
        verify_decision(self.root, candidate, PinnedDefinitions(self.root), records)
        with self.assertRaisesRegex(ValueError, 'later contradictory'):
            verify_decision(self.root, candidate, PinnedDefinitions(self.root), records, require_current=True)

    def test_review_registration_requires_explicit_attestation_and_keeps_selection_separate(self):
        run = capture(self.root, self.plan())
        path = self.root / 'evidence/fixture-decision.yaml'
        path.write_text(yaml_text(self.decision(run)))
        arguments = ['review_validation.py', '--root', str(self.root), 'check', '--candidate', str(path)]
        with patch.object(sys, 'argv', arguments), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
            self.assertEqual(review_main(), 0)
        self.assertFalse(list((self.root / 'validation/decisions').glob('*.yaml')))
        with patch.object(sys, 'argv', arguments + ['--register-reviewed']), redirect_stdout(StringIO()), redirect_stderr(StringIO()):
            self.assertEqual(review_main(), 0)
        records = load_records(self.root)
        self.assertIn('VDEC-001', records)
        self.assertIsNone(records['VAL-012']['current_decision_id'])
        self.assertEqual(records['VAL-012']['lifecycle']['state'], 'reviewing')

    def test_missing_declared_output_is_preserved_as_tool_error(self):
        plan = self.plan(selected=['TST-001'])
        plan['tests'][0]['artifacts'] = [{'path': 'absent-output.json', 'kind': 'test_report'}]
        run = capture(self.root, plan)
        self.assertEqual(run['outcome'], 'error')
        self.assertIn('artifact unavailable', run['tests'][0]['details'])
        self.verify(run)

    def test_shared_ci_rejects_performance_acceptance(self):
        path = self.root / 'validation/register/tests/TST-001.yaml'
        test = yaml.safe_load(path.read_text())
        test['classification'] = 'performance'
        self.write('validation/register/tests/TST-001.yaml', test)
        self.commit = self.commit_all()
        with self.assertRaisesRegex(ValueError, 'Shared CI cannot execute performance'):
            capture(self.root, self.plan(selected=['TST-001']), shared_ci=True)

    def test_unbound_pinned_method_is_rejected_before_execution(self):
        path = self.root / 'validation/register/tests/TST-001.yaml'
        test = yaml.safe_load(path.read_text())
        test['execution_binding'] = {'state': 'unbound', 'implementation_path': None, 'entrypoint': None}
        self.write('validation/register/tests/TST-001.yaml', test)
        self.commit = self.commit_all()
        with self.assertRaisesRegex(ValueError, 'mandatory methods must be bound'):
            capture(self.root, self.plan())

    def test_false_pass_missing_cases_and_lockfile_hash_are_rejected(self):
        run = capture(self.root, self.plan())
        records = load_records(self.root)
        changed = deepcopy(run)
        changed['tests'][0]['exit_code'] = 1
        with self.assertRaisesRegex(ValueError, 'exit code zero'):
            verify_run(self.root, changed, PinnedDefinitions(self.root), records)
        changed = deepcopy(run)
        changed['tests'].pop()
        with self.assertRaisesRegex(ValueError, 'every declared test'):
            verify_run(self.root, changed, PinnedDefinitions(self.root), records)
        changed = deepcopy(run)
        changed['source']['dependency_lockfiles'] = [{'path': 'fixture.txt', 'sha256': '0' * 64}]
        with self.assertRaisesRegex(ValueError, 'lockfile hash differs'):
            verify_run(self.root, changed, PinnedDefinitions(self.root), records)

    def test_immutable_file_write_is_exclusive(self):
        path = self.root / 'evidence/immutable.txt'
        immutable_write(path, 'first')
        with self.assertRaises(FileExistsError):
            immutable_write(path, 'second')
        self.assertEqual(path.read_text(), 'first')

    def test_history_blocks_definition_edits_without_revision(self):
        self.val['objective'] = 'Changed without revision'
        self.write('validation/register/validations/VAL-012.yaml', self.val)
        self.assertTrue(any('require a new revision' in error for error in check_history(self.root, self.commit)))

    def test_history_blocks_ledger_and_snapshot_rewrites(self):
        run = capture(self.root, self.plan())
        baseline = self.commit_all()
        path = self.root / f"validation/executions/{run['id']}.yaml"
        path.chmod(0o644)
        path.write_text(path.read_text().replace('outcome: passed', 'outcome: failed', 1))
        self.assertTrue(any('immutable content changed' in error for error in check_history(self.root, baseline)))

    def test_history_allows_appended_lifecycle_and_definition_revision(self):
        previous = deepcopy(self.val)
        self.write('validation/revisions/validations/VAL-012/r3.yaml', previous)
        self.val['revision'] = 4
        self.val['objective'] = 'Changed synthetic fixture question'
        self.val['revision_history'].append({'revision': 4, 'date': '2026-10-15', 'actor': 'fixture', 'reason': 'Synthetic revision'})
        self.write('validation/register/validations/VAL-012.yaml', self.val)
        self.assertEqual(check_history(self.root, self.commit), [])

    def test_event_base_is_explicit_and_missing_history_fails(self):
        event = self.root / 'evidence/event.json'
        event.parent.mkdir(exist_ok=True)
        event.write_text(json.dumps({'pull_request': {'base': {'sha': self.commit}}}))
        with patch.dict(os.environ, {'GITHUB_EVENT_NAME': 'pull_request', 'GITHUB_EVENT_PATH': str(event)}):
            self.assertEqual(automatic_base(self.root), self.commit)
        with self.assertRaises(ValueError):
            check_history(self.root, 'f' * 40)

    def test_aggregate_never_treats_missing_or_error_cases_as_pass(self):
        self.assertEqual(aggregate([]), 'incomplete')
        self.assertEqual(aggregate([{'outcome': 'skipped'}]), 'incomplete')
        self.assertEqual(aggregate([{'outcome': 'passed'}], infrastructure_error=True), 'error')
        self.assertEqual(aggregate([{'outcome': 'passed'}], aborted=True), 'aborted')


if __name__ == '__main__':
    unittest.main()
