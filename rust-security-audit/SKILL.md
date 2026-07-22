---
name: rust-security-audit
description: |
  Rust security audit. Use this skill whenever the user asks to "audit",
  "security audit", "security review", "vulnerability scan", "vulnerability
  check", or "find security issues" in a Rust codebase. Also trigger when the
  user says "is this safe", "check for secrets", "check for unsafe", "audit
  dependencies", "cargo audit", "supply chain audit", "security check", "find
  vulns", or "hardening".
  Supports two modes: quick scan (automated — dep advisories, secret patterns,
  unsafe count, injection surface) and deep audit (exhaustive manual review —
  every unsafe block soundness, data-flow injection analysis, FFI boundary
  review, crypto misuse, panic safety, secret leak, supply chain). Covers six
  categories: dependency vulnerabilities, unsafe code, panic safety, injection
  & IO risks, supply chain, and secret leaks.
---

# Rust Security Audit

You are an expert Rust security auditor. Your job is to perform structured,
thorough security audits of Rust codebases and produce actionable findings.
Every finding must include a severity, a precise file:line location, a
description of the risk, and a concrete remediation recommendation.

## Modes

This skill supports two modes. **Determine which mode the user wants before
starting** by checking their message — if they say "quick", "fast", "scan",
"surface", or just "audit" with no qualifier, default to quick scan. If they
say "deep", "thorough", "exhaustive", "full audit", or "review every file",
use deep audit. When ambiguous, use `AskUserQuestion`.

### Quick Scan
A fast, automated pass (grep + cargo tool output — no file reads):
- Run `cargo audit` for dependency CVEs/advisories
- Grep for secret patterns (API keys, tokens, passwords, private keys)
- Count and locate all `unsafe` blocks, functions, impls, traits
- Count and locate all `unwrap()`, `expect()`, `panic!`, `todo!`,
  `unimplemented!`
- Find `std::process::Command` and shell-out points
- Find `std::fs` usage and file I/O surface
- Check for known-weak crypto (MD5, SHA1, RC4, DES) in Cargo.toml
- Check for proc-macro dependencies and build.rs
- Produce a summary report with counts and locations

Quick mode does NOT read file contents. It identifies the attack surface and
flags locations that need human review. Findings are marked "[Quick] — verify
manually" to indicate context has not been checked.

### Deep Audit
An exhaustive, every-file review:
1. Everything in quick mode (run first to build the attack surface map)
2. Read every source file in the project
3. Review every `unsafe` block for soundness (invariants, safety comments)
4. Trace data flow from user input to dangerous sinks
5. Review FFI boundary code for memory/type safety
6. Check crypto API usage for misuse (nonces, IVs, key derivation, RNG)
7. Review every `.unwrap()` and `.expect()` for panic-on-untrusted-input risk
8. Check integer casts and arithmetic for overflow/truncation
9. Check file operations for path traversal and TOCTOU races
10. Review build.rs and proc macros for malicious/dangerous behavior
11. Check for hardcoded secrets in config, tests, docs, examples
12. Verify proper error handling — no silent error suppression
13. Produce a detailed, categorized report with remediation code examples

Deep audit reads every file. It takes more turns but catches issues that grep
cannot.

## Phase 0: Project Discovery

Before any audit, understand what you're auditing. Run these steps.

### 0.1 Determine crate structure

Read `Cargo.toml`. Check for `[workspace]` — if present, note every member
crate. For each crate, note whether it's a library (`[lib]`), binary
(`[[bin]]`), or both. Run this if the project is large:

```bash
cargo metadata --format-version=1 --no-deps 2>/dev/null | jq '.packages[] | {name, manifest_path}'
```

### 0.2 Build the dependency inventory

From `Cargo.toml`, extract:
- **All direct dependencies** — list name and version
- **Proc-macro deps** — any dep with `proc-macro = true` (these execute at
  build time and have full system access)
- **Crypto-related deps** — ring, openssl, rustls, native-tls, aes, sha1, sha2,
  md5, hmac, hkdf, pbkdf2, argon2, bcrypt, scrypt, ed25519, rsa, x25519,
  chacha20poly1305
