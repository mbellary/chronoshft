"""Regression checks for CI parity, fail-fast behavior and rendered links."""

from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import yaml

from scripts.check_documentation import run_checks
from scripts.check_documentation_links import check_links
from scripts.validate_register import ROOT


class DocumentationGateTests(unittest.TestCase):
    def run_quietly(self, root):
        with redirect_stdout(StringIO()), redirect_stderr(StringIO()):
            return run_checks(root)

    def test_failed_check_stops_before_build_or_publication(self):
        with patch('scripts.check_documentation.subprocess.run') as run:
            run.side_effect = [subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 7)]
            self.assertEqual(self.run_quietly(ROOT), 7)
            self.assertEqual(run.call_count, 2)
            self.assertFalse(any('mkdocs' in call.args[0] for call in run.call_args_list))

    def test_full_gate_checks_generated_files_without_regenerating(self):
        with patch('scripts.check_documentation.subprocess.run', return_value=subprocess.CompletedProcess([], 0)) as run:
            self.assertEqual(self.run_quietly(ROOT), 0)
            commands = [call.args[0] for call in run.call_args_list]
            generator = next(command for command in commands if 'scripts/generate_validation_docs.py' in command)
            self.assertIn('--check', generator)
            self.assertIn('--require-tests', commands[0])
            build = next(index for index, command in enumerate(commands) if 'mkdocs' in command)
            links = next(index for index, command in enumerate(commands) if 'scripts/check_documentation_links.py' in command)
            self.assertLess(build, links)
            self.assertTrue(all(call.kwargs['cwd'] == ROOT for call in run.call_args_list))

    def test_unstartable_check_fails_closed(self):
        with patch('scripts.check_documentation.subprocess.run', side_effect=OSError('fixture failure')) as run:
            self.assertEqual(self.run_quietly(ROOT), 1)
            self.assertEqual(run.call_count, 1)


class RenderedLinkTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def write(self, relative, content):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def check(self):
        return check_links(self.root, 'https://example.test/project/')

    def test_directory_fragment_query_and_encoded_file_links(self):
        self.write('index.html', '<a href="nested/?q=fixture#part%2Done">child</a><a href="data%20file.pdf#page=1">pdf</a>')
        self.write('nested/index.html', '<h1 id="part-one">Part</h1><a href="../#home">home</a>')
        self.write('data file.pdf', 'fixture, not a real PDF')
        self.write('index.html', (self.root / 'index.html').read_text() + '<h1 id="home">Home</h1>')
        errors, count, pages = self.check()
        self.assertEqual(errors, [])
        self.assertEqual((count, pages), (3, 2))

    def test_missing_local_file_is_a_failure(self):
        self.write('index.html', '<a href="missing/">missing</a>')
        self.assertIn('missing local file', '\n'.join(self.check()[0]))

    def test_missing_anchor_is_a_failure(self):
        self.write('index.html', '<a href="#missing">missing</a>')
        self.assertIn('missing local anchor', '\n'.join(self.check()[0]))

    def test_site_root_links_and_external_links(self):
        self.write('index.html', '<h1 id="home">Home</h1><a href="/project/#home">home</a><a href="https://elsewhere.invalid/unfetched">external</a><a href="mailto:example@example.test">mail</a>')
        errors, count, _ = self.check()
        self.assertEqual(errors, [])
        self.assertEqual(count, 1)

    def test_relative_link_cannot_escape_project_prefix(self):
        self.write('index.html', '<a href="../outside.html">outside</a>')
        self.assertIn('relative link escapes the site prefix', '\n'.join(self.check()[0]))

    def test_encoded_link_cannot_escape_rendered_directory(self):
        self.write('index.html', '<a href="%2e%2e/outside.html">outside</a>')
        self.assertIn('link escapes the rendered site', '\n'.join(self.check()[0]))

    def test_empty_build_does_not_pass(self):
        self.assertIn('No rendered HTML pages', '\n'.join(self.check()[0]))


class WorkflowContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # BaseLoader preserves GitHub's "on" key as text rather than YAML 1.1 true.
        cls.workflow = yaml.load((ROOT / '.github/workflows/documentation.yml').read_text(), Loader=yaml.BaseLoader)

    def test_all_gate_inputs_trigger_push_and_pull_request_checks(self):
        required = {'experiments/**', 'docs/**', 'research/**', 'validation/**', 'scripts/**', 'tests/**', 'mkdocs.yml', 'pyproject.toml', 'uv.lock', '.python-version', 'README.md', '.github/workflows/documentation.yml'}
        for event in ('push', 'pull_request'):
            self.assertTrue(required <= set(self.workflow['on'][event]['paths']))

    def test_ci_installs_locked_validation_dependencies_and_uses_local_gate(self):
        steps = self.workflow['jobs']['build']['steps']
        commands = [step['run'] for step in steps if 'run' in step]
        install = next(command for command in commands if 'uv sync' in command)
        self.assertIn('--locked', install)
        self.assertIn('--extra docs', install)
        self.assertIn('--extra validation', install)
        gate = next(step for step in steps if 'scripts/check_documentation.py' in step.get('run', ''))
        self.assertNotIn('if', gate)
        self.assertNotIn('continue-on-error', gate)
        checkout = next(step for step in steps if step.get('uses', '').startswith('actions/checkout@'))
        self.assertEqual(checkout['with']['fetch-depth'], '0')

    def test_execution_capture_is_explicit_and_retains_failure_artifacts(self):
        workflow = yaml.load((ROOT / '.github/workflows/validation-experiments.yml').read_text(), Loader=yaml.BaseLoader)
        self.assertEqual(set(workflow['on']), {'workflow_dispatch'})
        self.assertEqual(workflow['permissions'], {'contents': 'read'})
        steps = workflow['jobs']['capture']['steps']
        capture = next(step for step in steps if 'scripts/capture_execution.py' in step.get('run', ''))
        self.assertIn('--committed-plan', capture['run'])
        self.assertIn('--shared-ci', capture['run'])
        self.assertIn('"$VALIDATION_PLAN"', capture['run'])
        self.assertNotIn('${{ inputs.plan }}', capture['run'])
        upload = next(step for step in steps if step.get('uses', '').startswith('actions/upload-artifact@'))
        self.assertEqual(upload['if'], 'always()')
        self.assertIn('evidence/', upload['with']['path'])
        self.assertIn('validation/artifacts/', upload['with']['path'])

    def test_publication_depends_on_successful_full_gate_and_excludes_pull_requests(self):
        build = self.workflow['jobs']['build']
        steps = build['steps']
        gate = next(index for index, step in enumerate(steps) if 'scripts/check_documentation.py' in step.get('run', ''))
        upload = next(index for index, step in enumerate(steps) if 'actions/upload-pages-artifact@' in step.get('uses', ''))
        self.assertLess(gate, upload)
        self.assertIn("github.event_name != 'pull_request'", steps[upload]['if'])
        self.assertNotIn('continue-on-error', build)
        deploy = self.workflow['jobs']['deploy']
        self.assertEqual(deploy['needs'], 'build')
        self.assertIn("github.event_name != 'pull_request'", deploy['if'])
        self.assertEqual(self.workflow['permissions'], {'contents': 'read'})


if __name__ == '__main__':
    unittest.main()
