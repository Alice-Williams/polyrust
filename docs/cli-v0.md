# CLI and safe output v0

Status: stable for the v0 prototype (M09).

## Commands

```text
polyrust check <input.poly.json> [--target <id>] [--json]
polyrust emit <input.poly.json> --target <id> --out <directory>
              [--dry-run] [--json] [--option <name=value>]...
polyrust targets [--json]
polyrust explain <code> [--json]
```

`check` parses and semantically checks the whole input. With `--target`, it also
validates target compatibility and options and performs whole-program capability
preflight without calling generation. `emit --dry-run` performs generation and
manifest validation, prints files/dependencies/helpers, and never creates or
changes the output directory. `targets` is generated from the runtime registry
and reports backend/IR versions and each capability's support level.

JSON diagnostics are a single JSON array on standard error and never contain
ANSI escapes. Successful JSON command output is one JSON value on standard
output.

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | Success |
| 2 | Invalid command line or unknown diagnostic code |
| 3 | Input read, JSON parse, or semantic-check failure |
| 4 | Target lookup, compatibility, option, preflight, or generation failure |
| 5 | Output transaction or standard-output failure |

These meanings are stable within v0.

## Output transaction and recovery

Generation first produces a validated in-memory `OutputManifest`. The writer
canonicalizes the explicit parent, rejects symlink/reparse-point roots and
ancestors, and verifies every joined and resolved destination stays below the
output root. It stages and syncs all bytes in a sibling transaction directory
before writing a recovery journal or modifying output.

During commit, only paths named by the manifest may be replaced. Existing files
at those paths are moved to the transaction backup; unknown files are never
deleted. Normal errors trigger immediate rollback. If the process is interrupted
after the journal is durable, the next write to the same output detects the
fixed sibling transaction directory and restores backups/removes newly created
manifest files before beginning another transaction. The transaction directory
may also be recovered by rerunning the same `emit` command. v0 intentionally has
no clean mode.

This protocol protects failure atomicity and recovery for a single writer. Users
must not concurrently mutate the same generated paths while an emit is running.
