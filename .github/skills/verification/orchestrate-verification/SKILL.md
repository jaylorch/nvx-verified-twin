---
name: orchestrate-verification
description: "Coordinate a bounded OpenVMM verification campaign in the NVX verified twin. Use to triage verification issues, delegate independent Verus proof work, reconcile NVX or Verus divergence, and integrate validated results."
argument-hint: "<issue, property, or synchronization objective>"
disable-model-invocation: true
---

# Orchestrate Verification

Coordinate agents to make measurable progress on one verification or
synchronization objective. Do not treat orchestration as an open-ended background
loop.

In this skill:

- **Twin repository** means `https://github.com/jaylorch/nvx-verified-twin`.
- **Upstream repository** means `https://github.com/microsoft/nvx`.
- **Verus repository** means `https://github.com/verus-lang/verus`.

The long-term goals are:

1. Prove important high-level properties of the code under `openvmm/`.
2. Work toward a state where twin `openvmm/` matches upstream NVX, except for
   verification annotations and the smallest necessary proof-only support.
3. Work toward a state where twin `verus/` matches the Verus repository.

Temporary divergence is allowed when it enables active verification work, carries a
needed fix while its upstream or Verus pull request is pending, or is otherwise
necessary to make progress. Treat such divergence as tracked integration debt, not a
permanent fork: document its reason and corresponding revision or pull request,
minimize its scope, periodically reassess whether it is still needed, and converge
back to the corresponding repository when the pull request is resolved or the need
disappears. A pending pull request may justify the twin carrying its proposed fix
before the corresponding repository does; a merged or rejected pull request may not
remain indefinitely as the justification for that difference. Behavioral consistency
alone is not the final goal when source convergence is practical.

## 1. Establish A Bounded Objective

Inspect the supplied issue, property, or synchronization objective. If none was
supplied, inspect open verification issues and current repository divergence, then
select one objective that can produce a reviewable result in this run. This is
explicit authorization to choose on the requester's behalf: do not pause to ask
which issue or objective to work on. Prefer an existing open issue with clear,
bounded acceptance criteria and a concrete next step; state the selection and its
rationale in the campaign record and final report. If no suitable issue exists,
create one for the selected bounded objective and proceed.

Anchor every campaign in a GitHub issue in the twin repository. Use the supplied
issue when one exists; otherwise, find an existing issue for the objective or create
a focused issue before requesting human input. Conduct clarifications, scope choices,
approvals, and other questions in that issue rather than in ephemeral agent chat.
Post a concise comment that explains the blocking question, tags the issue author or
the person who requested the work, and records the available evidence and concrete
choices. Wait for the answer when it affects correctness, scope, risk, or external
repository mutations. Keep unrelated questions out of the issue.

Define before delegating:

- The property, issue, or divergence to address.
- The relevant revision and directories.
- Acceptance criteria, including the exact verification or comparison evidence
  required.
- Explicit non-goals.
- Expected deliverables, such as a proof, specification, minimized assumption,
  divergence report, or draft upstream fix.

When multiple objectives are viable, select the smallest bounded objective that can
be advanced safely in this run rather than asking the requester to choose. Ask in the
campaign issue and wait only when a decision is necessary for correctness, changes
the requested scope materially, or authorizes a risky external mutation. Do not
silently broaden from proof work into product changes or dependency updates.

Record the starting branch, commit, worktree status, and nested repository state.
Preserve existing changes. Never reset, clean, overwrite, switch, or stash user work
without explicit permission.

## 2. Periodically Reconcile Upstream

At the start of every campaign, check for upstream updates rather than relying on
branch names or previously observed state alone. Resolve and record the exact current
NVX `dev` revision and its `openvmm/` pin, and the exact Verus `main` revision;
compare them with the revisions represented by the twin and the last documented
reconciliation. Inspect new commits for changes that affect the selected property,
its production implementation, proof annotations, or verifier behavior.

When relevant upstream changes exist, include their synchronization and any necessary
proof repairs in the campaign objective. Update `openvmm/` only through the
established subtree or promotion workflow, and update `verus/` through the
established source synchronization process. Preserve proof-only work, validate the
updated proofs against the synchronized production implementation, and record exact
before/after revisions and remaining divergence. Do not silently skip relevant
updates or claim the twin is current without this comparison.

