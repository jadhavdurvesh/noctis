# NOCTIS Browser Alpha

## What works now

The `noctis-browser` application provides the first navigable browser loop
built from the NOCTIS workspace:

1. Accept a starting URL from the command line or address prompt.
2. Validate HTTP/HTTPS addresses and default bare hostnames to HTTPS.
3. Fetch pages using the bounded NOCTIS network layer.
4. Parse HTML with `noctis-html` into NOCTIS's own document tree.
5. Extract the title, headings, paragraphs, lists, blockquotes, preformatted
   text, image alternative text and links.
6. Resolve relative links against the final response URL.
7. Navigate links by number and move backward/forward through session history.

Build and start:

```sh
cargo build --release -p noctis-browser
cargo run --release -p noctis-browser -- https://example.com
```

Start without an argument for an address prompt:

```sh
cargo run --release -p noctis-browser
```

At the `noctis>` prompt, type a link number, a URL/hostname, `open URL`,
`b`, `f`, `r`, `h`, or `q`.

## Network limits

- HTTP and HTTPS only.
- Embedded URL credentials rejected.
- Eight redirects maximum.
- 20-second request/connect timeout.
- Five mebibyte response-body maximum.
- Only HTML/XHTML and plain text are rendered.
- HTML bytes currently use UTF-8 decoding with replacement characters for
  malformed sequences.
- No cookies, cache, authentication manager, content-encoding controls,
  downloads, or certificate UI yet.

## Rendering and document support limits

This is a terminal text browser, not a graphical browser. It uses the NOCTIS
HTML parser but does not yet apply CSS layout or paint. It does not execute
JavaScript, run event handlers, create browsing contexts, submit forms, or
support service workers, web storage, canvas, WebGL, media playback, or PDF
viewing.

The HTML parser and CSS subsystem are experimental subsets. A page may be
misparsed or displayed incompletely. Do not use this alpha for sensitive
accounts, payments, or arbitrary untrusted browsing.

## Security posture

The response cap and timeouts limit some resource-exhaustion cases, and URL
validation blocks non-web schemes and embedded credentials. They are not a
complete browser sandbox. The application does not yet isolate origins,
scripts, renderer processes, file access or downloads. JavaScript is disabled
rather than partially executed.

## Development sequence

The next browser milestones are:

1. Add CSS cascade integration to document presentation.
2. Build a real layout tree with block/inline formatting and text metrics.
3. Produce a display list and software rasterizer.
4. Add a platform window/input abstraction and a graphical desktop shell.
5. Add navigation UI, tabs, bookmarks, history storage and downloads.
6. Implement an independent JavaScript runtime and safe host APIs.
7. Harden navigation, origin separation, process isolation and web-platform
   conformance before claiming general-purpose browsing support.
