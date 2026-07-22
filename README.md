# Agent Skills

A collection of AI agent skills for Rust software development, compatible with [`skills` CLI](https://skills.sh) (`npx skills add`).

## Available Skills

- **`rust-sdd`**: Spec-Driven Development (SDD) for Rust projects. Enforces structured workflows: discovery → spec ingestion → phased plan → step-by-step implementation → zero-gap review.
- **`rust-code-review`**: Comprehensive code review skill for Rust changes focusing on correctness, safety, performance, and idiomatic Rust patterns.
- **`rust-security-audit`**: Security audit skill supporting quick scan and deep audit modes for Rust codebases (vulnerabilities, unsafe soundness, secrets, panic surface, supply chain).

## Usage

To add all skills from this repository to your AI agent environment:

```bash
npx skills add <owner>/<repo>
```

To add a specific skill (e.g. `rust-sdd`):

```bash
npx skills add <owner>/<repo> --skill rust-sdd
```

To test a skill without installing:

```bash
npx skills use <owner>/<repo>@rust-sdd
```

## Structure

```
.
├── rust-code-review/
│   ├── SKILL.md
│   └── evals/
├── rust-sdd/
│   ├── SKILL.md
│   ├── evals/
│   └── test-fixtures/
└── rust-security-audit/
    └── SKILL.md
```

## License

[MIT](LICENSE)