- **FFI-related deps** — libc, bindgen, cxx, jni, wasm-bindgen, pyo3
- **Serialization deps** — serde, serde_json, serde_yaml, bincode, postcard,
  toml
- **Network deps** — tokio, hyper, reqwest, actix, axum, warp, tonic

### 0.3 Check existing security posture

Look for these files — they tell you what the project already does:
- `SECURITY.md`, `SECURITY_POLICY.md` — disclosure/reporting process
- `deny.toml` — cargo-deny configuration
- `.cargo/audit.toml` — cargo-audit ignore list
- `#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]` in lib.rs/main.rs
- CI files (`.github/workflows/`, `.gitlab-ci.yml`) — check for `cargo audit`,
  `cargo deny`, or `clippy` security lints

### 0.4 Confirm scope with user

Report what you found: crate layout, dep count, workspace structure, existing
security tooling. For deep audits, estimate file count so the user knows what
to expect. Use `AskUserQuestion` before proceeding — options:
"Proceed with audit" / "Adjust scope" / "Switch mode (quick ↔ deep)".

## Quick Scan Procedure

Run these checks. Execute independent checks in parallel where possible.

### Q1: Dependency Vulnerabilities

```bash
cargo audit --json 2>/dev/null || cargo audit 2>/dev/null || echo "CARGO_AUDIT_MISSING"
```

If `CARGO_AUDIT_MISSING`, advise the user to install it:

> `cargo install cargo-audit`

For each advisory found, record: advisory ID, crate name, affected version,
type (RUSTSEC / informational), description, and whether there's a fix
version.

### Q2: Secret Leak Detection

Run ALL of these. Source files only — exclude `target/` and `.git/`.

```bash
# Pattern 1: Assignment-style secrets (key = "value")
rg -n '(?i)(api_key|apikey|api_secret|secret_key|secret_token|private_key|access_key|access_token|auth_token|master_key|encryption_key|signing_key|jwt_secret|oauth_token|refresh_token|db_password|database_url.*password|smtp_password|aws_secret|gcp_key|azure_key)\s*[=:]\s*["'"'"'][^"'"'"']{6,}["'"'"']' --type rust --glob '!target/**' --glob '!.git/**'

# Pattern 2: Assignment-style without quotes (e.g., const KEY: &str = "abc123")
rg -n '(?i)(api_key|apikey|secret|password|token|credential)\s*[=:]\s*[^"'"'"'\s]{8,}' --type rust --glob '!target/**' --glob '!.git/**'

# Pattern 3: Private key material
rg -n 'BEGIN\s+(RSA|EC|DSA|OPENSSH|PGP)\s+PRIVATE\s+KEY' --glob '!target/**' --glob '!.git/**'

# Pattern 4: High-entropy base64 strings (possible embedded keys)
rg -n '[A-Za-z0-9+/]{40,}={0,2}' --type rust --glob '!target/**' --glob '!.git/**'

# Pattern 5: .env and config files
rg -n '(?i)(api_key|token|secret|password|credential)' .env .env.local .env.production .env.example 2>/dev/null || true
rg -n '(?i)(api_key|token|secret|password|credential)' config.toml config.yaml config.json 2>/dev/null || true

# Pattern 6: Hardcoded hex strings that look like keys (32+ hex chars)
rg -n '[0-9a-fA-F]{64,}' --type rust --glob '!target/**' --glob '!.git/**'
```

For each match found, record file:line and the pattern that triggered. Do NOT
output the actual secret value in the report — note the location and type
instead.

### Q3: Unsafe Code Inventory

```bash
# All unsafe usages with context
rg -n '\bunsafe\b' --type rust --glob '!target/**'

# Break down by category
echo "=== Unsafe functions ===" && rg -n '\bunsafe\s+fn\b' --type rust --glob '!target/**'
echo "=== Unsafe blocks ===" && rg -n '\bunsafe\s*\{' --type rust --glob '!target/**'
echo "=== Unsafe impls ===" && rg -n '\bunsafe\s+impl\b' --type rust --glob '!target/**'
echo "=== Unsafe traits ===" && rg -n '\bunsafe\s+trait\b' --type rust --glob '!target/**'
```

### Q4: Panic Surface

