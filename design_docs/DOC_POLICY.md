# Documentation Policy

> **Canonical core v1 (2026-08-24).** Everything from "## Core principles" down
> to the end of §10 is the shared core, copied verbatim into every repository
> under `Code` that keeps a `design_docs/`. It is not owned by any one repo:
> change it in all of them or not at all, so that `diff` between any two copies
> shows only local addenda. Repo-specific rules belong under
> **Local addendum** at the foot of this file, never inside the core.

## Core principles

### 1. Control doc growth

Add to an existing doc unless the material is substantial (>500 words), covers
a distinct topic, and is unrelated to any current document. Keep the total doc
count low. Do not create a file for a one-time analysis.

### 2. Eliminate redundancy

Audit before commits and after substantial changes. Newer documents are
generally more authoritative. If two docs disagree, reconcile them — do not let
drift accumulate. Material shared across several repos lives once, in a named
home, and is cited by path from the others; never copied.

### 3. No legacy friction

When a path changes, optimize for clean fit with the new path. Do not preserve
obsolete parallel systems or migration shims unless they are needed for real
user data. Tests track current semantics only.

### 4. Location and archival

- **Active docs** live directly in `design_docs/`. Flat is fine, and is the
  right default. When one domain accumulates enough material to justify it,
  promote that domain to an area root, `design_docs/<area>_docs/`.
- **Area roots**, once a repo has them, take a consistent set of category
  subdirectories. Use only the ones a given area needs:

  | Category | Holds |
  |---|---|
  | `research/` | briefs, surveys, reports, critiques, design probes |
  | `technical_architecture/` | component definitions, boundaries, interfaces, decisions |
  | `implementation_strategy/` | dated plans, development approaches, roadmaps |
  | `design/` | UI/UX, interaction design, accessibility |
  | `testing/` | test plans, harness docs, manual checklists |

- **Docs live with the repo that owns the subject, at that repo's doc root.**
  Do not scatter `design_docs/` into member crates of a workspace: a doc in a
  member crate is invisible to the canonical index, which is a violation of §6
  rather than a matter of taste.
- **Archive**: `design_docs/archive_docs/<YYYY-MM-DD>/` for retired plans and
  superseded notes. Check for an existing checkpoint folder before creating a
  new one. Move rather than delete; delete only with rationale and
  confirmation.

### 5. Cross-referencing

- Within a repo: relative links.
- Across repos: cite by path (`isometry/design_docs/...`), since relative links
  do not cross repository boundaries reliably and rot silently when the
  neighbour moves or is archived.
- Crates: link to crates.io when referring to a public API
  (`https://crates.io/crates/<name>`).
- When a doc moves, repair the links that pointed at it in the same session.

### 6. DOC_README authority

`design_docs/DOC_README.md` is the sole canonical index. It must contain:

- AI-assistant working principles for this project
- An index of all active docs with one-line descriptions
- Pointers to `DOC_POLICY.md` and `PROJECT_DESCRIPTION.md`

Any doc added, moved, or removed requires a `DOC_README.md` update in the same
session. If any other index disagrees with `DOC_README.md`, `DOC_README.md`
wins.

### 7. PROJECT_DESCRIPTION.md ownership

`design_docs/PROJECT_DESCRIPTION.md` — inside the doc root, not at the
repository root — is reserved for the maintainer. Do not edit it without
explicit instruction. Treat it as authoritative and surface contradictions for
discussion rather than resolving them silently.

The root `README.md` is derived from `PROJECT_DESCRIPTION.md` and the current
authoritative docs. Speculative features without plans appear only in
`PROJECT_DESCRIPTION.md`.

### 8. Plan documents

Work that changes code — not doc-only work — gets a dated plan named
`<YYYY-MM-DD>_<keyword>_plan.md`, in `design_docs/` or, where the repo has area
roots, in `<area>_docs/implementation_strategy/`. Each plan carries:

- A dated **Status** line, kept current: plan, in progress, landed, superseded
  by X.
- **Phases** organised by feature target and validation criteria, each with
  **done-conditions**. Never calendar labels — no "Day 1", no "Week 2" — and
  never time estimates.
- A **Findings** section for facts verified during the work, dated, with code
  references.
- A **Progress** log, dated, appended as phases land.

Code samples in a plan state whether they are illustrative or compile-ready.

Update the plan every two prompts on the project, or every two completed tasks.
Re-read it before resuming work rather than working from memory of it. On
completion, extract any deferred or still-open points into a new or existing
plan *before* moving it to `archive_docs/<date>/`.

### 9. Implementation feedback loop

Every implementation pass is also a design probe. After each pass, disseminate
structural learnings to the relevant plans and docs in the same session.
Surface architectural problems explicitly in the plan even when the fix is
deferred.

### 10. Workflow rule for AI assistants

Read `DOC_README.md` first, then this policy, before starting work. Any durable
working principle learned during a session is promoted into `DOC_README.md`'s
working-principles section in that same session.

## Local addendum — smolweb

`design_docs/` was founded 2026-08-24, when the canonical core was distributed
across the workspace and the spec-level documents were repatriated here from
mere. Before that this workspace had no design docs at all — only crate READMEs
and the workspace README.

### What belongs here

This workspace is **the wire layer**, and the doc tree follows the same line.
The rule was decided by Mark on 2026-08-03 and is recorded in full in
[smolweb_home_decision](technical_architecture/2026-08-03_smolweb_home_decision.md):

> Spec-accurate implementations of smolweb protocols and their document
> grammars belong here as general-use crates. The implementation-specific
> parts — enrichment of the browsing experience, lowering, rendering — stay in
> genet or the cambium view layer.

So a doc belongs in this repository when it is about **a protocol**: its wire
format, its document grammar, its spec provenance, what a crate must do to be
faithful to it. A doc about how a browser *uses* those protocols — rendering,
theming, capture, lowering, trust chrome — belongs to genet, and citing it by
path across the repo boundary (core §5) is the right move rather than copying
it here.

### Structure

Docs are flat plus two category roots, which is as far as core §4's promotion
rule has been exercised here:

```
smolweb/design_docs/
├── DOC_README.md              ← canonical index (§6)
├── DOC_POLICY.md              ← this file
├── technical_architecture/    ← the home rule, grouping, boundaries
└── research/                  ← protocol surveys and analyses
```

No area roots per protocol. Thirteen crates would mean thirteen near-empty
roots, and core §4 says flat is the right default until one domain earns
promotion.

### Two standing conventions from the crate work

Both were established in practice before this policy existed, and both are
documented rather than invented here:

- **A wire format is never guessed.** This workspace has declined to implement
  a protocol from inference three times (gopher's introductory page, terse, and
  scroll before its spec was found). A doc proposing a crate states where the
  spec was read; "reachable spec" and "blocked on a dead host" are legitimate
  recorded states.
- **The durable copy of a spec is as likely to live in an interoperating
  client's repository as on its author's host.** Record the provenance actually
  used, including when the spec titles itself speculative.

### PROJECT_DESCRIPTION.md

This workspace has no `design_docs/PROJECT_DESCRIPTION.md`. The root
`README.md` carries the product description today, so core §7's derivation rule
is inert here rather than violated.
