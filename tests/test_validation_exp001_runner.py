"""Exercise real Rust tool/assertion failures in disposable copies, never the ledger."""
from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

from scripts.validate_register import ROOT


class Exp001RunnerChecks(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        shutil.copytree(ROOT / 'experiments', self.root / 'experiments',
                        ignore=shutil.ignore_patterns('target'))
        (self.root / 'scripts').mkdir()
        shutil.copyfile(ROOT / 'scripts/run_exp001_test.py', self.root / 'scripts/run_exp001_test.py')
        self.report = self.root / 'target/validation-reports/TST-001.txt'
        self.report.parent.mkdir(parents=True)
        self.report.write_text('STALE OBSERVATION: must never survive a new invocation\n')
        self.source = self.root / 'experiments/exp-001-spsc-queue/src/lib.rs'
        self.tests = self.root / 'experiments/exp-001-spsc-queue/tests/queue_contracts.rs'

    def run_case(self, case="fifo_exactly_once"):
        environment = os.environ.copy()
        environment.pop('CHRONOSHFT_REPORT_DIR', None)
        environment['CARGO_TARGET_DIR'] = str(self.root / 'target/build')
        return subprocess.run([sys.executable, 'scripts/run_exp001_test.py', case],
                              cwd=self.root, env=environment, capture_output=True, text=True, timeout=60)

    def test_all_declared_cases_pass_on_the_unmodified_prototype(self):
        for case, identity in [('fifo_exactly_once', 'TST-001'),
                               ('capacity_before_write', 'TST-002'),
                               ('wraparound_partial_drains', 'TST-003'),
                               ('capacity_configuration', 'TST-004')]:
            with self.subTest(case=case):
                result = self.run_case(case)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('1 passed; 0 failed', result.stdout)
                report = self.root / f'target/validation-reports/{identity}.txt'
                self.assertIn('PASS:', report.read_text())
                self.assertNotIn('STALE', report.read_text())

    def test_compiler_failure_is_tool_error_and_removes_stale_report(self):
        self.source.write_text(self.source.read_text() + '\nintentional invalid Rust fixture\n')
        result = self.run_case()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn('Tool error', result.stderr)
        self.assertFalse(self.report.exists())

    def test_missing_exact_case_cannot_report_an_empty_pass(self):
        self.tests.write_text(self.tests.read_text().replace('fn fifo_exactly_once()', 'fn renamed_fixture_case()'))
        result = self.run_case()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn('requested test is missing', result.stderr)
        self.assertFalse(self.report.exists())

    def test_assertion_failure_retains_fresh_observation_prefix(self):
        self.tests.write_text(self.tests.read_text().replace('assert_eq!(producer.capacity(), capacity);',
                                                           'assert_eq!(producer.capacity(), capacity + 1);', 1))
        result = self.run_case()
        self.assertEqual(result.returncode, 101, result.stdout + result.stderr)
        self.assertIn('FAILED', result.stdout)
        self.assertTrue(self.report.read_text().startswith('TST-001:'))
        self.assertNotIn('STALE', self.report.read_text())
