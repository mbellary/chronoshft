#!/usr/bin/env python3
"""Check validation schemas, registry references and the preserved research import."""

from __future__ import annotations

import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import sys

from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.exceptions import SchemaError, ValidationError
import yaml


ROOT = Path(__file__).resolve().parents[1]
CATALOGS = {
    "validation": "register/validations",
    "experiment": "register/experiments",
    "test": "register/tests",
    "execution": "executions",
    "evidence": "evidence-manifests",
    "decision": "decisions",
}
VAL_TRANSITIONS = {
    "identified": {"planned"}, "planned": {"ready"},
    "ready": {"running", "planned"}, "running": {"reviewing", "planned"},
    "reviewing": {"closed", "running", "planned"}, "closed": {"reopened"},
    "reopened": {"planned"},
}
EXP_TRANSITIONS = {
    "proposed": {"ready"}, "ready": {"running", "proposed"},
    "running": {"reviewing"}, "reviewing": {"completed", "running", "proposed"},
    "completed": set(),
}


class UniqueKeyLoader(yaml.SafeLoader):
    """Reject duplicate keys rather than silently overwriting a field."""


def unique_mapping(loader, node, deep=False):
    result = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in result:
            raise ValueError(f"duplicate YAML key: {key}")
        result[key] = loader.construct_object(value_node, deep=deep)
    return result


UniqueKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping
)


def read_yaml(path):
    return yaml.load(path.read_text(encoding="utf-8"), Loader=UniqueKeyLoader)


def digest(data):
    return hashlib.sha256(data).hexdigest()


FORMATS = FormatChecker()


@FORMATS.checks("date-time", raises=(ValueError, TypeError))
def valid_datetime(value):
    if not isinstance(value, str):
        return True  # Type validation reports non-strings.
    return bool(re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z", value)) and (
        datetime.fromisoformat(value.replace("Z", "+00:00")).utcoffset().total_seconds() == 0
    )


def local_path(root, relative):
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f"path escapes repository: {relative}")
    return path


def check_baseline(root, records):
    base = read_yaml(root / "validation/baseline.yaml")
    schema = json.loads((root / "validation/schemas/baseline.schema.json").read_text())
    Draft202012Validator.check_schema(schema)
    Draft202012Validator(schema, format_checker=FORMATS).validate(base)
    expected_ids = [f"VAL-{n:03}" for n in range(1, 29)]
    if base["record_count"] != 28 or base["validation_ids"] != expected_ids:
        raise ValueError("baseline must preserve VAL-001–VAL-028")
    source = base["source_document"]
    if digest(local_path(root, source["path"]).read_bytes()) != source["sha256"]:
        raise ValueError("preserved source document hash differs from baseline")
    rows = [line for line in local_path(root, base["baseline_path"]).read_text().splitlines()
            if line.startswith("| VAL-")]
    baseline_rows = [line for line in rows if line.split(" / ", 1)[0][2:] in expected_ids]
    if len(baseline_rows) != 28 or digest(("\n".join(baseline_rows) + "\n").encode()) != base["rows_sha256"]:
        raise ValueError("historical research table differs from pinned baseline")
    for row in baseline_rows:
        related, status, finding, closure = [cell.strip() for cell in row.strip("|").split("|")]
        identity, sources = related.split(" / ", 1)
        if identity not in records:
            raise ValueError(f"missing migrated record: {identity}")
        record = records[identity]
        expected = {
            "assessment_id": base["assessment_id"],
            "assessment_version": base["assessment_version"],
            "assessment_date": base["assessment_date"],
            "baseline_path": base["baseline_path"], "source_refs": sources,
            "source_status": status, "finding": finding,
            "required_closure_evidence": closure,
        }
        if record["origin"] != expected:
            raise ValueError(f"{identity}: origin differs from historical research assessment")
        if record["revision"] == 1 and record["required_evidence"] != [closure]:
            raise ValueError(f"{identity}: initial closure evidence was not preserved")


