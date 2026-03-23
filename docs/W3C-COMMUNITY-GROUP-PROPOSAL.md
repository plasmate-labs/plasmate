# W3C Community Group Proposal: Semantic Object Model (SOM)

## Proposed Group Name
**Web Content for AI Agents Community Group**

## Mission Statement
Develop and maintain the Semantic Object Model (SOM) specification - a standardized JSON format for representing web page content in a form optimized for consumption by AI agents and language models, reducing token costs by 90%+ while preserving semantic meaning and interactivity.

## Background
AI agents increasingly need to read and interact with web pages. The current approach - sending raw HTML or DOM snapshots to language models - is wasteful and lossy:

- A typical web page contains 300-500KB of HTML, of which 80-95% is presentation markup (CSS classes, layout divs, script tags, tracking pixels)
- This noise costs real money at $2.50-$10 per million input tokens
- The DOM is a rendering tree optimized for visual browsers, not a meaning tree optimized for machine understanding
- No standard exists for representing web content to AI agents

SOM addresses this by compiling HTML into a structured JSON representation that preserves content, structure, and interactivity while discarding presentation markup. Benchmarks across 49 real-world websites show a median 10.5x compression ratio and 94% token cost savings.

## Scope of Work
1. **SOM Specification** - Define and iterate on the SOM JSON schema, including:
   - Region roles (navigation, content, form, header, footer, aside, dialog)
   - Element roles (link, button, heading, paragraph, input, select, table, image, etc.)
   - Action annotations (click, type, clear, select, toggle)
   - Attribute preservation rules (which HTML attributes carry semantic meaning)
   - Structured data extraction (JSON-LD, OpenGraph, meta tags)
   - Compression and token estimation guidelines

2. **Conformance Test Suite** - Develop a reproducible benchmark suite that:
   - Tests SOM output against a reference set of URLs
   - Validates schema conformance
   - Measures compression ratios and token savings
   - Ensures interoperability between implementations

3. **Best Practices** - Document guidelines for:
   - When to use SOM vs raw HTML vs screenshots
   - Handling dynamic/SPA content
   - Accessibility considerations
   - Privacy and content filtering

## Deliverables
- SOM Specification (Community Group Report)
- SOM JSON Schema
- Conformance test suite
- Reference implementations in Rust, JavaScript/TypeScript, and Python
- Integration guides for popular agent frameworks

## Existing Work
- **SOM Spec v1.0**: Published at https://plasmate.app/docs/som-spec
- **Reference implementation**: Plasmate (Apache 2.0) - https://github.com/plasmate-labs/plasmate
- **Standalone parsers**: npm (`som-parser`) and PyPI (`som-parser`) packages
- **Benchmarks**: 49-URL cost analysis with reproducible methodology
- **JSON Schema**: Available in the Plasmate repository

## Proposed Chairs
- David Hurley (DBH Ventures / Plasmate Labs)

## Communication
- GitHub repository for specification work and issues
- Monthly virtual meetings
- Public mailing list via W3C infrastructure

## Patent Policy
This group will operate under the W3C Community Contributor License Agreement (CLA).

## Participation
Open to all. W3C Membership is not required. Participants must have a W3C account and agree to the CLA.

## Duration
Ongoing, with major specification milestones every 6 months.

---

## How to Submit

### Steps to create the Community Group:
1. Go to https://www.w3.org/community/groups/propose_cg/
2. Sign in with a W3C account (create one at https://www.w3.org/accounts/request if needed)
3. Fill in:
   - **Group name**: Web Content for AI Agents Community Group
   - **Shortname**: web-content-ai (used in URLs)
   - **Mission**: (copy from above)
   - **Proposed charter**: Link to this document or the SOM spec
4. Submit the proposal
5. Recruit at least 5 participants to join (W3C accounts required, membership not required)
6. W3C staff reviews and approves (typically 1-2 weeks)

### After Approval:
- Set up the GitHub repository under the W3C org
- Publish the SOM spec as a Community Group Draft
- Schedule the first meeting
- Invite agent framework maintainers (Browser Use, LangChain, Playwright, Puppeteer communities)
- Submit to relevant W3C Working Groups for awareness (Web Applications WG, Accessibility WG)

### Key Contacts at W3C:
- Community Group program: https://www.w3.org/community/
- Staff contact for new groups: team-community-process@w3.org
