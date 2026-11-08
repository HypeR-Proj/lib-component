<!--
SPDX-FileCopyrightText: 2026 roolrz
SPDX-License-Identifier: Apache-2.0
-->

# lib-component

Native component management, connection and installation protocols. This library
owns client and wire helpers; the component-manager service owns lifecycle state.

## Workspace tests

Use the [HypeR workspace](https://github.com/HypeR-Proj/HypeR) to resolve dependencies:

```sh
python3 project.py test lib-component
```

Paths in the documentation are relative to the repo-managed workspace root.
Edit this repository and prepare the workspace again before building.

## Admission boundary

Connection `receive`/`accept` and installation `receive` return the
`hyper_service_ipc::Admission` outcome. Invalid request payloads and rolled-back
transfers reject one request before it reaches the lifecycle owner. Endpoint and
local SDK failures remain errors. Installation also scopes private second-stage
closure, timeout and cancellation to that installation request; accepted object
ownership is transferred only in the returned call.

## Connection failures and publication

`connect`, `publish`, `status` and `report_failure` return `ConnectionError`.
Match `Transport` for local/transport failures and `Rejected` for the manager's
exact `RejectionKind` and diagnostic. In particular, `Rejected(Unavailable)`
differs from `Transport(PEER_CLOSED)`. Keep this error in the caller's error type
or a boxed error; it has no implicit conversion to a Native status. The library
does not reconnect, retry a rejected request or restart dependent components.

A process being running does not imply its protocols are published. `publish`
makes an offered protocol routable; successful `connect` means delivery of the
server endpoint, not completion of application initialization or acceptance of
a business operation. Startup `Notification::Ready` is a separate coordinator
milestone. Prepare the protocol receiver and its state before publication.

Connection replies fit the complete 1024-byte encoded frame. Rejection kinds
remain exact; diagnostic text is shortened only as needed to fit JSON escaping
and UTF-8 expansion, preserving a complete character prefix. Reply delivery
failure does not undo an already applied lifecycle operation.

`report_critical_failure` reports an authenticated local subsystem failure that
requires CRITICAL severity even when the component itself is not configured as
critical. It uses the component manager's Console fallback independently of the
journal. Diagnostic text is bounded by the complete wire frame; the sender should
retain its full structured incident for administrative queries.

The typed object inventory supports `ExecutableAuthorityObject` for explicit
object-capability routes. Carrying this authority does not implicitly grant it
to children: the deployment must export and offer the object, and the target
component must request it with a `use` declaration. The SDK publication API
creates an immutable executable copy; component routing supplies authority,
not a bypass for writable/executable memory restrictions.
