# Go SDK

The official Go SDK for Plasmate, with typed structs and SOM query functions.

## Installation

```sh
go get github.com/nickel-org/plasmate-go
```

Requires Go 1.21+ and the `plasmate` binary on your PATH.

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

    som, err := client.FetchPage("https://example.com")
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println(som.Title)
    fmt.Printf("Regions: %d, Elements: %d\n", len(som.Regions), som.Meta.ElementCount)
}
```

## Go Structs

### `Som`

```go
type Som struct {
    SOMVersion     string          `json:"som_version"`
    URL            string          `json:"url"`
    Title          string          `json:"title"`
    Lang           string          `json:"lang"`
    Regions        []Region        `json:"regions"`
    Meta           SomMeta         `json:"meta"`
    StructuredData *StructuredData `json:"structured_data,omitempty"`
}
```

### `Region`

```go
type Region struct {
    ID       string    `json:"id"`
    Role     string    `json:"role"`     // navigation, main, aside, header, footer, form, dialog, content
    Label    *string   `json:"label,omitempty"`
    Action   *string   `json:"action,omitempty"`
    Method   *string   `json:"method,omitempty"`
    Elements []Element `json:"elements"`
}
```

### `Element`

```go
type Element struct {
    ID       string        `json:"id"`
    Role     string        `json:"role"`     // link, button, text_input, textarea, select, checkbox, radio, heading, image, list, table, paragraph, section, separator
    Text     *string       `json:"text,omitempty"`
    Label    *string       `json:"label,omitempty"`
    Actions  []string      `json:"actions,omitempty"`
    Attrs    *ElementAttrs `json:"attrs,omitempty"`
    Children []Element     `json:"children,omitempty"`
    Hints    []string      `json:"hints,omitempty"`
}
```

### `SomMeta`

```go
type SomMeta struct {
    HTMLBytes        int `json:"html_bytes"`
    SOMBytes         int `json:"som_bytes"`
    ElementCount     int `json:"element_count"`
    InteractiveCount int `json:"interactive_count"`
}
```

## Query Functions

### `Parse(data)`

Parse raw JSON bytes into a `Som` struct.

```go
som, err := plasmate.Parse(jsonBytes)
```

### `FindByRole(som, role)`

Find all regions matching a given role.

```go
navRegions := plasmate.FindByRole(som, "navigation")
```

### `FindByID(som, id)`

Find a single element by its stable ID. Returns `nil` if not found.

```go
el := plasmate.FindByID(som, "login-btn")
if el != nil {
    fmt.Println(*el.Text)
}
```

### `FindByTag(som, tag)`

Find elements matching a tag/role string.

```go
links := plasmate.FindByTag(som, "link")
```

### `FindInteractive(som)`

Return all interactive elements.

```go
interactive := plasmate.FindInteractive(som)
fmt.Printf("%d interactive elements\n", len(interactive))
```

### `FindByText(som, text)`

Find elements whose text content contains the given string (case-insensitive).

```go
matches := plasmate.FindByText(som, "Sign in")
```

### `FlatElements(som)`

Flatten all elements across all regions into a single slice.

```go
all := plasmate.FlatElements(som)
```

### `TokenEstimate(som)`

Estimate the LLM token count for the SOM (heuristic: SOMBytes / 4).

```go
tokens := plasmate.TokenEstimate(som)
fmt.Printf("~%d tokens\n", tokens)
```

## Client Usage

### Creating a Client

```go
// Default options
client := plasmate.NewClient()

// Custom binary path
client := plasmate.NewClient(plasmate.WithBinary("/usr/local/bin/plasmate"))
```

### Stateless Methods

```go
// Fetch a page and return SOM
som, err := client.FetchPage("https://example.com")

// Fetch with options
som, err := client.FetchPageWithOptions("https://example.com", plasmate.FetchOptions{
    Budget:     8000,
    JavaScript: true,
})

// Extract plain text
text, err := client.ExtractText("https://example.com")
```

### Stateful Sessions

```go
// Open a persistent page session
session, err := client.OpenPage("https://example.com")
// session.SessionID, session.Som

// Execute JavaScript
result, err := client.Evaluate(session.SessionID, "document.title")

// Click an element and get updated SOM
updatedSom, err := client.Click(session.SessionID, "login-btn")

// Close the session
err = client.ClosePage(session.SessionID)
```

### Cleanup

```go
err := client.Close()
```

The client is thread-safe with internal mutex protection. It communicates with the `plasmate mcp` subprocess over JSON-RPC 2.0 on stdio.
