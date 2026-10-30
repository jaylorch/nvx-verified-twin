---
name: documentation-updater
description: Keep docs accurate and up-to-date
intent: Keep NVX MicroVM design documentation aligned with the exact OpenVMM implementation vendored in the verified twin while avoiding speculative or duplicate updates.
on:
  schedule: weekly
  workflow_dispatch:
  skip-if-match:
    query: 'is:pr is:open in:title "[documentation-updater] "'
    max: 1
permissions:
  contents: read
  pull-requests: read
  copilot-requests: write
strict: true
engine:
  id: copilot
  version: "1.0.86"
  args:
    - --reasoning-effort
    - max
    - --context
    - long_context
model: gpt-5.6-sol-fast
max-turns: 100
timeout-minutes: 60
concurrency: documentation-updater
sandbox:
  agent:
    id: awf
    model-fallback: false
    token-steering: false
network:
  allowed:
    - defaults
    - github
tools:
  bash:
    - cat
    - find
    - git
    - grep
    - head
    - ls
    - python3
    - rg
    - sed
    - sort
    - tail
    - test
    - wc
steps:
  - name: Prepare documentation context and validate the baseline
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      REPO: ${{ github.repository }}
      DEFAULT_BRANCH: main
    run: |
      set -euo pipefail
      mkdir -p /tmp/gh-aw/agent
      cat > /tmp/gh-aw/agent/validate_design_docs.py <<'PY'
      import argparse
      import json
      import os
      import re
      import subprocess
      from pathlib import Path

      parser = argparse.ArgumentParser()
      parser.add_argument("--allow-unmaterialized-openvmm", action="store_true")
      args = parser.parse_args()

      root = Path.cwd().resolve()
      index = root / "doc" / "design.md"
      design_dir = root / "doc" / "design"
      openvmm_dir = (root / "openvmm").resolve()
      openvmm_materialized = (openvmm_dir / "Cargo.toml").is_file()
      errors: list[str] = []

      if not index.is_file():
          errors.append("missing doc/design.md")
      if not design_dir.is_dir():
          errors.append("missing doc/design/")

      chapters = sorted(design_dir.glob("*.md")) if design_dir.is_dir() else []
      documents = ([index] if index.is_file() else []) + chapters
      indexed_chapters: set[Path] = set()
      link_pattern = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")

      for document in documents:
          text = document.read_text(encoding="utf-8")
          relative = document.relative_to(root)
          if not text.strip():
              errors.append(f"{relative}: empty document")
              continue

          headings = re.findall(r"^# .+$", text, flags=re.MULTILINE)
          if len(headings) != 1:
              errors.append(
                  f"{relative}: expected exactly one top-level heading, found {len(headings)}"
              )

          fences = sum(1 for line in text.splitlines() if line.startswith("```"))
          if fences % 2:
              errors.append(f"{relative}: unbalanced fenced code block")

          for raw_target in link_pattern.findall(text):
              target = raw_target.strip().strip("<>")
              if (
                  not target
                  or target.startswith(("#", "http://", "https://", "mailto:"))
              ):
                  continue
              file_target = target.split("#", 1)[0]
              if not file_target:
                  continue
              resolved = (document.parent / file_target).resolve()
              try:
                  resolved.relative_to(root)
              except ValueError:
                  errors.append(f"{relative}: link escapes repository: {target}")
                  continue
              missing_unmaterialized_openvmm = (
                  args.allow_unmaterialized_openvmm
                  and not openvmm_materialized
                  and (resolved == openvmm_dir or openvmm_dir in resolved.parents)
              )
              if not resolved.exists() and not missing_unmaterialized_openvmm:
                  errors.append(f"{relative}: broken relative link: {target}")
              if document == index and resolved.parent == design_dir:
                  indexed_chapters.add(resolved)

      missing_from_index = [path for path in chapters if path not in indexed_chapters]
      for path in missing_from_index:
          errors.append(f"doc/design.md: missing chapter link for {path.name}")

      if errors:
          raise SystemExit("\n".join(errors))

      openvmm_tree = subprocess.run(
          ["git", "rev-parse", "HEAD:openvmm"],
          check=True,
          capture_output=True,
          text=True,
      ).stdout.strip()
      subprocess.run(
          ["git", "diff", "--exit-code", "HEAD", "--", "openvmm"],
          check=True,
      )
      untracked_openvmm = subprocess.run(
          ["git", "ls-files", "--others", "--exclude-standard", "--", "openvmm"],
          check=True,
          stdout=subprocess.PIPE,
          text=True,
      ).stdout
      if untracked_openvmm:
          raise SystemExit(
              "Untracked OpenVMM files are not part of HEAD:\n" + untracked_openvmm
          )
      context = {
          "default_branch": os.environ["DEFAULT_BRANCH"],
          "openvmm_tree": openvmm_tree,
          "repository_sha": subprocess.run(
              ["git", "rev-parse", "HEAD"],
              check=True,
              capture_output=True,
              text=True,
          ).stdout.strip(),
          "design_index": str(index.relative_to(root)),
          "design_chapters": [str(path.relative_to(root)) for path in chapters],
          "post_edit_validation": [
              "python3 /tmp/gh-aw/agent/validate_design_docs.py",
              "git diff --exit-code HEAD -- openvmm",
              "untracked=$(git ls-files --others --exclude-standard -- openvmm) "
              "&& test -z \"$untracked\"",
              "git diff --check",
          ],
      }
      Path("/tmp/gh-aw/agent/repository-context.json").write_text(
          json.dumps(context, indent=2) + "\n",
          encoding="utf-8",
      )
      print(
          f"validated {len(documents)} design documents against "
          f"vendored OpenVMM tree {openvmm_tree}"
      )
      PY
      python3 /tmp/gh-aw/agent/validate_design_docs.py --allow-unmaterialized-openvmm
      gh pr list \
        --repo "$REPO" \
        --state all \
        --search 'in:title "[documentation-updater] "' \
        --limit 10 \
        --json number,title,state,isDraft,createdAt,updatedAt,closedAt,mergedAt,url \
        > /tmp/gh-aw/agent/documentation-updater-pr-history.json
