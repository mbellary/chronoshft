"""Regression checks for migration integrity and registry guardrails."""

from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import yaml

from scripts.validate_register import ROOT, validate


class RegisterChecks(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        shutil.copytree(ROOT / "validation", self.root / "validation")
        shutil.copytree(ROOT / "docs/10-validation-lab", self.root / "docs/10-validation-lab")
        shutil.copyfile(ROOT / "mkdocs.yml", self.root / "mkdocs.yml")
        for relative in (
            "docs/00-governance/technical-validation.md",
            "docs/00-governance/validation-coverage.md",
            "docs/00-governance/validation-tests.md",
            "docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md",
        ):
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        self.record_path = self.root / "validation/register/validations/VAL-012.yaml"
        self.record = yaml.safe_load(self.record_path.read_text())

    def write_record(self):
        self.record_path.write_text(yaml.safe_dump(self.record, sort_keys=False))

    def assert_rejected(self, message):
        errors, _ = validate(self.root, require_coverage=True, require_tests=True)
        self.assertTrue(any(message in error for error in errors), errors)

    def test_complete_migration(self):
        errors, records = validate(self.root, require_coverage=True, require_tests=True)
        self.assertEqual(errors, [])
        vals = [r for r in records.values() if r["id"].startswith("VAL-")]
        experiments = [r for r in records.values() if r["id"].startswith("EXP-")]
        self.assertEqual(len(vals), 28)
        self.assertEqual(len(experiments), 29)
        tests = [r for r in records.values() if r["id"].startswith("TST-")]
        self.assertEqual(len(tests), 90)
        self.assertEqual(len(vals) + len(experiments) + len(tests), 147)
        self.assertTrue(all(r["lifecycle"]["state"] == "planned" and r["revision"] == (4 if r["id"] in {"VAL-012", "VAL-013"} else 3) for r in vals))
        self.assertTrue(all(r["current_decision_id"] is None for r in vals))
        self.assertTrue(all(r["lifecycle"]["state"] == ("ready" if r["id"] == "EXP-001" else "proposed") and r["revision"] == (3 if r["id"] in {"EXP-001", "EXP-002", "EXP-003"} else 2) and r["tests"] for r in experiments))
        self.assertTrue(all(r["execution_binding"]["state"] == ("bound" if r["id"] in {"TST-001", "TST-002", "TST-003", "TST-004"} else "unbound") and r["mandatory"] for r in tests))
        self.assertNotIn("EXP-004", records)

    def test_missing_historical_record(self):
        self.record_path.unlink()
        self.assert_rejected("missing migrated record: VAL-012")

    def test_changed_research_finding(self):
        self.record["origin"]["finding"] = "Queue safety proven"
        self.write_record()
        self.assert_rejected("origin differs")

    def test_changed_closure_requirement(self):
        self.record["revision"] = 1
        self.record["revision_history"] = self.record["revision_history"][:1]
        self.record["required_evidence"] = ["Documentation build"]
        self.write_record()
        self.assert_rejected("initial closure evidence was not preserved")

    def test_coordinated_markdown_and_yaml_edit(self):
        path = self.root / "docs/00-governance/technical-validation.md"
        old = self.record["origin"]["finding"]
        path.write_text(path.read_text().replace(old, "Queue safety proven"))
        self.record["origin"]["finding"] = "Queue safety proven"
        self.write_record()
        self.assert_rejected("historical research table differs")

    def test_source_document_change(self):
        path = self.root / "docs/assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md"
        path.write_text(path.read_text() + "\nChanged source\n")
        self.assert_rejected("source document hash differs")

    def test_duplicate_yaml_key(self):
        self.record_path.write_text(self.record_path.read_text() + "revision: 2\n")
        self.assert_rejected("duplicate YAML key")

    def test_invalid_state(self):
        self.record["lifecycle"]["state"] = "validated"
        self.write_record()
        self.assert_rejected("not one of")

    def test_unknown_field(self):
        self.record["approved"] = True
        self.write_record()
        self.assert_rejected("Additional properties")

    def test_invalid_date(self):
        self.record["revision_history"][0]["date"] = "2026-02-30"
        self.write_record()
        self.assert_rejected("not a 'date'")

    def test_dangling_experiment(self):
        self.record["coverage"] = {
            "status": "mapped", "experiments": [{"id": "EXP-999", "revision": 1, "mandatory": True}], "tests": [],
        }
        self.write_record()
        self.assert_rejected("unknown record reference: EXP-999")

    def test_untracked_state_change(self):
        self.record["coverage"]["status"] = "mapped"
        self.record["lifecycle"]["state"] = "ready"
        self.write_record()
        self.assert_rejected("lifecycle state disagrees")

    def test_revision_history_gap(self):
        self.record["revision"] += 1
        self.write_record()
        self.assert_rejected("revision history must account")

    def test_filename_id_mismatch(self):
        self.record_path.rename(self.record_path.with_name("VAL-999.yaml"))
        self.assert_rejected("filename must match")

    def test_coverage_gate_rejects_unmapped_validation(self):
        self.record["coverage"] = {"status": "unmapped", "experiments": [], "tests": []}
        self.record["lifecycle"]["state"] = "identified"
        self.record["lifecycle"]["history"] = self.record["lifecycle"]["history"][:1]
        self.write_record()
        self.assert_rejected("complete coverage requires")

    def test_coverage_requires_mandatory_investigation(self):
        for exp in self.record["coverage"]["experiments"]:
            exp["mandatory"] = False
        self.write_record()
        self.assert_rejected("complete coverage requires")

    def test_missing_reciprocal_link(self):
        self.record["coverage"]["experiments"] = self.record["coverage"]["experiments"][:1]
        self.write_record()
        self.assert_rejected("experiment/validation coverage is not reciprocal")

    def test_stale_experiment_revision(self):
        self.record["coverage"]["experiments"][0]["revision"] = 999
        self.write_record()
        self.assert_rejected("stale planning reference")

    def test_duplicate_coverage_id_with_different_metadata(self):
        reference = dict(self.record["coverage"]["experiments"][0], mandatory=False)
        self.record["coverage"]["experiments"].append(reference)
        self.write_record()
        self.assert_rejected("duplicate relationship ID")

    def test_duplicate_hypothesis_identity(self):
        path = self.root / "validation/register/experiments/EXP-003.yaml"
        experiment = yaml.safe_load(path.read_text())
        experiment["hypotheses"][1]["id"] = experiment["hypotheses"][0]["id"]
        path.write_text(yaml.safe_dump(experiment, sort_keys=False))
        self.assert_rejected("duplicate hypothesis ID")

    def test_experiment_cannot_be_ready_without_tests(self):
        path = self.root / "validation/register/experiments/EXP-003.yaml"
        experiment = yaml.safe_load(path.read_text())
        experiment["lifecycle"]["state"] = "ready"
        experiment["tests"] = []
        experiment["lifecycle"]["history"].append({
            "from": "proposed", "to": "ready", "date": "2026-10-15", "actor": "test", "reason": "premature",
        })
        path.write_text(yaml.safe_dump(experiment, sort_keys=False))
        self.assert_rejected("should be non-empty")

    def test_shared_experiment_keeps_both_validations(self):
        errors, records = validate(self.root, require_coverage=True)
        self.assertEqual(errors, [])
        self.assertEqual({v["id"] for v in records["EXP-003"]["validates"]}, {"VAL-012", "VAL-013"})
        self.assertEqual({e["id"] for e in records["VAL-012"]["coverage"]["experiments"]}, {"EXP-001", "EXP-002", "EXP-003"})

    def test_missing_import_snapshot(self):
        (self.root / "validation/revisions/validations/VAL-012/r1.yaml").unlink()
        self.assert_rejected("missing prior revision snapshot")

    def test_changed_snapshot_provenance(self):
        path = self.root / "validation/revisions/validations/VAL-012/r1.yaml"
        prior = yaml.safe_load(path.read_text())
        prior["origin"]["source_status"] = "Accepted"
        path.write_text(yaml.safe_dump(prior, sort_keys=False))
        self.assert_rejected("prior revision snapshot changed")

    def test_generated_coverage_is_current(self):
        result = subprocess.run([
            sys.executable, str(ROOT / "scripts/generate_validation_coverage.py"), "--root", str(self.root), "--check",
        ], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_generated_coverage_rejects_stale_view(self):
        path = self.root / "docs/00-governance/validation-coverage.md"
        path.write_text(path.read_text() + "\nStale manual status\n")
        result = subprocess.run([
            sys.executable, str(ROOT / "scripts/generate_validation_coverage.py"), "--root", str(self.root), "--check",
        ], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing or stale", result.stderr)

    def change_test(self, identity, change):
        path = self.root / f"validation/register/tests/{identity}.yaml"
        record = yaml.safe_load(path.read_text())
        change(record)
        path.write_text(yaml.safe_dump(record, sort_keys=False))

    def test_unknown_test_hypothesis(self):
        self.change_test("TST-001", lambda test: test.update(hypothesis_ids=["HYP-999"]))
        self.assert_rejected("unknown hypothesis in owning experiment")

    def test_stale_test_validation_revision(self):
        self.change_test("TST-001", lambda test: test["validations"][0].update(revision=1))
        self.assert_rejected("stale planning reference: VAL-012")

    def test_wrong_test_owner(self):
        self.change_test("TST-001", lambda test: test["experiment"].update(id="EXP-002"))
        self.assert_rejected("test/experiment relationship is not reciprocal")

    def test_test_applicability_outside_experiment(self):
        self.change_test("TST-001", lambda test: test.update(validations=[{"id": "VAL-001", "revision": 3}]))
        self.assert_rejected("test applies to a validation outside its experiment")

    def test_missing_validation_test_link(self):
        self.record["coverage"]["tests"] = [ref for ref in self.record["coverage"]["tests"] if ref["id"] != "TST-001"]
        self.write_record()
        self.assert_rejected("test/validation applicability is not reciprocal")

    def test_mandatory_test_obligation_mismatch(self):
        self.record["coverage"]["tests"][0]["mandatory"] = False
        self.write_record()
        self.assert_rejected("validation/test mandatory obligation disagrees")

    def test_uncovered_hypothesis(self):
        experiment = yaml.safe_load((self.root / "validation/register/experiments/EXP-001.yaml").read_text())
        for ref in experiment["tests"]:
            self.change_test(ref["id"], lambda test: test.update(hypothesis_ids=["HYP-001"]))
        self.assert_rejected("every hypothesis requires a mandatory test")

    def test_mixed_experiment_requires_review_and_automation(self):
        experiment = yaml.safe_load((self.root / "validation/register/experiments/EXP-002.yaml").read_text())
        for ref in experiment["tests"]:
            self.change_test(ref["id"], lambda test: test.update(execution_type="automated"))
        self.assert_rejected("mandatory tests do not cover the declared investigation methods")

    def test_unbound_tests_block_experiment_readiness(self):
        path = self.root / "validation/register/experiments/EXP-002.yaml"
        experiment = yaml.safe_load(path.read_text())
        experiment["lifecycle"]["state"] = "ready"
        experiment["lifecycle"]["history"].append({
            "from": "proposed", "to": "ready", "date": "2026-10-15", "actor": "test", "reason": "premature",
        })
        path.write_text(yaml.safe_dump(experiment, sort_keys=False))
        self.assert_rejected("experiment cannot advance with unbound mandatory tests")

    def test_unbound_test_cannot_claim_an_entrypoint(self):
        self.change_test("TST-001", lambda test: test["execution_binding"].update(state="unbound", implementation_path=None, entrypoint="claimed_runner"))
        self.assert_rejected("not of type 'null'")

    def test_bound_automation_requires_source_and_command(self):
        self.change_test("TST-001", lambda test: test["execution_binding"].update(state="bound", implementation_path=None, entrypoint="test-command"))
        self.assert_rejected("not of type 'string'")

    def test_test_requires_an_oracle(self):
        self.change_test("TST-001", lambda test: test["oracle"].update(description=""))
        self.assert_rejected("should be non-empty")

    def test_test_requires_evidence(self):
        self.change_test("TST-001", lambda test: test.update(required_evidence=[]))
        self.assert_rejected("should be non-empty")

    def test_shared_experiment_test_scope(self):
        errors, records = validate(self.root, require_tests=True)
        self.assertEqual(errors, [])
        ownership = next(test for test in records.values() if test.get("title") == "Producer and consumer API misuse rejection")
        self.assertEqual({v["id"] for v in ownership["validations"]}, {"VAL-013"})
        self.assertNotIn(ownership["id"], {t["id"] for t in records["VAL-012"]["coverage"]["tests"]})

    def test_missing_experiment_snapshot(self):
        (self.root / "validation/revisions/experiments/EXP-001/r1.yaml").unlink()
        self.assert_rejected("missing prior revision snapshot: EXP-001 r1")

    def test_generated_test_catalog_rejects_stale_view(self):
        path = self.root / "docs/00-governance/validation-tests.md"
        path.write_text(path.read_text() + "\nInvented pass\n")
        result = subprocess.run([
            sys.executable, str(ROOT / "scripts/generate_validation_coverage.py"), "--root", str(self.root), "--check",
        ], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("validation-tests.md is missing or stale", result.stderr)

    def test_missing_lab_page_is_detected_without_writing(self):
        path = self.root / "docs/10-validation-lab/experiments/EXP-001.md"
        path.unlink()
        nav_before = (self.root / 'mkdocs.yml').read_bytes()
        result = subprocess.run([
            sys.executable, str(ROOT / 'scripts/generate_validation_docs.py'), '--root', str(self.root), '--check',
        ], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('EXP-001.md is missing or stale', result.stderr)
        self.assertFalse(path.exists())
        self.assertEqual(nav_before, (self.root / 'mkdocs.yml').read_bytes())

    def test_unregistered_experiment_page_is_rejected(self):
        path = self.root / 'docs/10-validation-lab/experiments/EXP-999.md'
        path.write_text('# Unregistered experiment\n')
        result = subprocess.run([
            sys.executable, str(ROOT / 'scripts/generate_validation_docs.py'), '--root', str(self.root), '--check',
        ], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Unregistered experiment pages', result.stderr)


if __name__ == "__main__":
    unittest.main()