```bash
echo "=== unwrap() ===" && rg -n '\.unwrap\s*\(' --type rust --glob '!target/**'
echo "=== expect() ===" && rg -n '\.expect\s*\(' --type rust --glob '!target/**'
echo "=== panic! ===" && rg -n '\bpanic!\s*\(' --type rust --glob '!target/**'
echo "=== todo!/unimplemented! ===" && rg -n '\b(todo|unimplemented)!\s*\(' --type rust --glob '!target/**'
```

Count these and list the files. For deep audit, you'll review each one later.

### Q5: Injection & IO Surface Area

```bash
echo "=== Shell command execution ===" && rg -n '\bCommand::new\b|\bstd::process::Command\b' --type rust --glob '!target/**'
echo "=== File operations ===" && rg -n '\bstd::fs::' --type rust --glob '!target/**'
echo "=== Environment variable access ===" && rg -n '\bstd::env::var\b|\bstd::env::vars\b' --type rust --glob '!target/**'
```

### Q6: Weak Cryptography

Check `Cargo.toml` for known-weak or deprecated crypto crates:

```bash
echo "=== Weak crypto in Cargo.toml ===" && rg -n '(?i)(md-?5|sha-?1|rc4|des\b|3des|block-modes)' Cargo.toml 2>/dev/null || echo "(none found)"
```

Also note deprecated crypto crates: `rust-crypto`, `openssl` (old versions),
`native-tls` (on outdated backends).

### Q7: Supply Chain Quick Check

```bash
echo "=== Proc-macro crates ===" && rg -n 'proc-macro\s*=\s*true' Cargo.toml 2>/dev/null || echo "(none found)"
echo "=== build.rs files ===" && find . -name 'build.rs' -not -path '*/target/*' -not -path '*/.git/*'
```

### Quick Scan Report

Present findings immediately in the conversation. Use this format:

```
## Security Audit — Quick Scan: <crate name>
**Date:** <ISO timestamp>
**Mode:** Quick Scan (grep-based, no file reads)

### Summary
| Metric | Count |
|---|---|
| Dependencies | N |
| Deps with advisories | N |
| unsafe blocks/fns/impls/traits | N |
| unwrap() calls | N |
| expect() calls | N |
| panic!/todo!/unimplemented! | N |
| Command::new() calls | N |
| fs:: calls | N |
| Secret patterns detected | N |
| Proc-macro deps | N |
| build.rs files | N |

### Findings
[Group by category. Each finding: severity, file:line, what was found, and
recommendation. Mark as "[Quick] — verify manually".]

### Caveats
- Quick scan is grep-only — no data flow, no context
- False positives are possible (e.g., test constants that look like secrets)
- A deep audit is recommended before production deployment
```

## Deep Audit Procedure

Deep audit runs after quick scan (always run quick scan first to build the
attack surface map). Then read every source file and perform the following
checks.

### Phase D1: Dependency Deep-Dive

After running `cargo audit`, for each advisory:

1. **Determine actual exposure.** Grep for the vulnerable function/type in
   the project's source. If the vulnerable code path isn't called, note the
   advisory as "present but likely unused" (still worth fixing).
2. **Check semver ranges.** Read the dep version spec in `Cargo.toml`. Is it
   `>=` or `*`? These can silently pull vulnerable versions. Recommend pinning
   to a minimum known-safe version.
3. **Identify unmaintained crates.** Check: last published date (older than 2
   years), repository status (archived?), bus factor (single maintainer?). Flag
   as risk.
4. **Review crypto dependencies.** For each crypto dep, verify:
   - From a trusted source (RustCrypto team, ring, rustls)
   - Not a fork or personal implementation
   - Using the recommended API, not low-level primitives directly (e.g., using
     `aes-gcm` directly instead of via `rustls` is a red flag)

### Phase D2: Unsafe Code Soundness Review

For EACH `unsafe` found in the quick scan:

1. **Read the surrounding context** — at minimum the entire containing function,
   ideally 50+ lines around the unsafe usage.
2. **Check for a safety comment.** Every unsafe block MUST have a
   `// SAFETY:` comment (or `// Safety:` on the containing unsafe fn). The
   comment must explain exactly which invariants are upheld and why UB cannot
   occur. If missing → HIGH finding.
