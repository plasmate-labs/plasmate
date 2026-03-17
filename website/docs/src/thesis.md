---
tags: [project, agentic-browser, thesis, evaluation]
created: 2026-03-15
status: draft
---

# Netscape for the Agentic Web - Thesis and Evaluation

## 1. The Kernel (David's Thesis, normalized)

### 1.1 Netscape lesson
Netscape did not only build a browser UI. It created foundational infrastructure and protocols that defined how the world interacted with the web (SSL, JavaScript, cookies).

### 1.2 The diagnosis
The current agentic browser ecosystem is built on legacy foundations:

- Most tools control Chromium through CDP (Chrome DevTools Protocol).
- CDP was designed for human debugging and developer tooling.
- This creates a mismatch: agents need semantics and intent, not pixels and debug APIs.

### 1.3 The proposed leap
To dominate agentic browsing, abandon legacy browser architecture and build an engine natively designed for LLM agents.

### 1.4 Proposed architecture (high level)

1. **Rust headless-native engine**
   - Strip human UI elements.
   - Optimize for concurrency, memory safety, and determinism.

2. **Stealth networking layer (stealth core)**
   - Bake anti-detection into the network stack.
   - TLS fingerprint control (JA3/JA4 style control), HTTP/2 tuning, proxy routing.

3. **Semantic Object Model (SOM)**
   - Compile HTML into a deterministic semantic representation.
   - Drop visual rendering and CSS bloat.
   - Output clean JSON or Markdown tree optimized for LLM consumption.

4. **Sandboxed JS execution**
   - Execute JavaScript enough to support SPAs and state transitions.
   - Isolate memory per request/session so the engine can run thousands of concurrent sessions.

5. **Kill CDP with a new protocol (AWP)**
   - Replace coordinate commands with intent-based actions.
   - Example: `action: checkout_cart` instead of `click(x,y)`.

6. **Extension ecosystem via WebAssembly skills**
   - Developers write "skills" (Stripe Checkout Resolver, Salesforce Navigator).
   - Compile to Wasm and inject into the engine.
   - This allows continuous improvement without updating core.

### 1.5 Bottom line
Lightpanda is an optimization of the old way. The new venture should be the foundation of the new way.

---

## 2. My Evaluation

### 2.1 Thesis correctness
The core diagnosis is correct: CDP is not agent-native. It carries overhead and brittleness because it is a debugging interface.

### 2.2 Why this is a real venture
If successful, this is infrastructure that:

- every agent framework will embed,
- every AI automation startup will depend on,
- and enterprises will pay heavily for at scale.

This is a platform, not a feature.

### 2.3 What would be genuinely new
The three truly differentiating primitives are:

1. **SOM (semantic output)**
   - Token efficiency and determinism are the unlock.

2. **AWP (agent-native protocol)**
   - A dedicated standard for agent browsing is a long-term moat.

3. **Wasm skills marketplace**
   - The ecosystem moat.
   - This becomes compounding capability and distribution.

### 2.4 Threats and constraints

1. **Complexity risk**
   - Building browser engines is hard.
   - Even headless-only still requires broad JS and Web API coverage.

2. **Standards window risk**
   - Chrome shipping agent-oriented tooling could narrow the protocol window.
   - AWP needs to ship quickly as a spec and SDK and win mindshare.

3. **Stealth arms race**
   - Anti-bot is ongoing R and D.
   - Proxies and fingerprinting require continuous updates.

4. **Migration inertia**
   - CDP has massive adoption.
   - AWP must offer a migration path (shims, compatibility layers, incremental adoption).

### 2.5 The right sequencing
This is not a 2 week product.

The correct first move is:

- publish AWP as an RFC-style draft,
- prove SOM token efficiency with a Rust PoC,
- show benchmark superiority (tokens, memory, startup, concurrency),
- then raise to build the full engine.

---

## 3. What "Full Spectrum" Means (Deliverables)

To hand this to an agentic build team and get a Rust proof of concept, the documentation package must include:

1. **Product Spec**
   - Vision, market, competitive map
   - Full architecture
   - Roadmap and team plan

2. **Protocol Spec (AWP)**
   - Transport, message schema
   - Session model
   - SOM observation and element addressing
   - Intent actions and skill invocation

3. **PoC Build Brief**
   - Exact scope for the first Rust implementation
   - Acceptance criteria
   - Benchmark harness and token counting method

In this folder you now have:

- `SPEC.md`
- `AWP-SPEC.md`

Next recommended doc: `POC-BUILD-BRIEF.md`

---

## 4. Naming and positioning

Working names that fit the thesis:

- **Plasmate** - foundation, structural backbone
- **Wake** - trail of a fast-moving machine
- **Prow** - cutting edge

Positioning statement:

"Plasmate is an open-source, agent-native headless browser engine. It replaces CDP with AWP, replaces DOM dumps with a Semantic Object Model, and grows capability through a Wasm skill marketplace."

---

## 5. Summary

This is a legitimate attempt to build the foundational infrastructure for the agentic web.

If executed, it becomes:

- the runtime browser engine for most agent frameworks,
- the protocol standard for agent browsing,
- and the commercial control plane for large-scale agent fleets.

The key is to win the standard and the ecosystem, not to compete on milliseconds.