safe-outputs:
  mentions: false
  report-incomplete: {}
  create-pull-request:
    title-prefix: "[documentation-updater] "
    branch-prefix: "documentation-updater/"
    labels: [documentation]
    draft: true
    max: 1
    expires: 14
    base-branch: main
    allowed-files:
      - "doc/design/**"
    protected-files: fallback-to-issue
    fallback-as-issue: false
    if-no-changes: ignore
    max-patch-size: 512
    max-patch-files: 6
evals:
  questions:
    - id: operational_value
      question: Does the agent output demonstrate that a draft pull request corrected at least one specific mismatch between doc/design and the pinned OpenVMM MicroVM implementation?
    - id: pinned_revision
      question: Does the agent output identify the repository commit and exact vendored OpenVMM tree that were examined?
    - id: implementation_evidence
      question: Does the agent output cite specific OpenVMM source or test paths as evidence for the documentation changes?
    - id: scoped_changes
      question: Does the agent output show that every modified file is under doc/design/?
    - id: documentation_validation
      question: Does the agent output report that the design-document validator passed after the edits?
  model: small
---

# Documentation Updater

## Operational value

A successful run opens one draft pull request that corrects at least one
demonstrable mismatch in `doc/design/**` against the vendored OpenVMM MicroVM
implementation, cites the supporting code or tests, and passes documentation
validation.

## Required preparation

1. Read `README.md`, `.github/copilot-instructions.md`, `doc/design.md`,
   `doc/contribute.md`, `doc/build.md`, `doc/ci.md`, and
   `doc/project-structure.md`.
2. Read `/tmp/gh-aw/agent/repository-context.json` and
   `/tmp/gh-aw/agent/documentation-updater-pr-history.json`.
