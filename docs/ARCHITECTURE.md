# NOCTIS Architecture

## High-level pipeline

```text
Browser UI (tabs, profiles, downloads)
             |
     Navigation / Session
             |
     Document orchestration
       /       |        \
     HTML     CSS      JavaScript
      |        |          |
     DOM   Computed     Runtime
           styles
       \      |         /
          Layout tree
              |
          Display list
              |
       Paint / Composite
              |
       Platform boundary
   Windows | Linux | SAND OS (future)
```

## Dependency direction
- Lower-level crates never depend on browser UI.
- HTML/CSS parsing never performs network I/O.
- DOM nodes never own platform window or graphics handles.
- Layout consumes document/style data and produces geometry.
- Painting consumes display data and never decides navigation.
- The platform crate exposes narrow OS capabilities to the browser shell.

## Subsystem ownership
- **core:** IDs, errors, shared lifecycle contracts.
- **html:** tokenizer, tree construction, DOM, serialization.
- **css:** tokenizer, parser, selectors, cascade, computed values.
- **layout:** formatting contexts, intrinsic sizing, geometry.
- **paint:** display list, rasterization interfaces, compositing.
- **net:** URL parsing, fetch policy, transport, cache boundaries.
- **js:** language parser, bytecode/VM, event loop, host bindings.
- **platform:** windows, input, timers, clipboard, fonts, graphics adapters.
- **browser:** profiles, tabs, navigation, UI and permissions.

## Security
All documents, URLs, images, fonts, scripts, and downloaded bytes are untrusted.
Keep file, network, script, and UI privileges separate. Design origin isolation
and sandbox boundaries before enabling arbitrary remote content.
