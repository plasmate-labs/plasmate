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
| `FlatElements(som)` | Flatten all elements |
| `TokenEstimate(som)` | Estimate token count |
