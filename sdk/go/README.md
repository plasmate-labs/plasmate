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

    index := plasmate.GetActionPlanIndex(som, true)
    fmt.Printf("Enabled buttons: %d\n", len(index.ByRole["button"]))
    fmt.Printf("Enabled click targets: %d\n", len(index.ByAction["click"]))

    planSelect := plasmate.FindActionTargetByLabel(som, "Plan")
    if planSelect != nil {
        fmt.Println("Plan target:", planSelect.ID)
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
index := plasmate.GetActionPlanIndex(som, true)
buttons := plasmate.FindActionTargetsByRole(som, "button", true)
clicks := plasmate.FindActionTargetsByAction(som, "click", true)
matches := plasmate.FindActionTargetsByLabel(som, "billing", false, true)
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
| `FindByID(som, id)` | Find element by ID |
| `FindByTag(som, tag)` | Find elements by role/tag |
| `FindInteractive(som)` | Find elements with actions |
| `FindByText(som, text)` | Case-insensitive text search |
| `FindByAction(som, action)` | Find elements exposing an action |
| `FindByHint(som, hint)` | Find elements tagged with a semantic hint |
| `GetActionPlan(som)` | Return compact action targets with cache keys, availability, link target/rel/download cues, form submission context, submitter override cues, text-entry/input-affordance cues, popover/command relationship cues, ARIA live-region cues, ARIA owns/flowto/details relationships, ARIA widget affordances, range constraints, orientation/sort/value state, and set-position cues for agents |
| `GetActionPlanIndex(som, enabledOnly...)` | Index compact targets by replay ids and group them by role/action/label, including `ByLabelAll` for duplicate labels |
| `GetActionPlanCacheKey(item)` | Return a deterministic key for caching or comparing an action target |
| `FindActionTarget(som, value, by...)` | Resolve a replay id by SOM id, cache key, HTML id, test id, exact label, or auto lookup |
| `FindActionTargetByLabel(som, label)` | Resolve the first compact target with an exact accessible label |
| `FindActionTargetsByLabel(som, label, exact, enabledOnly...)` | Return compact targets whose label matches exactly or by substring |
| `FindActionTargetsByRole(som, role, enabledOnly...)` | Return compact action targets for one SOM role |
| `FindActionTargetsByAction(som, action, enabledOnly...)` | Return compact action targets exposing one action |
| `FlatElements(som)` | Flatten all elements, including shadow roots |
| `TokenEstimate(som)` | Estimate token count |

The Go types include current SOM actionability fields such as
`attrs.description`, `attrs.name`, `attrs.accept`, `attrs.capture`,
`attrs.multiple`, `attrs.autocomplete`, `attrs.inputmode`,
`attrs.enterkeyhint`, `attrs.autocapitalize`, `attrs.dirname`,
`attrs.spellcheck`, `attrs.form`, `attrs.list`, `attrs.popovertarget`,
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
`GetActionPlanIndex` also includes `ByRole`, `ByAction`, and `ByLabelAll`
buckets so Go workers can scope a replay plan to enabled click targets, all
text inputs, or every target sharing an accessible label without scanning the
full SOM.