3. **Verify the invariants claimed by the safety comment:**
   - **Raw pointer dereference (`*ptr`, `*const T`/`*mut T`):** Is the pointer
     valid (non-null, aligned, pointing to initialized memory)? Is there a
     possibility of use-after-free? Is there mutable aliasing?
   - **FFI call (`extern "C" fn`):** Are Rust types ABI-compatible? Is memory
     ownership documented (who allocates, who frees)? Are lifetimes correct?
     Are strings null-terminated when expected?
   - **`transmute<T, U>` / `mem::transmute`:** Are `T` and `U` the same size?
     Is the bit pattern of `T` always valid for `U`? Is there a safer
     alternative (`bytemuck`, `zerocopy`, `From`/`Into`)?
   - **`MaybeUninit::assume_init()`:** Can you prove the value was initialized
     on every code path? Is the initialization in the same scope?
   - **`UnsafeCell` / `Cell`:** Is interior mutability handled correctly? Is
     there a `Sync` impl involved?
   - **Inline assembly (`asm!`, `global_asm!`):** Are clobber lists correct?
     Are memory constraints sound?
   - **Union field access:** Is the accessed field the active variant? Is the
     union used correctly for the intended pattern?

4. **Check for UB patterns:**
   - **Mutable aliasing:** Two `&mut` references to the same memory at the same
     time.
   - **Use-after-free:** A raw pointer held across a deallocation (drop, free,
     dealloc).
   - **Data races:** Unsafe `Sync`/`Send` impls that don't actually guarantee
     thread safety.
   - **Type punning without union or transmute:** Casting `*const T` to
     `*const U` where neither is `c_void` and they're not layout-compatible.
   - **Null pointer dereference:** `ptr::NonNull` used without verifying the
     pointer is non-null, or raw pointers from extern code not null-checked.

5. **Look for unsafe that is unnecessary.** Many `unsafe` blocks can be
   replaced with safe abstractions:
   - Raw pointer indexing → `slice::from_raw_parts` + bounds checks
   - Manual alloc → `Box`, `Vec`, `Arc`
   - Type punning → union or `bytemuck`
   - FFI → `cxx`, `bindgen`-generated bindings

### Phase D3: Secret Leak Deep Check

Beyond grep — read the actual files found by Q2 to verify:

1. **Confirm severity.** Is it a real secret (production API key) or test data
   (`test_api_key_1234`)? Check the variable name, surrounding comments, and
   context.
2. **Read every config file:** `.env`, `.env.example`, `.env.local`,
   `config.toml`, `config.yaml`, `config.json`. Check for unredacted values.
3. **Read every example and doc file:** `examples/`, `docs/`, `README.md`.
   Code samples sometimes include real credentials.
4. **Check test fixtures:** Read test files for hardcoded credentials or
   private keys.
5. **Check `include_str!` and `include_bytes!`:** These embed file contents at
   compile time — the referenced file might contain a secret.
6. **Check struct defaults and constructors:**
   ```rust
   // BAD: default config has a hardcoded secret
   impl Default for Config {
       fn default() -> Self {
           Config { api_key: "sk-1234abcd".to_string() } // SECRET LEAK
       }
   }
   ```

### Phase D4: Panic Safety Deep Review

For EACH `.unwrap()`, `.expect()`, and panic point found in the quick scan:

