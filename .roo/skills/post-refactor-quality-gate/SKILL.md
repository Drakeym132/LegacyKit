---
name: post-refactor-quality-gate
description: Runs a structured post-change quality pass after refactors or large code edits to verify behavior, detect bugs, flag optimization and structural issues, and drive cleanup using tests, lint/type checks, build checks, and targeted code review heuristics.
---

# Post-Refactor Quality Gate

## When to use this skill
- After a large refactor, migration, or broad codebase change.
- When a user asks to verify work quality, hunt regressions, or clean up structure.
- Before merge/release when confidence is needed across correctness + maintainability.

## When NOT to use this skill
- For implementing new features from scratch (use implementation-focused skills first).
- For one-file typo fixes where full-project validation would be wasteful.
- For deep security audits (use a dedicated security review skill/mode instead).

## Inputs required from the user
1. Scope of change (folders/files, PR, or commit range).
2. Success criteria (must-pass checks and acceptable risk level).
3. Constraints (time budget, areas that must not be modified).

## Workflow
1. **Define validation scope**
   - Map changed areas and impact radius (runtime paths, shared modules, build config).
   - Prioritize high-risk surfaces: core flows, state management, persistence, concurrency, and external integrations.

2. **Run deterministic project checks**
   - Run the project's native quality commands for changed scope first, then full scope when needed.
   - Typical command classes: format check, lint, type-check, unit/integration tests, and build.
   - Record failures grouped by: correctness, compilation/type, style, flaky behavior, performance regressions.

3. **Perform structural cleanup review**
   - Identify dead code, orphaned exports, stale TODOs, duplicate logic, and over-coupled modules.
   - Flag unnecessary abstraction layers introduced during refactor.
   - Check naming consistency, folder intent, and boundary clarity between modules.

4. **Check optimization opportunities**
   - Look for obvious hot-path inefficiencies (repeated expensive calls, avoidable allocations, redundant I/O).
   - Check N+1 or repeated fetch patterns, unnecessary rerenders/recomputations, and oversized dependency usage.
   - Propose low-risk optimizations first; defer risky micro-optimizations unless requested.

5. **Apply minimal, safe fixes**
   - Fix issues in smallest viable diffs.
   - Re-run only relevant checks after each fix, then run final aggregate checks.
   - Avoid behavior-changing refactors unless explicitly requested.

6. **Report outcome with prioritized actions**
   - Provide: issues found, fixes applied, remaining risks, and follow-up recommendations.
   - Prioritize by severity: block merge, should-fix, nice-to-have.

## Output format
- **Summary:** pass/fail status per check category.
- **Findings:** bug risks, optimization gaps, structural debt.
- **Fixes applied:** exact files and rationale.
- **Residual risk:** what was not verified and why.
- **Next actions:** smallest high-impact follow-ups.

## Troubleshooting
- If checks are slow, run targeted checks by changed packages/modules first, then full suite.
- If failures are noisy, isolate pre-existing issues vs regressions introduced by the change.
- If toolchain commands are unclear, inspect project manifests/config (e.g., npm/cargo/pytest/make targets) and use documented CI equivalents.
