# SOM-first websites

SOM is usually something agents generate by compiling a web page.

A SOM-first website goes one step further. It is built so that:

1. The HTML and DOM compile into a high-signal SOM with minimal noise
2. Optionally, the site publishes SOM as an official alternate representation

This page is a best-practices guide for site owners and a checklist for Plasmate Labs properties.

---

## Goals

- Make pages easier for agents to understand
- Reduce token costs by avoiding boilerplate and duplicated content
- Make automation more reliable by keeping element structure stable
- Make caching easier by publishing stable, machine-readable representations

---

## Level 1: Make your DOM SOM-friendly

This is the most important step. If the HTML is semantic and stable, Plasmate can compile it into a great SOM.

### Semantic structure

- Use `main`, `nav`, `header`, `footer`, `article`, `section` for page layout
- Use a single real `h1` and a correct heading hierarchy (`h2`, `h3`, ...)
- Use real lists and tables (`ul`, `ol`, `li`, `table`) when content is a list or table

### Real interactive elements

- Prefer `button`, `a`, `input`, `select`, `textarea`
- Avoid clickable `div` elements as the primary UI control

### Labels and accessibility

- Use `aria-label` and `aria-describedby` where needed
- Use proper `<label for="...">` for form fields
- Make link text meaningful (avoid many identical "Learn more" links)

### Server-side rendering for primary content

- Ensure core content exists in the initial HTML
- Avoid putting the primary text behind delayed client-side fetch or late hydration

### Stability

- Keep stable IDs or stable ordering for key elements
- Avoid reshuffling the DOM on each deploy due to random classnames or keys

### Reduce noise

- Minimize repeated nav, footer, and unrelated "recommended" blocks
- Keep cookie dialogs from injecting large amounts of duplicate text into the DOM

---

## Level 2: Publish SOM as an alternate representation

This is optional, but powerful. A site can publish a canonical SOM JSON file so agents do not need to render or compile.

### Recommended patterns

#### A. A well-known SOM URL

Publish a SOM JSON representation at:

- `/.well-known/som.json`

This is easy to host on static sites.

#### B. An explicit alternate link

Add an alternate link in your HTML:

```html
<link
  rel="alternate"
  type="application/som+json"
  href="/.well-known/som.json"
  title="SOM"
/>
```

If you control HTTP headers, you can also send:

```http
Link: </.well-known/som.json>; rel="alternate"; type="application/som+json"; title="SOM"
```

#### C. Content negotiation

If you run an application server, you can support a dedicated endpoint, for example:

- `GET /som?url=https://example.com/page`
- `GET /page?format=som`

This is more complex but can be made fully automatic.

---

## How to generate a SOM file

If you own the site, you can generate the SOM using Plasmate and publish it as a static file.

Example:

```bash
plasmate fetch https://example.com > som.json
# publish som.json at /.well-known/som.json
```

---

## Checklist for Plasmate Labs properties

All Plasmate Labs properties should aim to:

- Use semantic HTML and accessible labels
- Avoid div-only UI for key controls
- Publish `/.well-known/som.json` and link it from HTML

If a property cannot publish SOM yet, it should at least meet the Level 1 DOM guidelines.

