package plasmate

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func TestFetchPageArgumentsIncludeSelector(t *testing.T) {
	budget := 512
	javascript := false
	selector := "interactive"

	got := fetchPageArguments("fixture", FetchPageOptions{
		Budget:     &budget,
		JavaScript: &javascript,
		Selector:   &selector,
	})
	want := map[string]interface{}{
		"url":        "fixture",
		"budget":     512,
		"javascript": false,
		"selector":   "interactive",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("fetchPage arguments = %#v, want %#v", got, want)
	}
}

func TestFetchPageArgumentsOmitNilSelector(t *testing.T) {
	got := fetchPageArguments("fixture", FetchPageOptions{})
	if _, ok := got["selector"]; ok {
		t.Fatalf("fetchPage arguments unexpectedly included nil selector: %#v", got)
	}
}

func TestExtractTextArgumentsIncludeOptions(t *testing.T) {
	maxChars := 1200
	selector := "main"

	got := extractTextArguments("fixture", ExtractTextOptions{
		MaxChars: &maxChars,
		Selector: &selector,
	})
	want := map[string]interface{}{
		"url":       "fixture",
		"max_chars": 1200,
		"selector":  "main",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("extract_text arguments = %#v, want %#v", got, want)
	}
}

func TestExtractTextArgumentsOmitNilOptions(t *testing.T) {
	got := extractTextArguments("fixture", ExtractTextOptions{})
	if len(got) != 1 || got["url"] != "fixture" {
		t.Fatalf("extract_text arguments = %#v, want only fixture URL", got)
	}
}

func TestEmptyToolErrorKeepsBoundedMessage(t *testing.T) {
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	if err := os.WriteFile(fixture, []byte(`#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":""}],"isError":true}}'
      ;;
  esac
done
`), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	_, err := client.FetchPage("fixture")
	if err == nil || err.Error() != "unknown error" {
		t.Fatalf("FetchPage error = %v, want bounded unknown error", err)
	}
}

func TestToolErrorDiagnosticIsBounded(t *testing.T) {
	diagnostic := strings.Repeat("x", 5000)
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	script := fmt.Sprintf(`#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"%s"}],"isError":true}}'
      ;;
  esac
done
`, diagnostic)
	if err := os.WriteFile(fixture, []byte(script), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	_, err := client.FetchPage("fixture")
	if err == nil {
		t.Fatal("FetchPage returned nil error for a failed tool result")
	}
	got := []rune(err.Error())
	if len(got) != 200 || got[len(got)-1] != '…' {
		t.Fatalf("FetchPage error length/marker = %d/%q, want 200/ellipsis", len(got), got[len(got)-1])
	}
}
