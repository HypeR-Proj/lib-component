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