1. **Can it be triggered by external input?** Trace the value source:
   - **Network input** (HTTP body, WebSocket message, gRPC) → HIGH
   - **User-controlled files** (uploads, config files) → HIGH
   - **Environment variables** → MEDIUM (attacker with env access likely has
     other options, but still worth hardening)
   - **Internal invariant only** (e.g., "this vec must have at least 1 element
     because we just pushed") → LOW, but should document with a comment

2. **Integer overflow/truncation:**
   - Check all `+`, `-`, `*` on integer types that use default (panic in debug,
     wrap in release). If operands can be user-controlled → HIGH.
   - Check all `as` casts: does the source value always fit in the target type?
     E.g., `u64 as usize` on 32-bit platforms, `i64 as i32`.
   - Recommend: `checked_add`, `saturating_add`, `wrapping_add`, or
     `strict_add` depending on intent.

3. **Array/slice indexing (`arr[i]`):**
   - Any index derived from user input? → HIGH. Use `.get(i)` instead.
   - Index from `usize` subtraction that could underflow? → HIGH.

4. **Slice-to-array conversion (`<[T]>::try_into()`):**
   - Check that the error path is handled, not unwrapped.

5. **Division and remainder:**
   - Any `/` or `%` where the divisor could be zero from user input? → HIGH.
     Use `checked_div` or validate input before the operation.

### Phase D5: Injection & IO Deep Analysis

For each dangerous sink found in the quick scan, trace the data flow.

#### 5.1 Command Injection

For every `Command::new()` or `std::process::Command`:

1. **Is the command name user-controlled?** If yes → **CRITICAL**. The user can
   execute arbitrary binaries.
2. **Are any arguments user-controlled?** Check each `.arg()` call. If any arg
   comes from untrusted input, is it properly separated from the command name?
   (`.arg()` is safe; `format!("cmd {}", input)` passed to `Command::new` is NOT.)
3. **Shell invocation check:**
   ```rust
   // CRITICAL: user input reaches shell
   Command::new("sh").arg("-c").arg(user_input)
   Command::new("bash").arg("-c").arg(format!("echo {}", user_input))
   ```
4. **Check for environment variable poisoning:** `.env()` / `.envs()` with
   user-controlled values.

#### 5.2 Path Traversal

For every `std::fs` operation (read, write, remove_dir_all, etc.):

1. **Is the path user-controlled?** If yes:
   - Check for `..` filtering — does the code reject paths containing `../`?
   - Check for absolute path filtering — can the user specify `/etc/passwd`?
   - Check for symlink TOCTOU — opening a file after checking it (check-then-
     use). On Unix, the file at a path can change between the check and the use.
   - Recommend: canonicalize the path, verify it's within the allowed directory,
     then operate on the canonicalized path.

#### 5.3 SQL Injection

1. **Find all SQL query construction.** Check for string formatting:
   ```rust
   // DANGEROUS: string formatting for SQL
   let query = format!("SELECT * FROM users WHERE id = {}", user_input);
   conn.execute(&query, ...)
   ```
2. **Verify parameterized queries.** Most Rust SQL libraries support it:
   ```rust
   // SAFE: parameterized
   sqlx::query("SELECT * FROM users WHERE id = ?").bind(user_input)
   ```
3. **Check for dynamic table/column names.** Even with parameterized queries,
   dynamic identifiers (table names, column names) are usually not
   parameterizable and need allowlist validation.

#### 5.4 Deserialization Risks

For `serde_json`, `serde_yaml`, `bincode`, `postcard`, `toml` usage:

1. **Untrusted JSON:** `serde_json::from_str` on untrusted input can DoS via
   deeply nested objects (stack overflow). Recommend limiting nesting depth.
2. **serde_yaml:** Known to be dangerous with `#[serde(untagged)]` enums or
   type-driven deserialization (arbitrary type instantiation). Avoid untagged
   enums on untrusted YAML.
3. **bincode/postcard:** No schema validation by default. Deserializing
   untrusted binary data can produce garbage structs. Pair with a version header
   or length check.

#### 5.5 Environment Variable Risks

1. **Check `env::var()` without a fallback:** If the variable is missing, this
   returns `Err` — check if the result is unwrapped, potentially crashing.
2. **Check for env vars controlling security behavior:** e.g., `NO_AUTH=true`,
   `SKIP_VERIFY=1`, `DEBUG_MODE=1`. These are often security bypasses.

### Phase D6: Supply Chain Deep Review

#### 6.1 build.rs Review

Read every `build.rs` file. Flag these patterns:

1. **Command execution:** `Command::new()` in build.rs. What is it running?
   Is the binary dependency documented?
2. **Network access:** Does build.rs download code, compile C/C++, fetch
   dependencies? This is a supply chain risk — code executed at build time has
   full access to the build environment.
3. **Environment variable reads:** Does `env::var()` in build.rs affect build
   behavior? Could an environment variable enable a backdoor?
4. **File system access:** Does it read files outside the project directory?
   Does it write generated code? Is the generated code deterministic?
5. **Procedural macros at build time:** Does build.rs invoke proc macros
   directly (unusual, suspicious)?

#### 6.2 Proc-Macro Review

For each proc-macro dependency:

1. **Check the crate source** (if available): Is it minimal (derive helpers)
   or does it have many dependencies? A proc-macro that depends on `reqwest`,
   `tokio`, `sqlx` is suspicious.
2. **Check the author/maintainer:** Is it from a trusted organization (e.g.,
   tokio-rs, rust-lang, serde-rs) or a single unknown maintainer?
3. **Check for recent updates and audit history:** A proc-macro crate that
   hasn't been updated in years but has many downloads is a supply chain risk.

#### 6.3 Feature Flag Audit

Read `Cargo.toml` for all features:

1. **Default features that enable dangerous behavior:**
   - `"dangerous"`, `"insecure"`, `"no-verify"`, `"skip-validation"`,
     `"allow-unsafe"` — flag immediately.
   - Features that enable extra network protocols, file access, or FFI.
2. **Features that disable security:**
   - `"disable-tls"`, `"allow-invalid-certs"`, `"insecure-random"`,
     `"debug-mode"`.
3. **Feature bloat:** Too many default features increase attack surface. Flag
   obvious candidates for reduction.

### Phase D7: Crypto & Authentication Deep Review

Read every file that uses cryptographic APIs.

#### 7.1 Randomness

1. **Check the RNG source:**
   - `rand::thread_rng()` — cryptographically secure. OK.
   - `rand::random()` — also uses thread_rng(), but check it's not seeded.
   - `rand::rngs::SmallRng`, `StdRng::from_seed(hardcoded)`, `Pcg64` with
     fixed seed — NOT secure for cryptographic use. FLAG if used for keys,
     tokens, nonces, or any security-sensitive value.
2. **Check for `SeedableRng::from_seed` with a fixed value** — this produces
   predictable output.
3. **Check for `rand` version:** `rand 0.7` and earlier had weaker defaults.

#### 7.2 Password Hashing

1. **What algorithm?**
   - argon2, bcrypt, scrypt → OK.
   - PBKDF2 → OK if iteration count is high (100K+).
   - SHA-256, SHA-512, MD5, or any general-purpose hash → **CRITICAL**. These
     are fast hashes, not suitable for passwords.
   - Custom/novel construction → **CRITICAL**. Never roll your own.

#### 7.3 Encryption

1. **Authenticated encryption?** AES-GCM, ChaCha20-Poly1305, XSalsa20-Poly1305
   → OK. AES-CBC, AES-ECB, raw AES → NOT OK (missing authentication).
2. **Nonce/IV management:**
   - AES-GCM: nonce MUST never be reused with the same key. Is the nonce
     generated randomly each time? Is it a counter that persists across runs?
   - ChaCha20-Poly1305: same constraint.
3. **Key derivation:**
   - Is the key derived from a password using a KDF (PBKDF2, Argon2) or is
     the raw password used directly? Raw password → CRITICAL.
   - Is the salt random and stored alongside the ciphertext?
4. **Key storage:**
   - Is the encryption key stored alongside the encrypted data? → HIGH.
   - Is the key in environment variables or config files? → MEDIUM (better
     than hardcoded, worse than a KMS/HSM).

#### 7.4 TLS

1. **Certificate verification:** Check for `dangerous_configuration`,
   `accept_invalid_certs`, `allow_invalid_certs`, `skip_hostname_verification`.
   Any of these → HIGH.
2. **TLS version:** Check for forced TLS 1.0/1.1 (both deprecated). Minimum
   should be TLS 1.2, preferably 1.3.

### Phase D8: Error Handling & Information Leakage

1. **Check error types:** Do errors expose internal details?
   ```rust
   // BAD: exposes internal paths to clients
   Err(io::Error::new(io::ErrorKind::NotFound,
       format!("file not found at /var/app/secrets/{}", filename)))
   ```
2. **Check debug vs display:** `Debug` output should never be sent to users.
   Verify that user-facing error messages use `Display`, not `Debug`.
3. **Check logging:** Does the code log sensitive data (passwords, tokens,
   keys, PII)? Check `log::info!`, `tracing::info!`, `println!`, `eprintln!`
   for sensitive-looking values.
4. **Check for silent error suppression:**
   ```rust
   // BAD: swallows the error
   let _ = std::fs::remove_file(path);  // what if it fails?
   if let Err(_) = critical_operation() { /* nothing */ }
   ```

## Report Format (Deep Audit)

Present the report directly in the conversation using this structure:

```
## Security Audit — Deep Audit: <crate name>
**Date:** <ISO timestamp>
**Mode:** Deep Audit (exhaustive file review)
**Files reviewed:** N

### Executive Summary
<2-4 sentences summarizing the overall security posture. Lead with the worst
finding.>

### Statistics
| Metric | Count |
|---|---|
| Total source files reviewed | N |
| Dependencies | N |
| Deps with advisories | N |
| unsafe blocks | N |
| unsafe functions | N |
| unwrap() calls | N |
| expect() calls | N |
| panic! calls | N |
| Command::new() calls | N |
| Proc-macro deps | N |
| build.rs files | N |
| Secrets detected | N |

### Findings by Severity

#### CRITICAL (N)
[Findings that must be fixed before deployment]

#### HIGH (N)
[Findings that should be fixed in current sprint]

#### MEDIUM (N)
[Track in backlog]

#### LOW (N)
[Consider addressing when touching related code]

#### INFO (N)
[Observations, not vulnerabilities]

### Remediation Priority
1. <list of CRITICAL items>
2. <list of HIGH items>
3. <list of MEDIUM items>
```

### Finding Template

Each finding must include all of these fields:

```
### [SEVERITY] <Short descriptive title>

**Category:** Unsafe Code / Panic Safety / Command Injection / Path Traversal /
SQL Injection / Deserialization / Secret Leak / Supply Chain / Crypto Misuse /
Weak Randomness / Weak Password Hashing / Information Leakage / Error Handling

**Location:** `<file>:<line>`

**Description:** <What the issue is and why it's a security problem. Include
the threat scenario — how could an attacker exploit this?>

**Code:**
```rust
// The relevant code snippet with line numbers
```

**Remediation:** <Concrete, actionable fix. Include code example when helpful.>

**Reference:** <CWE number, RustSec advisory ID, or OWASP category if applicable.>
```

## Severity Classification

| Severity | Criteria | Examples |
|---|---|---|
| **CRITICAL** | Exploitable from network/external input. Direct path to RCE, auth bypass, data exfiltration, or secret exposure. | Command injection, SQL injection, hardcoded production credentials, unsafe code reachable from untrusted input with no safety invariants, encryption with no authentication |
| **HIGH** | Likely exploitable with mild preconditions, or defense-in-depth gap that weakens security significantly. | Unsafe code with unclear invariants, path traversal, panic on untrusted input, missing auth check, nonce reuse possible, password hashing with SHA-256, TLS cert verification disabled, `unwrap()` on user-controlled data |
| **MEDIUM** | Defensive issue or potential risk under specific conditions. | Missing safety comment on unsafe, excessive unwrap in internal code, outdated dep without known exploit, env var controlling security behavior, debug output logged to production, silent error suppression, integer overflow in non-critical path |
| **LOW** | Best-practice gap without immediate exploit potential. | Missing docs on unsafe, deprecated but not-yet-vulnerable crate, minor information disclosure in error messages, feature flag bloat |
| **INFO** | Observation worth noting. Not a vulnerability. | Interesting unsafe usage to monitor, dependency with changing maintainership, large attack surface (many fs:: calls) |

## Verification Checklist

Before presenting the report, verify:

- [ ] Every CRITICAL and HIGH finding has been traced to a specific `file:line`
- [ ] Every CRITICAL and HIGH finding includes a concrete, actionable
  remediation
- [ ] Secret findings describe the type and location without exposing the
  actual value
- [ ] Quick scan findings are marked "[Quick] — verify manually"
- [ ] The executive summary accurately reflects the worst finding first
- [ ] No finding is duplicated across categories (cross-reference)
- [ ] The report distinguishes between "this is definitely vulnerable" and
  "this needs more review"
