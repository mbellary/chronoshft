"""Behavioral checks for Lab projections and bounded navigation generation."""

from copy import deepcopy
import unittest

from scripts.generate_validation_docs import (
    LAB, NAV_START, NAV_END, cell, generated_documents, render_environments,
    render_evidence, render_executions, render_experiment, render_legacy_tests,
    render_navigation, render_overview, render_results, render_test_details,
    replace_navigation,
)
from scripts.validate_register import ROOT, validate


class ValidationDocsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        errors, cls.records = validate(ROOT, require_tests=True)
        if errors:
            raise AssertionError(errors)

    def test_every_experiment_has_a_generated_page_and_nav_entry(self):
        documents = generated_documents(self.records)
        nav = render_navigation(self.records)
        experiments = [record for record in self.records.values() if record['id'].startswith('EXP-')]
        paths = {LAB / 'experiments' / f"{exp['id']}.md" for exp in experiments}
        self.assertEqual({path for path in documents if path.parent == LAB / 'experiments'}, paths)
        self.assertEqual(len(paths), 29)
        for path in paths:
            self.assertEqual(nav.count(str(path.relative_to('docs'))), 1)

    def test_nav_replacement_preserves_other_configuration(self):
        before = 'nav:\n  - Rust reference: existing.md\n'
        after = '\n  - Financial domain: financial.md\nextra: preserve-me\n'
        original = before + NAV_START + '\nold-content\n' + NAV_END + after
        updated = replace_navigation(original, render_navigation(self.records))
        self.assertTrue(updated.startswith(before))
        self.assertTrue(updated.endswith(after))
        self.assertNotIn('old-content', updated)
        self.assertEqual(updated.count('Technical Validation Lab:'), 1)

    def test_missing_duplicate_or_reversed_nav_markers_are_rejected(self):
        for text in ('nav: []', NAV_START + NAV_END + NAV_START, NAV_END + '\n' + NAV_START):
            with self.subTest(text=text), self.assertRaises(ValueError):
                replace_navigation(text, render_navigation(self.records))

    def test_lab_follows_rust_and_precedes_financial_domain(self):
        text = (ROOT / 'mkdocs.yml').read_text()
        self.assertLess(text.index('  - Rust reference:'), text.index(NAV_START))
        self.assertLess(text.index(NAV_END), text.index('  - Financial domain:'))

    def test_unexecuted_states_do_not_become_results(self):
        runs = render_executions(self.records)
        evidence = render_evidence(self.records)
        results = render_results(self.records)
        environment = render_environments(self.records)
        self.assertIn('No executions recorded.', runs)
        self.assertIn('No evidence manifests recorded.', evidence)
        self.assertIn('Every current disposition is pending', results)
        self.assertIn('Validated (role only)', results)
        self.assertIn('No run environments recorded.', environment)
        self.assertNotIn('100%', render_overview(self.records))

    def test_unbound_experiment_has_no_invented_runner(self):
        text = render_experiment(self.records['EXP-001'], self.records)
        self.assertIn('No build, test or benchmark commands are bound.', text)
        self.assertIn('Unassigned; no implementation is asserted.', text)
        self.assertIn('No experimental findings or conclusion are recorded.', text)
        self.assertNotIn('cargo test', text)
        for reference in self.records['EXP-001']['tests']:
            self.assertIn(f"#{reference['id'].lower()} }}", text)
            self.assertIn(self.records[reference['id']]['oracle']['description'], text)

    def test_shared_experiment_preserves_narrow_test_applicability(self):
        text = '\n'.join(render_test_details(self.records['TST-009']))
        self.assertIn('VAL-013 r3', text)
        self.assertNotIn('VAL-012 r3', text)

    def test_legacy_test_anchors_link_to_owning_experiment(self):
        text = render_legacy_tests(self.records)
        for record in self.records.values():
            if record['id'].startswith('TST-'):
                self.assertIn(f"#{record['id'].lower()} }}", text)
                self.assertIn(f"../10-validation-lab/experiments/{record['experiment']['id']}.md#{record['id'].lower()}", text)

    def test_table_values_escape_html_and_pipe_characters(self):
        self.assertEqual(cell('<script>a|b</script>\nnext'), '&lt;script&gt;a\\|b&lt;/script&gt;<br>next')

    def test_failed_run_keeps_tested_revision_and_does_not_accept_validation(self):
        records = deepcopy(self.records)
        records['RUN-synthetic-failed'] = {
            'id': 'RUN-synthetic-failed', 'started_at': '2026-10-15T10:00:00Z', 'ended_at': '2026-10-15T10:00:01Z',
            'experiment': {'id': 'EXP-001', 'revision': 1, 'registry_commit': '1' * 40},
            'tests': [{'test': {'id': 'TST-001', 'revision': 1, 'registry_commit': '1' * 40}, 'outcome': 'failed', 'details': 'Fixture mismatch', 'exit_code': 1, 'duration_seconds': 1, 'evidence_ids': [], 'command': ['fixture']}],
            'outcome': 'failed', 'procedure': ['Synthetic renderer fixture; no real execution.'],
            'source': {'registry_commit': '1' * 40, 'implementation_commit': '2' * 40, 'dirty': False, 'changed_during_run': False, 'dependency_lockfiles': []},
            'working_directory': '.', 'inputs': [], 'random_seeds': [], 'instrumentation': 'Synthetic fixture only',
            'evidence_ids': [], 'failures': ['Fixture mismatch'], 'skips': [], 'tool_errors': [], 'supersedes': None,
            'environment': {'toolchain': 'fixture compiler', 'os': 'fixture OS', 'cpu': 'fixture CPU', 'configuration': 'fixture config', 'deviations': []},
        }
        text = render_executions(records)
        self.assertIn('EXP-001](experiments/EXP-001.md) r1', text)
        self.assertIn('Fixture mismatch', text)
        self.assertIn('**Recorded outcome:** failed', text)
        self.assertNotIn('No executions recorded.', text)
        self.assertIn('Every current disposition is pending', render_results(records))
        self.assertIn('fixture compiler', render_environments(records))


if __name__ == '__main__':
    unittest.main()
