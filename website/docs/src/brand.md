# Plasmate Brand Guide

> "The living information, the plasmate, replicates itself - not through information or in information - but *as* information."
> -- Philip K. Dick, VALIS (1981)

---

## Origin Story

The name **Plasmate** comes from Philip K. Dick's novel VALIS, where the plasmate is described as "living information" - an energy form that crosses between realities, transforms raw matter into structured intelligence, and replicates itself through contact with human consciousness.

This is exactly what Plasmate the browser engine does: it transforms the raw, chaotic HTML of the web into a **Semantic Object Model** - living, structured information that AI agents can understand, reason about, and act upon.

**Plasma is the fourth state of matter.** Agents are the fourth state of the web.

| State | Matter | Web |
|-------|--------|-----|
| 1st | Solid | Static HTML |
| 2nd | Liquid | Dynamic (JavaScript) |
| 3rd | Gas | Real-time (WebSockets) |
| **4th** | **Plasma** | **Agentic (SOM)** |

The "mate" suffix carries double meaning: a companion (your agent's browser companion) and the act of bonding (the plasmate "crossbands" with the agent, just as Dick described it bonding with human consciousness).

---

## Brand Personality

**Pioneer.** We are building the Netscape of agentic browsers. The first to reject CDP and DOM as the interface between agents and the web.

**Precise.** This is an engine, not a wrapper. Every design decision reflects engineering discipline.

**Open.** Apache 2.0. The standard belongs to everyone. The brand should feel institutional enough to trust, not corporate enough to fear.

**Alive.** The plasmate is living information. The brand should feel energetic, luminous, present - never cold or sterile.

---

## Color System

### Philosophy

Plasma in nature is not one color. It is determined by the gas being ionized and the energy level of the excitation. Lightning is blue-white (nitrogen ionization at extreme temperature). Neon is amber-red. Solar corona is golden-white. The Plasmate palette draws from these real plasma emission colors.

**No purple.** No gradients. Flat, confident, scientific.

### Primary Palette

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Plasma White** | `#F0EDE8` | 240, 237, 232 | Backgrounds, body text areas. Warm white - not blue-white, not stark. Like the afterglow of an ionized field. |
| **Core Black** | `#0D0D0D` | 13, 13, 13 | Primary text, dark backgrounds, terminal UI. Near-black with warmth. |
| **Ember** | `#E8853A` | 232, 133, 58 | Primary brand color. The color of neon plasma - ionized gas at mid-energy. Used for the wordmark, primary CTAs, key accents. |

### Secondary Palette

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Arc** | `#3D8FD4` | 61, 143, 212 | Secondary accent. The color of an electric arc - nitrogen plasma at high energy. Used for links, interactive states, code highlights. |
| **Ion** | `#D4C5A0` | 212, 197, 160 | Tertiary. Warm sand/parchment tone. Backgrounds, cards, subtle dividers. The color of low-energy plasma glow. |
| **Ash** | `#6B6560` | 107, 101, 96 | Muted text, captions, borders. Warm gray - the residue after plasma discharge. |
| **Corona** | `#F5C842` | 245, 200, 66 | Highlight/attention only. The solar corona. Use sparingly for badges, warnings, "new" tags. |

### Dark Mode

| Name | Hex | Usage |
|------|-----|-------|
| **Void** | `#111110` | Dark background |
| **Deep** | `#1A1918` | Cards, elevated surfaces |
| **Smoke** | `#2A2825` | Borders, dividers |
| **Dim** | `#8A8480` | Muted text in dark mode |

### Usage Rules

1. **Ember is the signature.** When you see that warm orange on a dark field, you should think Plasmate. It should appear on every page, but not dominate.
2. **Arc is functional.** Blue is for clickable things, focused states, and code. It is not decorative.
3. **Never use gradients.** Flat color only. A gradient says "startup trying to look cool." A flat field of Ember on Core Black says "we built a browser engine."
4. **Corona is rare.** It exists for moments of delight or urgency. If everything is golden, nothing is.
5. **White space is structural.** Plasmate design uses generous whitespace. The content breathes.

---

## Typography

### Philosophy

Plasmate is infrastructure. The typography should feel engineered, not designed. We avoid Inter (ubiquitous to the point of invisibility), geometric sans-serifs that feel like SaaS dashboards, and anything with "startup energy."

### Type Stack

**Primary: Space Grotesk**
- Proportional sans-serif derived from Space Mono
- Open source (SIL OFL), designed by Florian Karsten
- Has a mechanical, engineered quality without being cold
- Distinctive letter shapes (the 'a', the 'g') give it identity
- Use for: headlines, navigation, UI labels, the wordmark lockup

**Secondary: IBM Plex Mono**
- Monospaced, designed for IBM's systems identity
- Open source (SIL OFL)
- Carries the weight of infrastructure heritage
- Use for: code blocks, terminal output, technical specs, element IDs, protocol names
- Also use for small data labels, version numbers, and timestamps

**Body: IBM Plex Sans**
- Pairs with IBM Plex Mono from the same family
- Open source (SIL OFL)
- Highly readable at body sizes, neutral but not generic
- Use for: documentation, long-form text, descriptions, README content

### Type Scale

```
Display:    Space Grotesk Bold     48px / 56px line
H1:         Space Grotesk SemiBold 36px / 44px line
H2:         Space Grotesk Medium   28px / 36px line
H3:         Space Grotesk Medium   22px / 30px line
Body:       IBM Plex Sans Regular  16px / 26px line
Body Small: IBM Plex Sans Regular  14px / 22px line
Caption:    IBM Plex Sans Regular  12px / 18px line
Code:       IBM Plex Mono Regular  14px / 22px line
Code Small: IBM Plex Mono Regular  12px / 18px line
Label:      Space Grotesk Medium   11px / 16px line, UPPERCASE, 0.08em tracking
```

### CSS Implementation

```css
@import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap');

:root {
  --font-display: 'Space Grotesk', system-ui, sans-serif;
  --font-body: 'IBM Plex Sans', system-ui, sans-serif;
  --font-mono: 'IBM Plex Mono', 'Menlo', monospace;
}
```

---

## Pixie Dust: The Particle System

### Concept

The "pixie dust" is Plasmate's signature visual texture. It represents **ionized particles** - the fundamental unit of plasma. When matter transitions to the plasma state, atoms lose electrons and become a field of charged particles. This is what we visualize.

### Implementation

**Dot Field** - A sparse, randomized grid of small circles (2-4px) at varying opacities (0.08-0.25) placed on dark backgrounds. The dots are always Ember (`#E8853A`) or Plasma White (`#F0EDE8`).

```
Rules:
- Dots are circular, never other shapes
- Size range: 2px to 4px diameter
- Opacity range: 0.08 to 0.25 (barely visible to faintly present)
- Density: sparse. Think starfield, not confetti.
- Distribution: pseudo-random with clustering bias toward edges
- Never animate on marketing pages (respect user motion preferences)
- On interactive surfaces (demos, playground): subtle drift at 0.1px/frame
```

**Emission Lines** - Thin horizontal or vertical lines (1px, Ember at 0.12 opacity) that suggest spectral emission lines from plasma discharge. Used as section dividers or background texture on hero sections.

```
Rules:
- Always 1px width
- Always Ember color at low opacity (0.08-0.15)
- Horizontal preferred; vertical only in sidebar contexts
- Spacing: irregular, clustered in groups of 2-4 with large gaps between
- Never more than 5-7 lines visible at once
```

**Ionization Border** - A subtle particle scatter effect along the edges of cards or containers. Small dots (1-2px) that appear to be dispersing from the edge of a surface. Suggests matter transitioning states at the boundary.

```
Rules:
- Only on key interactive cards (not every card)
- Particles scatter outward from the card edge
- 8-15 particles per edge
- Fade to 0 opacity within 20px of the card edge
- Use on: hero cards, feature highlights, the "SOM output" preview
```

### Where to Use Pixie Dust

| Context | Technique | Density |
|---------|-----------|---------|
| Hero background (dark) | Dot field + emission lines | Medium |
| Documentation | None | None - docs are clean |
| Code examples | Subtle dot field behind code block | Very low |
| Marketing cards | Ionization border on hover | Low |
| CLI / terminal screenshots | Emission lines as scan lines | Very low |
| 404 / empty states | Dense dot field (the void) | High |
| Loading states | Converging dot animation | Medium |

---

## Logo

### The Mark

The Plasmate mark is a **stylized "P"** whose vertical stroke disperses into particles at the top - matter transitioning into plasma. The letter form is geometric, drawn from Space Grotesk's proportions but simplified for icon use.

The dispersion effect at the top of the P is the brand's pixie dust rendered at mark scale: 5-8 small circles breaking away from the letterform, decreasing in size and opacity as they scatter upward and rightward.

### Lockup Variants

| Variant | Usage |
|---------|-------|
| **Full lockup** | Mark + "PLASMATE" wordmark (Space Grotesk SemiBold, uppercase, 0.06em tracking) |
| **Compact** | Mark + "plasmate" wordmark (Space Grotesk Medium, lowercase) |
| **Icon only** | The P mark alone. For favicons, app icons, social avatars. |

### Color Variants

| Context | Mark | Wordmark | Background |
|---------|------|----------|------------|
| Primary (dark bg) | Ember | Plasma White | Core Black / Void |
| Primary (light bg) | Ember | Core Black | Plasma White |
| Monochrome (dark bg) | Plasma White | Plasma White | Core Black |
| Monochrome (light bg) | Core Black | Core Black | Plasma White |

### Clear Space

Minimum clear space around the full lockup: the width of the "P" mark on all sides. For the icon only: half the mark width.

### Don'ts

- Do not rotate the mark
- Do not apply gradients to the mark
- Do not change the dispersion direction (always up-right)
- Do not use the mark smaller than 16px (switch to a simplified version without particles below 16px)
- Do not place the mark on busy photography or patterned backgrounds
- Do not outline the mark
- Do not animate the mark dispersion without explicit brand team approval

---

## Voice and Tone

### Writing Principles

**Be direct.** "Plasmate compiles HTML into a Semantic Object Model." Not "Plasmate leverages cutting-edge AI to revolutionize how agents interact with the web."

**Be specific.** "10.4x compression on Wikipedia. 4ms per page." Not "blazing fast performance."

**Be honest about scope.** "v0.1 does not execute JavaScript dynamically" is better than silence. Open source trust is built on honesty.

**Use the metaphor sparingly.** The plasma/fourth-state metaphor is powerful in introductions and keynotes. In documentation, just be clear.

### Naming Conventions

| Thing | Name | Style |
|-------|------|-------|
| The product | Plasmate | Capital P, no "the" |
| The output format | SOM (Semantic Object Model) | Acronym, always capitalized |
| The protocol | AWP (Agent Web Protocol) | Acronym, always capitalized |
| The CLI commands | `plasmate fetch`, `plasmate serve` | Monospace, lowercase |
| The organization | Plasmate Labs | Two words, both capitalized |
| Element IDs | `e_a1b2c3d4e5f6` | Monospace, always with `e_` prefix |
| Session IDs | `s_abc123` | Monospace, always with `s_` prefix |

### Taglines

**Primary:** *The browser engine for agents.*

**Technical:** *HTML in. Semantic Object Model out.*

**Philosophical:** *The fourth state of the web.*

**For developers:** *Drop-in Puppeteer replacement. 10x less tokens.*

---

## Application Examples

### GitHub README Header

```
[P mark, Ember on transparent]

# plasmate

The browser engine for agents.

HTML in. Semantic Object Model out.
```

Dark background. Mark left-aligned. Title in Space Grotesk SemiBold. Subtitle in IBM Plex Sans. Dot field texture at very low opacity behind the header area.

### Documentation Site (docs.plasmate.app)

- Sidebar: Core Black background, Plasma White text, Ember for active section
- Content: Plasma White background, Core Black text
- Code blocks: Void background, syntax highlighted with Arc for keywords and Ember for strings
- Navigation: Space Grotesk Medium, 14px
- Body: IBM Plex Sans Regular, 16px

### Terminal / CLI Output

- Background: Core Black
- Standard output: Plasma White
- Success messages: Ember
- URLs and paths: Arc
- Metrics and numbers: Corona
- Errors: `#D4443A` (a warm red, not pure red)
- Version strings: IBM Plex Mono

### Social / Open Graph Cards

- Background: Core Black
- Mark: Ember, centered or left-third
- Title: Space Grotesk SemiBold, Plasma White
- Subtitle: IBM Plex Sans, Ash
- Dot field: subtle, 0.08-0.12 opacity, Ember particles

---

## File Formats and Assets

### Deliverables Needed

- [ ] SVG mark (all color variants)
- [ ] SVG full lockup (all color variants)
- [ ] PNG mark at 16, 32, 64, 128, 256, 512, 1024px
- [ ] Favicon set (ICO + PNG)
- [ ] Open Graph image template (1200x630)
- [ ] Twitter card template (1200x600)
- [ ] GitHub social preview (1280x640)
- [ ] Dot field SVG pattern tile (repeatable)
- [ ] Emission line SVG pattern tile
- [ ] Color swatch file (ASE, CLR)

### Repository Location

All brand assets live in the `plasmate` repo under `brand/`:

```
brand/
  mark/
    plasmate-mark-ember.svg
    plasmate-mark-white.svg
    plasmate-mark-black.svg
  lockup/
    plasmate-lockup-dark.svg
    plasmate-lockup-light.svg
  icons/
    favicon.ico
    icon-16.png ... icon-1024.png
  social/
    og-default.png
    twitter-card.png
    github-social.png
  patterns/
    dot-field.svg
    emission-lines.svg
  colors/
    plasmate-palette.ase
```

---

## Quick Reference Card

```
COLORS
  Ember:        #E8853A    Primary brand
  Arc:          #3D8FD4    Links, interactive
  Corona:       #F5C842    Highlights (rare)
  Core Black:   #0D0D0D    Text, dark bg
  Plasma White: #F0EDE8    Light bg
  Ion:          #D4C5A0    Cards, subtle bg
  Ash:          #6B6560    Muted text

FONTS
  Display/UI:   Space Grotesk (SemiBold, Medium)
  Body:         IBM Plex Sans (Regular, Medium)
  Code:         IBM Plex Mono (Regular, Medium)

RULES
  No gradients. No purple. No Inter.
  Ember is the signature color.
  Pixie dust = sparse particle dots, never confetti.
  Direct voice. Specific claims. Honest scope.

TAGLINE
  The browser engine for agents.
```

---

*Created March 16, 2026. Plasmate Labs. Apache 2.0.*
