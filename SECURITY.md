# Security Policy

## Supported versions

LegacyKit is a small community project. Security fixes are issued only against
the current release line.

| Version | Supported |
|---|---|
| 1.0.x | ✅ |
| < 1.0 | ❌ |

## Reporting a vulnerability

**Please do not file public GitHub Issues for security vulnerabilities.**

To report a vulnerability, use GitHub's private security advisory feature:

> https://github.com/Drakeym132/LegacyKit/security/advisories/new

This sends the report directly to the maintainers without exposing it
publicly. Please include:

- A clear description of the issue and its impact
- Reproduction steps (or a proof-of-concept) on a specific OS / arch / version
- Any suggested mitigation, if you have one

You should expect an initial acknowledgement within a few days. If you have not
heard back within a reasonable window, feel free to ping the report through the
same advisory.

## Scope

**In scope:**

- The LegacyKit Tauri application (frontend + Rust backend) shipped from this
  repository
- The way LegacyKit integrates and invokes its bundled sidecar tools — for
  example, command-injection or path-traversal in arguments LegacyKit
  constructs, unsafe handling of workspace files, IPC misuse, or insecure
  defaults
- LegacyKit-managed configuration, credentials, and device backups on disk

**Out of scope:**

- Vulnerabilities in upstream sidecar tools themselves (e.g., bugs internal to
  futurerestore, idevicerestore, gaster, ipwnder, tsschecker, ipsw, kloader,
  irecovery, libimobiledevice, etc.). Please report those to their respective
  upstream projects. If LegacyKit's *integration* with such a tool turns a
  benign upstream bug into a security issue for end users, that integration
  layer is in scope.
- Issues that require an attacker who already has full local user access to the
  machine running LegacyKit (LegacyKit runs at user privilege; such an attacker
  already has equal or greater access)
- Risks inherent to iOS device modification — bricking, data loss during
  restores or downgrades, jailbreak instability — which are not security
  vulnerabilities

## Disclosure timeline

LegacyKit follows a coordinated disclosure model with a target window of **90
days** from the initial private report to public disclosure. The maintainers
will work with reporters to ship a fix in a 1.0.x patch release before that
window closes. If a fix is genuinely not feasible within 90 days the timeline
may be extended by mutual agreement.

Once a fix is released, the corresponding GitHub Security Advisory will be
published and credited.

## End-user notice

LegacyKit is community software for iOS device restore, downgrade, and
jailbreaking workflows. By design it invokes low-level tooling against
end-of-life iOS firmware. Users assume the risk for any device modification
performed with this tool, including bricking, data loss, and warranty
implications. The security policy above covers the LegacyKit application
itself and its handling of user data; it does not warrant the safety or
correctness of any specific restore or jailbreak operation against a given
device.
