# Plasmate Go SDK

Go SDK for the [Plasmate](https://plasmate.dev) agent-native headless browser.

## Install

```bash
go get github.com/nickel-org/plasmate-go
```

Requires the `plasmate` binary in your PATH.

## Quick Start

```go
package main

import (
    "fmt"
    "log"

    plasmate "github.com/nickel-org/plasmate-go"
)

func main() {
    client := plasmate.NewClient()
    defer client.Close()

    // One-shot: fetch a page as SOM
    som, err := client.FetchPage("https://example.com")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(som.Title, len(som.Regions), "regions")

    // Query helpers
    links := plasmate.FindByTag(som, "link")
    fmt.Printf("Found %d links\n", len(links))

    interactive := plasmate.FindInteractive(som)
    fmt.Printf("Found %d interactive elements\n", len(interactive))

    plan := plasmate.GetActionPlan(som)
    fmt.Printf("Found %d action targets\n", len(plan))
    for _, item := range plan {
        fmt.Println(item.ID, item.CacheKey)
        if !item.Enabled {
            fmt.Printf("Skipping %s: %s\n", item.ID, *item.BlockedReason)
        }
    }

    fmt.Printf("~%d tokens\n", plasmate.TokenEstimate(som))
}
```

## Interactive Sessions

```go
session, err := client.OpenPage("https://news.ycombinator.com")
if err != nil {
    log.Fatal(err)
}
defer client.ClosePage(session.SessionID)

// Click an element
updated, err := client.Click(session.SessionID, "e5")
if err != nil {
    log.Fatal(err)
}
fmt.Println("Page after click:", updated.Title)

// Evaluate JavaScript
title, err := client.Evaluate(session.SessionID, "document.title")
if err != nil {
    log.Fatal(err)
}
fmt.Println("Title:", title)
```

## SOM Parsing

Parse SOM JSON directly without the client:

```go
som, err := plasmate.Parse(jsonBytes)
if err != nil {
    log.Fatal(err)
}

// Find elements
el := plasmate.FindByID(som, "e3")
navs := plasmate.FindByRole(som, "navigation")
buttons := plasmate.FindByTag(som, "button")
results := plasmate.FindByText(som, "sign in")
clickable := plasmate.FindByAction(som, "click")
required := plasmate.FindByHint(som, "required")
plan := plasmate.GetActionPlan(som)
all := plasmate.FlatElements(som)
```

## API Reference

### Client

| Method | Description |
|--------|-------------|
| `NewClient(opts...)` | Create client (spawns `plasmate mcp` on first call) |
| `FetchPage(url)` | Fetch page as SOM |
| `FetchPageWithOptions(url, opts)` | Fetch with budget/JS options |
| `ExtractText(url)` | Fetch page as plain text |
| `OpenPage(url)` | Open interactive session |
| `Evaluate(sessionID, expr)` | Run JS in page context |
| `Click(sessionID, elementID)` | Click element, get updated SOM |
| `ClosePage(sessionID)` | Close session |
| `Close()` | Shut down subprocess |

### Query Helpers

| Function | Description |
|----------|-------------|
| `Parse(data)` | Parse SOM JSON bytes |
| `FindByRole(som, role)` | Find regions by role |
| `FindByID(som, id)` | Find element by stable SOM ID |
| `FindByHTMLID(som, htmlID)` | Find element by original HTML ID |
| `FindByTag(som, tag)` | Find elements by role/tag |
| `FindInteractive(som)` | Find elements with actions |
| `FindByText(som, text)` | Case-insensitive text search |
| `FindByAction(som, action)` | Find elements exposing an action |
| `FindByHint(som, hint)` | Find elements tagged with a semantic hint |
| `GetActionPlan(som)` | Return compact action targets with cache keys, availability, original `html_id` bridge cues, link target/rel/download cues, graphical submitter alt/src cues, form submission context, submitter override cues, select selected_values/size context, text-entry/input-affordance cues, popover/command relationship cues, title/label/description ID relationships, ARIA live-region cues, ARIA owns/flowto/details relationships, ARIA widget affordances, range constraints, orientation/sort/value state, and set-position cues for agents |
| `EnabledActionPlan(som)` | Return compact action targets whose `enabled` field is true |
| `GetActionPlanCacheKey(item)` | Return a deterministic key for caching or comparing an action target |
| `GetActionPlanIndex(som, enabledOnly...)` | Index compact action targets by `ByID`, `ByCacheKey`, and `ByHTMLID` for replay validation |
| `FindActionTargetByCacheKey(som, cacheKey)` | Resolve a cached action target from the current SOM action plan |
| `FindActionTargetByID(som, id)` | Resolve an action target by stable SOM id |
| `FindActionTargetByHTMLID(som, htmlID)` | Resolve an action target by original HTML id |
| `FlatElements(som)` | Flatten all elements, including shadow roots |
| `TokenEstimate(som)` | Estimate token count |

The Go types include current SOM actionability fields such as
`attrs.description`, `attrs.name`, `attrs.accept`, `attrs.capture`,
`attrs.multiple`, `attrs.autocomplete`, `attrs.inputmode`,
`attrs.enterkeyhint`, `attrs.autocapitalize`, `attrs.dirname`, `attrs.dir`,
`attrs.lang`, `attrs.spellcheck`, `attrs.aria_label`,
`attrs.aria_description`, `attrs.form`, `attrs.list`, `attrs.popovertarget`,
`attrs.popovertargetaction`, `attrs.commandfor`, `attrs.command`,
`attrs.button_type`, `attrs.formaction`, `attrs.formmethod`,
`attrs.formenctype`, `attrs.formtarget`, `attrs.formnovalidate`, `attrs.accesskey`,
`attrs.aria`, iframe attrs, form validation constraints, and `shadow` roots so
Go agents receive the same
contract as the Python and Node parser packages. Action-plan items include
`Enabled`, `BlockedReason`, `PopoverTarget`, `CommandFor`, `KeyShortcuts`, and
`RoleDescription`, `Owns`, `FlowTo`, `Details`, `Multiline`, and
`MultiSelectable` so agents can skip
known-unavailable controls and understand popover, command, keyboard,
custom-role, ARIA relationship, and widget affordance cues before acting.
They also include deterministic `CacheKey` values plus `Autocomplete`,
`AutoCapitalize`, `DirName`, `Spellcheck`, `AriaPlaceholder`, `MinLength`,
`MaxLength`, `Pattern`, and `Invalid` cues for local action-plan caches,
prompt dedupe, and trace correlation.