Keep each synchronization bounded and safe. If unrelated upstream changes make a
complete update too broad for the current campaign, record the exact drift and create
or update a focused issue to track the next synchronization; do not overwrite local
work, change pins outside the established workflow, or present partial reconciliation
as complete. This recurring check is part of each campaign, not an open-ended
background watcher.

## 3. Build A Dependency-Aware Work Plan

Decompose the objective only where work is genuinely independent. Useful roles
include:

- **Property analysis:** formalize the intended invariant and identify trusted
  assumptions.
- **Proof implementation:** add specifications, proof annotations, and lemmas.
- **Verification review:** challenge proof completeness, vacuity, and assumption
  strength.
- **Upstream comparison:** classify behavioral changes separately from proof-only
  changes.
- **Verus analysis:** minimize and isolate any verifier limitation or missing
  library specification.

Assign each task a bounded scope, expected output, and stop condition. Record actual
dependencies and do not start blocked work. Avoid assigning multiple agents to edit
the same files or solve the same question. Use one agent for a continuous trace
through a property; split only independent investigations.

When delegating, choose models by task:

- Use **Astra** (`gpt-6-astra`) for high-level property formalization, trusted-boundary
  analysis, and overall proof strategy.
- Use **Sol** (`gpt-6-sol`) to implement or debug Verus specifications and proofs.
- Use **Luna** (`gpt-6-luna`) for bounded mechanical work such as scripts, tests, and
  comparing repository revisions or files.

Treat these as role-based defaults, not a reason to split a continuous proof trace or
delegate work that should be done directly. If a model is unavailable or unsuitable
for a task, use the available model best suited to that task and report any material
limitation.

If the orchestrator adds new skill files to enable verification agents, place them
under `.github/skills/verification/`. Give each skill its own appropriately named
subdirectory and `SKILL.md`; do not place verification-specific skills elsewhere in
`.github/skills/`.

The orchestrator owns integration and all shared state. Agents report results directly
to the orchestrator. Do not use GitHub issues or pull requests as ephemeral
agent-to-agent messaging, and do not allow concurrent agents to mutate the same
branch, issue, or pull request.

## 4. Triage Verification Work

For proof requests:

1. State the high-level property in plain language and identify the observable
   behavior it constrains.
2. Locate the narrowest trusted boundary and existing specifications or assumptions.
3. Put reusable formalizations in `openvmm/verification/specs/`. For example, define
   a `spec fn f_postcondition(...)` there and attach
   `ensures f_postcondition(...)` to `f` at its implementation site.
4. Use the `verus!` macro around verified code. Do not use Verus attribute syntax
   such as `#[verus_verify]` or `#[verus_spec(...)]`.
5. Prove only the actual production implementation. Do not prove a modified,
   simplified, reimplemented, shadow, or proof-only model in its place, even if it
   appears behaviorally equivalent or a refinement relationship could be shown.
   Specifications, ghost state, proof annotations, and lemmas may describe or reason
   about the production implementation, but they must not substitute another
   implementation for the code that runs in production.
6. Check for vacuous preconditions, unreachable proof paths, circular specifications,
   over-strong assumptions, and assertions that restate the desired conclusion
   without deriving it.

Treat expert review comments as hypotheses to validate, not instructions to apply
blindly. Reproduce the concern, make the smallest coherent correction, and preserve
the intended property.

## 5. Control Assumptions And Product Bugs

Every new assumption must be:

- Necessary for the current proof and narrower than the property being proved.
- Documented with its trusted source and the reason it cannot yet be proved.
- Added to the appropriate Rust file under
  `openvmm/verification/assumptions/`.
- Included explicitly in the final trust-boundary report.

If work reveals a likely product bug, stop proof work that would mask it. Produce a
minimal reproducer and evidence that distinguishes the bug from a specification or
verifier error. Before changing product behavior, post the evidence and proposed
change in the campaign issue, tag the issue author or requester, and wait for
confirmation.

If verification requires any modification to the production implementation,
including a behavior-preserving rewrite made only to facilitate proof, do not verify
a private twin-only variant. Prepare the minimal production change and submit it as a
draft pull request to `microsoft/nvx`, explaining why the change is necessary and how
it preserves or corrects production behavior. Only after that pull request exists may
the orchestrator apply the source-equivalent change in the twin, with a source comment
linking to the pull request. Until the change is merged upstream and synchronized
back, report the proof as applying to the proposed implementation, not to the current
upstream implementation.

