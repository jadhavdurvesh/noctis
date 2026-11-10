# NOCTIS Browser — terminal alpha

This is the first navigable NOCTIS browser application. It uses:

- `noctis-net` for bounded HTTP/HTTPS transport.
- `noctis-html` for parsing returned HTML into NOCTIS's document tree.
- A NOCTIS-owned document view to display headings, paragraphs, lists,
  blockquotes, preformatted text, image alt text, and numbered links.

It does not embed Chromium, Gecko, WebKit, Electron, or WebView. JavaScript is
not executed. CSS is not yet applied to page layout or painting. The current
presentation is terminal text rather than a graphical browser window.

## Build

```sh
cargo build --release -p noctis-browser
```

Run with a URL:

```sh
cargo run --release -p noctis-browser -- https://example.com
```

Or start without an argument to get an address prompt:

```sh
cargo run --release -p noctis-browser
```

Bare hostnames default to HTTPS. Only HTTP and HTTPS URLs are accepted.

## Navigation

- Enter a number to open a numbered link.
- Enter a URL or hostname to navigate.
- `open URL` also navigates.
- `b` and `f` move through browser history.
- `r` reloads the current page.
- `h` shows help.
- `q` quits.

## Current safety/compatibility limits

- 20-second connection/request timeout.
- Maximum of eight redirects.
- Maximum response body of 5 MiB.
- HTTP and HTTPS only; embedded URL credentials are rejected.
- HTML and plain-text responses only. Binary downloads, images as page
  content, PDF, CSS resources, cookies, forms, authentication, JavaScript,
  service workers and browser storage are not implemented.
- The HTML parser is an experimental subset, not fully standards conformant.
  Treat this alpha as a development preview, not a secure browser for arbitrary
  untrusted websites.
- HTML is currently decoded as UTF-8 with replacement for malformed sequences;
  broader charset support is future work.
