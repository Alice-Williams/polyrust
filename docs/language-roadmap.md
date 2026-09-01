# Target-language roadmap

## What “most common” means

There is no single authoritative top-ten list. Surveys count people who often
use several languages, while repository rankings count activity rather than
developers. Percentages therefore overlap and cannot be added into a defensible
“99% of programmers” coverage claim.

For a reproducible activity-oriented baseline, GitHub's 2025 Octoverse reports
the top ten languages by monthly contributors as:

1. TypeScript
2. Python
3. JavaScript
4. Java
5. C#
6. PHP
7. Shell
8. C++
9. HCL
10. Go

Source: [GitHub Octoverse 2025 analysis](https://github.blog/news-insights/octoverse/what-the-fastest-growing-tools-reveal-about-how-software-is-being-built/).
The [2025 Stack Overflow Developer Survey](https://survey.stackoverflow.co/2025/technology)
is a complementary self-reported usage source; it includes markup and query
languages such as HTML/CSS and SQL that are not general-purpose PolyRust
emission targets.

## PolyRust target strategy

After M21, PolyRust covers six general-purpose ecosystems:

- Rust
- TypeScript
- JavaScript
- Python
- Go
- Java

M22 commits the next two targets:

1. C++, using modern value types and RAII.
2. C, with an explicit ABI, allocation, ownership, and monomorphization design.

Recommended order after M22:

1. C#, because its generics, records, interfaces, exceptions, and UTF-16 strings
   map closely to the existing checked IR and Java backend.
2. PHP, for broad server-side and web-library coverage.
3. Kotlin, sharing JVM tooling while adding null-safety and sealed-type evidence.
4. Swift, for Apple-platform reach and a strong typed value model.
5. Zig, as its own systems-language backend rather than a conversion hub.

Shell and HCL rank highly in repository activity but are domain-specific
automation/configuration languages rather than suitable general-purpose
backends for the current IR. They should eventually be considered as narrower
IR profiles, not full semantic targets.

## C versus C++

C++ should precede C. The current IR has records, interfaces, tagged enums,
`Option`, `Result`, immutable generic lists, owned strings, and structured
failures. Modern C++ can represent these with library types and RAII. C needs a
stable generated ABI, explicit allocation/ownership rules, concrete
monomorphization, and tagged runtime structures before equivalent behavior can
be claimed safely.

The long-term goal should be coverage of the dominant general-purpose
ecosystem families, measured separately against GitHub activity and developer
survey data, rather than an unsupported additive 99% figure.

## Why Zig is not the conversion hub

Zig is valuable for C interoperability, cross-compilation, and as a
Clang-compatible C compiler driver. Its official translation feature,
`zig translate-c`, translates C into Zig. Zig can export a C ABI, but that
produces linkable artifacts and headers rather than behaviorally equivalent,
readable C and C++ source trees. PolyRust therefore keeps checked IR as the
semantic hub and can add Zig later as another independently tested target.