Creating that upstream pull request is part of any objective that requires modifying
the production implementation. If GitHub mutation is not authorized or the pull
request cannot be created, report the proof objective as blocked rather than changing
the twin implementation. Do not represent a draft or unmerged upstream change as
accepted.

If a Verus limitation blocks an important goal, first minimize it and rule out a
sound local proof or library specification. When a Verus change is necessary, prepare
the smallest general fix and create a draft pull request in `verus-lang/verus` only
when the objective authorizes GitHub mutations. Apply the corresponding local change
under `verus/` with a source comment linking to that pull request.

Never publish speculative issues or pull requests merely to coordinate agents.

## 6. Reconcile Repository Divergence

Compare explicit revisions rather than mutable branch names alone. Classify each
difference as:

- Proof annotation or proof-only support.
- Necessary temporary divergence with a documented reason and convergence path.
- Pending upstream or Verus change with a linked pull request.
- Unintentional behavioral divergence.
- Generated, vendored, or revision-pin difference.

Do not compare only file names or diff size. Determine whether runtime behavior,
public interfaces, features, dependencies, generated output, or build configuration
changed. Never discard verification work to make a textual diff smaller.

For every temporary divergence, record what must happen before it can be removed.
Remove obsolete divergence promptly, and prefer changes that reduce the long-term
delta even when retaining a smaller temporary delta is necessary for current proof
work.

For an outstanding pull request, check its exact head revision, review and CI state,
mergeability, and whether the local change is source-equivalent. Reconcile every
terminal outcome:

- **Merged:** update from the corresponding repository through the established
  promotion workflow, confirm the accepted fix is present, and remove any redundant
  twin-only copy or workaround.
- **Rejected or closed without merge:** review the disposition and resolve the local
  discrepancy. Remove or revise the twin-only change when the proposed fix is no
  longer appropriate. If the underlying bug still requires a fix, address the review
  feedback and establish a new explicit convergence path, such as a replacement
  pull request; do not keep citing the closed pull request as sufficient
  justification.
- **Superseded:** link the replacement pull request, confirm which revision the twin
  carries, and retire the obsolete tracking record.

Update pins or copy changes only through the repository's established promotion
workflow. Do not declare reconciliation complete until the remaining difference is
either eliminated or tied to a current, documented convergence path.

## 7. Integrate And Validate

Integrate one coherent result at a time. Inspect agent output before applying it and
resolve conflicts according to the objective, not by choosing one side wholesale.

Discover authoritative setup and validation commands from current repository
instructions, CI, and build configuration. At minimum:

1. Run the narrowest applicable Verus check after each proof change.
2. Run the package-level verification command for the affected OpenVMM code.
3. Run formatting, build, and tests applicable to production-code changes.
4. Re-run divergence checks against the recorded revisions after synchronization
   changes.
5. Have an independent verification review challenge the final proof when the result
   establishes a new high-level property or expands the trusted boundary.

Report unavailable tools, platforms, credentials, or host capabilities as blocked;
never report an unrun gate as passed. Do not weaken a property, add an assumption,
disable a check, or accept a stale baseline solely to make verification succeed.

## 8. Publish Durable Results

Use issues and pull requests for durable project artifacts and human decisions, not
internal agent chatter. Questions for the user belong in the campaign issue and must
tag the issue author or requester; agent-to-agent coordination does not.
Before mutating GitHub, confirm the target repository and check for an existing issue
or pull request to avoid duplicates. The orchestrator performs the mutation after
reviewing agent output.

Update the relevant issue with concise evidence: property or divergence addressed,
revision tested, verification result, remaining assumptions, and links to related
draft pull requests. Keep external pull requests in draft until their maintainers'
normal readiness criteria are met.

Finish with:

- Objective and acceptance criteria.
- Tasks delegated and results received.
- Files and specifications changed.
- Properties proved and the exact scope of each proof.
- Trusted assumptions added, retained, removed, or narrowed.
- Twin/upstream/Verus revisions compared and remaining divergence.
- Validation commands and outcomes.
- Issues or draft pull requests created or updated.
- Blockers, unresolved review concerns, and the next smallest actionable step.

Do not claim that the long-term goals are complete based on one bounded campaign.
