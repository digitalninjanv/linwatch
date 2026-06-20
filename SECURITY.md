# Security Policy

## Supported Versions

Security fixes target the latest released version.

## Reporting a Vulnerability

Please report suspected vulnerabilities through GitHub Security Advisories when available. If that is not possible, open a minimal issue without sensitive host details and request a private disclosure path.

## Security Model

`linwatch` is a local terminal monitor. It does not run a daemon, persist telemetry, or send network requests. It reads local Linux kernel/system data from `/proc`, `/sys`, `statvfs`, and a small set of documented optional local commands.

Treat all host data as sensitive when sharing JSON output or screenshots. Snapshots may include hostnames, process names, kernel versions, failed services, and local device details.
