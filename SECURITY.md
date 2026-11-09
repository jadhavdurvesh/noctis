# Security Policy

NOCTIS is experimental and must not be used as a general-purpose browser.

## Reporting
Please do not publish details of an exploitable security issue before a fix is
available. Open a private security advisory through GitHub's Security tab when
available, or contact the repository maintainer through the GitHub profile.

## Scope
Relevant reports include memory safety, parser crashes, out-of-bounds access,
unexpected file/network access, origin isolation failures, and unsafe handling
of untrusted documents.

## Current limitations
NOCTIS has no production sandbox, origin model, network stack, or mature
standards-compliant parser. Do not load untrusted content with this prototype.
