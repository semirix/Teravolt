# Teravolt - Multi-stream executor
[![](https://docs.rs/teravolt/badge.svg)](https://docs.rs/teravolt/)

Teravolt is an executor for handling streaming data from multiple sources and
enabling seamless communication between them.

> **Warning**: This is far from being complete and lacks many features at the
> moment. Use at your own risk!

## Goals
- Speed.
- Ergonomics.
- Abstract the general nastiness of dealing with multi-threaded async futures
  with inter-process communication.
- Reliability.

## Features
- [x] Multiple tasks communicating with each other.
- [x] Custom task restart policies.
- [x] Handling of errors in tasks cleanly.
- [ ] Subscribe to messages.
- [ ] Distribute task workloads over multiple threads.
- [ ] Procedural macros to create new tasks.
- [ ] Support mutable internal state for connections.

## License
```
Copyright 2020 Semirix

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this software except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```