def walk_references(value):
    if isinstance(value, dict):
        if "id" in value and "revision" in value:
            yield value
        for key, child in value.items():
            if key not in {"origin", "revision_history"}:
                yield from walk_references(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk_references(child)


def check_history(record, transitions, initial):
    history = record["lifecycle"]["history"]
    previous = None
    last_date = ""
    for entry in history:
        target = entry["to"]
        if entry["from"] != previous or entry["date"] < last_date:
            raise ValueError("lifecycle history is discontinuous or dates are out of order")
        if previous is None:
            if target != initial:
                raise ValueError(f"lifecycle must start at {initial}")
        elif target not in transitions[previous]:
            raise ValueError(f"illegal lifecycle transition: {previous} -> {target}")
        previous, last_date = target, entry["date"]
    if previous != record["lifecycle"]["state"]:
        raise ValueError("lifecycle state disagrees with its history")


def check_relationships(record, kind, records):
    identity = record["id"]
    relationships = []
    if kind == "validation":
        relationships = list(record["coverage"]["experiments"]), list(record["coverage"]["tests"])
    elif kind == "experiment":
        relationships = record["validates"], record["tests"]
    elif kind == "test":
        relationships = [record["validations"]]
    for references in relationships:
        ids = [reference["id"] for reference in references]
        if len(ids) != len(set(ids)):
            raise ValueError("duplicate relationship ID")
    # Historical references pin a registry commit; existence is checked here,
    # but resolving their old content belongs to execution/evidence tooling.
    for reference in walk_references(record):
        target = records.get(reference["id"])
        if target is None:
            raise ValueError(f"unknown record reference: {reference['id']}")
        if "registry_commit" not in reference and reference["revision"] != target["revision"]:
            raise ValueError(f"stale planning reference: {reference['id']}")
    if "revision_history" in record:
        revisions = record["revision_history"]
        if [entry["revision"] for entry in revisions] != list(range(1, record["revision"] + 1)):
            raise ValueError("revision history must account for every revision")
        if [entry["date"] for entry in revisions] != sorted(entry["date"] for entry in revisions):
            raise ValueError("revision history dates are out of order")
    if kind == "validation":
        check_history(record, VAL_TRANSITIONS, "identified")
        coverage = record["coverage"]
        if coverage["status"] == "mapped" and not coverage["experiments"]:
            raise ValueError("mapped coverage needs at least one experiment")
        for exp in coverage["experiments"]:
            if identity not in {v["id"] for v in records[exp["id"]]["validates"]}:
                raise ValueError("validation/experiment coverage is not reciprocal")
        covered_experiments = {exp["id"] for exp in coverage["experiments"]}
        for test in coverage["tests"]:
            target = records[test["id"]]
            if target["experiment"]["id"] not in covered_experiments:
                raise ValueError("coverage test belongs to an unlinked experiment")
            if identity not in {v["id"] for v in target["validations"]}:
                raise ValueError("validation/test applicability is not reciprocal")
            if test["mandatory"] != target["mandatory"]:
                raise ValueError("validation/test mandatory obligation disagrees")
        decision_id = record["current_decision_id"]
        for previous_id in record["prior_decision_ids"]:
            if previous_id not in records or records[previous_id].get("validation", {}).get("id") != identity:
                raise ValueError("prior decision must exist and refer to this validation")
        if decision_id is not None:
            decision = records.get(decision_id)
            if decision is None or decision["validation"]["id"] != identity or decision["validation"]["revision"] != record["revision"]:
                raise ValueError("current decision must refer to this validation revision")
        state = record["lifecycle"]["state"]
        if state == "closed" and records[decision_id]["disposition"] == "inconclusive":
            raise ValueError("inconclusive decisions cannot close validations")
        if state == "reopened" and decision_id is not None:
            raise ValueError("reopened scope must be pending; retain prior decisions separately")
    elif kind == "experiment":
        check_history(record, EXP_TRANSITIONS, "proposed")
        hypotheses = [hyp["id"] for hyp in record["hypotheses"]]
        if len(hypotheses) != len(set(hypotheses)):
            raise ValueError("duplicate hypothesis ID")
        for val in record["validates"]:
            if identity not in {exp["id"] for exp in records[val["id"]]["coverage"]["experiments"]}:
                raise ValueError("experiment/validation coverage is not reciprocal")
        for test in record["tests"]:
            if records[test["id"]]["experiment"]["id"] != identity:
                raise ValueError("experiment lists a test owned by another experiment")
        if record["lifecycle"]["state"] != "proposed":
            if any(records[test["id"]]["mandatory"] and records[test["id"]]["execution_binding"]["state"] != "bound" for test in record["tests"]):
                raise ValueError("experiment cannot advance with unbound mandatory tests")
    elif kind == "test":
        experiment = records[record["experiment"]["id"]]
        if identity not in {test["id"] for test in experiment["tests"]}:
            raise ValueError("test/experiment relationship is not reciprocal")
        if not set(record["hypothesis_ids"]) <= {hyp["id"] for hyp in experiment["hypotheses"]}:
            raise ValueError("unknown hypothesis in owning experiment")
        if not {val["id"] for val in record["validations"]} <= {val["id"] for val in experiment["validates"]}:
            raise ValueError("test applies to a validation outside its experiment")
        for val in record["validations"]:
            if identity not in {test["id"] for test in records[val["id"]]["coverage"]["tests"]}:
                raise ValueError("test/validation applicability is not reciprocal")
    elif kind == "execution":
        if datetime.fromisoformat(record["ended_at"].replace("Z", "+00:00")) < datetime.fromisoformat(record["started_at"].replace("Z", "+00:00")):
            raise ValueError("execution ends before it starts")
        outcomes = [test["outcome"] for test in record["tests"]]
        test_ids = [test["test"]["id"] for test in record["tests"]]
        if len(test_ids) != len(set(test_ids)):
            raise ValueError("execution contains duplicate test results")
        if record["outcome"] != "aborted":
            aggregate = ("error" if "error" in outcomes or record['tool_errors'] or record['source']['changed_during_run'] else "failed" if "failed" in outcomes
                         else "incomplete" if not outcomes or "skipped" in outcomes else "passed")
            # Omitted tests may also make an otherwise passing run incomplete.
            if record["outcome"] != aggregate and not (aggregate == "passed" and record["outcome"] == "incomplete"):
                raise ValueError("execution outcome disagrees with test results")
    if kind in {"execution", "decision"}:
        for key in ("evidence_ids", "execution_ids"):
            for target in record.get(key, []):
                if target not in records:
                    raise ValueError(f"unknown reference: {target}")
    if kind == "evidence":
        run = record["provenance"]["execution_id"]
        if run is not None and run not in records:
            raise ValueError(f"unknown producing execution: {run}")
    if record.get("supersedes") is not None and record["supersedes"] not in records:
        raise ValueError("superseded record does not exist")


def check_revision_snapshots(root, record, kind, validator):
    """Keep definitions recoverable even when staged work has not been committed."""
    if "revision_history" not in record:
        return
    directory = {"validation": "validations", "experiment": "experiments", "test": "tests"}[kind]
    for revision in range(1, record["revision"]):
        path = root / "validation/revisions" / directory / record["id"] / f"r{revision}.yaml"
        if not path.is_file():
            raise ValueError(f"missing prior revision snapshot: {record['id']} r{revision}")
        prior = read_yaml(path)
        validator.validate(prior)
        if prior["id"] != record["id"] or prior["revision"] != revision:
            raise ValueError("prior revision snapshot identity does not match its path")
        if prior["revision_history"] != record["revision_history"][:revision]:
            raise ValueError("prior revision history differs from retained history")
        if kind == "validation":
            if prior["origin"] != record["origin"]:
                raise ValueError("prior revision snapshot changed the historical research origin")
            if revision == 1 and prior["required_evidence"] != [prior["origin"]["required_closure_evidence"]]:
                raise ValueError("initial closure evidence was not preserved in the prior snapshot")
            check_history(prior, VAL_TRANSITIONS, "identified")
        elif kind == "experiment":
            check_history(prior, EXP_TRANSITIONS, "proposed")


def check_test_coverage(records):
    errors = []
    for identity, experiment in records.items():
        if not identity.startswith("EXP-"):
            continue
        tests = [records[ref["id"]] for ref in experiment["tests"]]
        mandatory = [test for test in tests if test["mandatory"]]
        covered = {hypothesis for test in mandatory for hypothesis in test["hypothesis_ids"]}
        required = {hypothesis["id"] for hypothesis in experiment["hypotheses"]}
        if not mandatory or covered != required:
            errors.append(f"{identity}: every hypothesis requires a mandatory test")
        modes = {test["execution_type"] for test in mandatory}
        kind = experiment["method"]["kind"]
        required_modes = {"manual", "automated"} if kind == "mixed" else {"manual"} if kind == "documentary" else {"automated"}
        if not required_modes <= modes:
            errors.append(f"{identity}: mandatory tests do not cover the declared investigation methods")
        for val in experiment["validates"]:
            if not any(val["id"] in {ref["id"] for ref in test["validations"]} for test in mandatory):
                errors.append(f"{identity}: no mandatory applicable test for {val['id']}")
    for identity, val in records.items():
        if identity.startswith("VAL-") and not any(test["mandatory"] for test in val["coverage"]["tests"]):
            errors.append(f"{identity}: test coverage requires a mandatory applicable test")
    return errors


def validate(root=ROOT, require_coverage=False, require_tests=False):
    errors, records, kinds, validators = [], {}, {}, {}
    base = root / "validation"
    for kind, directory in CATALOGS.items():
        try:
            schema = json.loads((base / "schemas" / f"{kind}.schema.json").read_text())
            Draft202012Validator.check_schema(schema)
            validators[kind] = Draft202012Validator(schema, format_checker=FORMATS)
            folder = base / directory
            if not folder.is_dir():
                raise ValueError(f"missing registry directory: {directory}")
            paths = sorted(path for path in folder.iterdir() if path.name != ".gitkeep")
            for path in paths:
                try:
                    if path.suffix != ".yaml" or not path.is_file():
                        raise ValueError("registry entries must be .yaml files")
                    record = read_yaml(path)
                    problems = list(validators[kind].iter_errors(record))
                    if problems:
                        for problem in problems:
                            location = ".".join(str(part) for part in problem.absolute_path)
                            errors.append(f"{path.relative_to(root)}:{location}: {problem.message}")
                        continue
                    identity = record["id"]
                    if path.stem != identity:
                        raise ValueError("filename must match record ID")
                    if identity in records:
                        raise ValueError(f"duplicate record ID: {identity}")
                    records[identity], kinds[identity] = record, kind
                except (OSError, ValueError, TypeError, yaml.YAMLError) as exc:
                    errors.append(f"{path.relative_to(root)}: {exc}")
        except (OSError, ValueError, SchemaError) as exc:
            errors.append(f"{kind} schema/registry: {exc}")
    if errors:
        return errors, records
    try:
        check_baseline(root, records)
    except (OSError, ValueError, KeyError, TypeError, yaml.YAMLError, SchemaError, ValidationError) as exc:
        errors.append(f"baseline migration: {exc}")
    for identity, record in records.items():
        try:
            check_relationships(record, kinds[identity], records)
            check_revision_snapshots(root, record, kinds[identity], validators[kinds[identity]])
        except (OSError, ValueError, KeyError, TypeError, yaml.YAMLError, ValidationError) as exc:
            errors.append(f"{identity}: {exc}")
    if require_coverage or require_tests:
        for identity, record in records.items():
            if kinds[identity] == "validation":
                coverage = record["coverage"]
                if coverage["status"] != "mapped" or not any(exp["mandatory"] for exp in coverage["experiments"]):
                    errors.append(f"{identity}: complete coverage requires a mapped mandatory investigation")
    if require_tests and not errors:
        errors.extend(check_test_coverage(records))
    return errors, records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT, help="repository root (default: script's repository)")
    parser.add_argument("--require-coverage", action="store_true", help="require a mandatory investigation for every validation")
    parser.add_argument("--require-tests", action="store_true", help="also require mandatory hypothesis, method and validation test coverage")
    args = parser.parse_args()
    errors, records = validate(args.root.resolve(), require_coverage=args.require_coverage, require_tests=args.require_tests)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"Validated {len(records)} records across {len(CATALOGS)} registries; 28 historical findings preserved.")
    print("Registry consistency only; execution evidence and reviewer acceptance are separate gates.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