3. Confirm `openvmm/Cargo.toml` exists and run
   `python3 /tmp/gh-aw/agent/validate_design_docs.py`, which rejects both
   tracked changes and non-ignored untracked files under `openvmm/`.
   The verified twin vendors OpenVMM
   as a tracked source tree, not a private submodule. Do not fetch, initialize,
   replace, or edit it, and do not request a deploy key. If the source tree is
   unavailable or differs from HEAD, call `report_incomplete`; do not create a
   pull request. Record both `repository_sha` and `openvmm_tree` from context.
4. Read `openvmm/.github/copilot-instructions.md` before inspecting OpenVMM.

## Task

Compare the current design chapters with the vendored OpenVMM implementation and
tests. Treat implementation and executable tests as the source of truth. Use
`doc/design/code-ownership-map.md` to start from the relevant implementation
surface, then verify claims directly in code. Inspect only tracked implementation
and test paths in the recorded tree; ignored or generated content is not
implementation evidence.

Use the configured GPT-5.6 SOL Fast model, 1M-token long-context tier, and
maximum reasoning effort to inspect all relevant evidence before editing.
Optimize for reliable, roughly 80% success by selecting one coherent design
area per run instead of attempting a repository-wide rewrite. Prioritize
material drift in:

- public MicroVM configuration and CLI contracts;
- machine, device, interrupt, and fixed-placement ABI;
- snapshot format, capture, restore, CPU, RAM, and attachment behavior;
- console, network, filesystem, sandbox, and control-plane lifecycle;
- validation coverage, current limits, implementation status, and code
  ownership.

Inspect adjacent implementation and test paths when evidence crosses ownership
boundaries. Distinguish current behavior from future architecture: sections
marked **Proposed** remain proposals unless the pinned code and tests prove they
are implemented.

Create a focused documentation patch only when you can identify a specific,
material mismatch. Accuracy corrections include stale behavior, missing
implemented constraints, obsolete limits, incorrect implementation paths,
misclassified proposal status, or internally inconsistent design chapters.
Purely cosmetic rewriting does not qualify.

## Required validation

Before requesting the safe output:

1. Run `python3 /tmp/gh-aw/agent/validate_design_docs.py`.
2. Run `git diff --exit-code HEAD -- openvmm`.
3. Run `git ls-files --others --exclude-standard -- openvmm` and confirm its
   output is empty and its exit status is zero. Do not inspect untracked files
   as implementation evidence; the validator rejects them before writing context.
4. Run `git diff --check`.
5. Review `git diff -- doc/design` and confirm every changed statement is
   supported by the vendored OpenVMM source or tests.

The pull request body must state the repository SHA and OpenVMM tree SHA, the mismatches
corrected, the exact evidence paths inspected, and the validation results.

Reserve one model invocation after the final commit for `create-pull-request`.
Once the commit succeeds, call that safe output immediately without additional
searches or rereading passing validation logs.

## Boundaries

- **DO NOT** edit files outside `doc/design/**`.
- **DO NOT** modify OpenVMM source, manifests, CI
  configuration, workflow files, generated files, agent instructions, scripts,
  kernel inputs, Alpine files, or performance data.
- **DO NOT** delete design chapters, sections, rationale, diagrams, tables, or
  substantial existing content. Make the smallest evidence-backed correction.
- **DO NOT** erase or present proposed architecture as implemented without
  direct code and test evidence.
- **DO NOT** invent behavior from issue text, pull request discussion, comments,
  or memory when the pinned implementation does not support it.
- **DO NOT** broaden the patch to unrelated documentation cleanup.
- **DO NOT** create more than one pull request or use `gh` or GitHub APIs for
  writes.

If no material mismatch meets the evidence threshold, if the same correction
is already represented by recent workflow history, or if validation fails for
reasons caused by the proposed patch, call `noop` with a concise reason and
make no visible write. Use the configured `create-pull-request` safe output for
the single allowed write.
