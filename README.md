# Agent Skills

A collection of AI agent skills for Rust software development, compatible with [`skills` CLI](https://skills.sh) (`npx skills add`).

## Available Skills

- **`rust-sdd-planner`**: Spec-Driven Development planner for Rust projects. Discovers project structure (workspace vs single crate), ingests specs, and produces fine-grained plan files in `sdd/plans/` — each sized for an LLM agent to implement in one session. For workspaces, plans are broken down per-crate with dependency ordering.
- **`rust-sdd-executor`**: Spec-Driven Development executor for Rust projects. Reads a single plan file from `sdd/plans/`, implements every deliverable exactly as specified, runs verification (cargo check/test/clippy/fmt), fixes failures, and marks the plan complete. Designed to work with plans produced by `rust-sdd-planner`.
- **`rust-code-review`**: Comprehensive code review skill for Rust changes focusing on correctness, safety, performance, and idiomatic Rust patterns.
- **`rust-security-audit`**: Security audit skill supporting quick scan and deep audit modes for Rust codebases (vulnerabilities, unsafe soundness, secrets, panic surface, supply chain).

## Usage

To add all skills from this repository to your AI agent environment:

```bash
npx skills add <owner>/<repo>
```

To add a specific skill (e.g. `rust-sdd-planner`):

```bash
npx skills add <owner>/<repo> --skill rust-sdd-planner
```

To test a skill without installing:

```bash
npx skills use <owner>/<repo>@rust-sdd-planner
```

## Structure

```
.
├── rust-code-review/
│   ├── SKILL.md
│   └── evals/
├── rust-sdd-planner/
│   ├── SKILL.md
│   └── evals/
├── rust-sdd-executor/
│   ├── SKILL.md
│   └── evals/
└── rust-security-audit/
    └── SKILL.md
```

## License

[MIT](LICENSE)
