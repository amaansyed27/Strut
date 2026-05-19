# Security Policy

Strut is a local-first desktop application that can talk to cloud model providers, local model runtimes, and local coding agents. That makes the security model part of the product, not an afterthought.

## Supported Versions

Strut is pre-alpha. Security reports are welcome, but there are not yet stable releases.

## Security Principles

- API keys stay local and encrypted where the platform allows.
- BYOK requests must make context inclusion visible to the user.
- Local agent execution must be permissioned and auditable.
- The MCP server starts read-only by default.
- Write operations should be explicit and scoped.
- Provider adapters should block accidental private-network forwarding except approved local model endpoints.
- Verifier and import pipelines must treat external files as untrusted input.

## Reporting

Until the project has a public security contact, open a private maintainer channel before publishing sensitive details. Avoid posting live API keys, tokens, project data, or exploit payloads in public issues.
