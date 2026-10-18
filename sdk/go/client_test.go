package plasmate

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func TestPageSessionDecodesCacheRestored(t *testing.T) {
	var session PageSession
	if err := json.Unmarshal([]byte(`{"session_id":"s1","cache_restored":true}`), &session); err != nil {
		t.Fatalf("unmarshal page session: %v", err)
	}
	if !session.CacheRestored {
		t.Fatalf("cache_restored = false, want true")
	}
}

func TestOpenPagePayloadDecodesFlatSOM(t *testing.T) {
	raw := []byte(`{"session_id":"s1","title":"Example","url":"https://example.com","som_version":"1.0","regions":[{"id":"main","role":"main","elements":[]}],"meta":{"html_bytes":42,"som_bytes":84,"element_count":1,"interactive_count":0}}`)
	session, err := decodePageSession(raw)
	if err != nil {
		t.Fatalf("unmarshal open_page payload: %v", err)
	}
	if session.SessionID != "s1" || session.Som.Title != "Example" {
		t.Fatalf("decoded session = %#v, want session and flat SOM", session)
	}
	if len(session.Som.Regions) != 1 || session.Som.Meta.HTMLBytes != 42 {
		t.Fatalf("decoded SOM = %#v, want flat regions and metadata", session.Som)
	}
}

func TestOpenPagePayloadDecodesFlatSOMWithoutRegions(t *testing.T) {
	raw := []byte(`{"session_id":"s1","som_version":"1.0","meta":{"html_bytes":42}}`)
	session, err := decodePageSession(raw)
	if err != nil {
		t.Fatalf("unmarshal open_page payload: %v", err)
	}
	if session.Som.SOMVersion != "1.0" || session.Som.Meta.HTMLBytes != 42 {
		t.Fatalf("decoded SOM = %#v, want flat version and metadata", session.Som)
	}
}

func TestOpenPagePayloadMergesSparseFlatSOMWithNestedSOM(t *testing.T) {
	raw := []byte(`{"session_id":"s1","som":{"title":"Nested","regions":[{"id":"main","role":"main","elements":[]}]},"meta":{"html_bytes":42}}`)
	session, err := decodePageSession(raw)
	if err != nil {
		t.Fatalf("unmarshal open_page payload: %v", err)
	}
	if session.Som.Title != "Nested" || len(session.Som.Regions) != 1 || session.Som.Meta.HTMLBytes != 42 {
		t.Fatalf("decoded SOM = %#v, want nested fields plus flat metadata", session.Som)
	}
}

func TestNavigateToPayloadDecodesFlatSOM(t *testing.T) {
	raw := []byte(`{"session_id":"s1","title":"Next","url":"https://example.com/next","cache_restored":true,"regions":[{"id":"main","role":"main","elements":[]}]}`)
	session, err := decodePageSession(raw)
	if err != nil {
		t.Fatalf("unmarshal navigate_to payload: %v", err)
	}
	if session.SessionID != "s1" || session.Som.Title != "Next" || session.Som.URL != "https://example.com/next" {
		t.Fatalf("decoded session = %#v, want navigation session and flat SOM", session)
	}
	if !session.CacheRestored || len(session.Som.Regions) != 1 {
		t.Fatalf("decoded navigation SOM = %#v, want restored session with one region", session)
	}
}

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

func TestOpenPageArgumentsIncludeOptions(t *testing.T) {
	selector := "main"
	trace := true

	got := openPageArguments("fixture", OpenPageOptions{
		Selector: &selector,
		Trace:    &trace,
	})
	want := map[string]interface{}{
		"url":      "fixture",
		"selector": "main",
		"trace":    true,
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("open_page arguments = %#v, want %#v", got, want)
	}
}

func TestOpenPageArgumentsOmitNilOptions(t *testing.T) {
	got := openPageArguments("fixture", OpenPageOptions{})
	if len(got) != 1 || got["url"] != "fixture" {
		t.Fatalf("open_page arguments = %#v, want only fixture URL", got)
	}
}

func TestTypeTextArgumentsIncludeAppend(t *testing.T) {
	appendText := true

	got := typeTextArguments("s1", "e5", " world", TypeTextOptions{Append: &appendText})
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e5",
		"text":       " world",
		"append":     true,
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("type_text arguments = %#v, want %#v", got, want)
	}
}

func TestTypeTextArgumentsOmitNilAppend(t *testing.T) {
	got := typeTextArguments("s1", "e5", "hello", TypeTextOptions{})
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e5",
		"text":       "hello",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("type_text arguments = %#v, want %#v", got, want)
	}
}

func TestClearArguments(t *testing.T) {
	got := clearArguments("s1", "e5")
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e5",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("clear arguments = %#v, want %#v", got, want)
	}
}

func TestSelectOptionArguments(t *testing.T) {
	got := selectOptionArguments("s1", "e5", "premium")
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e5",
		"value":      "premium",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("select_option arguments = %#v, want %#v", got, want)
	}
}

func TestToggleArguments(t *testing.T) {
	got := toggleArguments("s1", "e5")
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e5",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("toggle arguments = %#v, want %#v", got, want)
	}
}

func TestScrollArgumentsIncludeOptions(t *testing.T) {
	pixels := 640
	elementID := "e5"

	got := scrollArguments("s1", ScrollOptions{
		Direction: "bottom",
		Pixels:    &pixels,
		ElementID: &elementID,
	})
	want := map[string]interface{}{
		"session_id": "s1",
		"direction":  "bottom",
		"pixels":     640,
		"element_id": "e5",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("scroll arguments = %#v, want %#v", got, want)
	}
}

func TestScrollArgumentsOmitDefaults(t *testing.T) {
	got := scrollArguments("s1", ScrollOptions{})
	want := map[string]interface{}{"session_id": "s1"}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("scroll arguments = %#v, want %#v", got, want)
	}
}

func TestScrollDirectionConstants(t *testing.T) {
	want := map[string]string{
		"down":   ScrollDown,
		"up":     ScrollUp,
		"top":    ScrollTop,
		"bottom": ScrollBottom,
		"home":   ScrollHome,
		"end":    ScrollEnd,
	}

	for name, got := range want {
		if got != name {
			t.Errorf("scroll direction %q = %q, want %q", name, got, name)
		}
	}
}

func TestScrollIntoViewArguments(t *testing.T) {
	elementID := "e9"
	got := scrollArguments("s1", ScrollOptions{ElementID: &elementID})
	want := map[string]interface{}{
		"session_id": "s1",
		"element_id": "e9",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("scroll into view arguments = %#v, want %#v", got, want)
	}
}

func TestExtractTextArgumentsIncludeOptions(t *testing.T) {
	maxChars := 1200
	javascript := false
	selector := "main"

	got := extractTextArguments("fixture", ExtractTextOptions{
		MaxChars:   &maxChars,
		JavaScript: &javascript,
		Selector:   &selector,
	})
	want := map[string]interface{}{
		"url":        "fixture",
		"max_chars":  1200,
		"javascript": false,
		"selector":   "main",
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

func TestExtractLinksArgumentsIncludeOptions(t *testing.T) {
	javascript := false
	maxLinks := 3
	selector := "main"

	got := extractLinksArguments("fixture", ExtractLinksOptions{
		JavaScript: &javascript,
		MaxLinks:   &maxLinks,
		Selector:   &selector,
	})
	want := map[string]interface{}{
		"url":        "fixture",
		"javascript": false,
		"max_links":  3,
		"selector":   "main",
	}

	if !reflect.DeepEqual(got, want) {
		t.Fatalf("extract_links arguments = %#v, want %#v", got, want)
	}
}

func TestExtractLinksArgumentsOmitNilOptions(t *testing.T) {
	got := extractLinksArguments("fixture", ExtractLinksOptions{})
	if len(got) != 1 || got["url"] != "fixture" {
		t.Fatalf("extract_links arguments = %#v, want only fixture URL", got)
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
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":""}],"isError":true}}'
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

func TestToolErrorUsesFirstNonEmptyContentText(t *testing.T) {
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	if err := os.WriteFile(fixture, []byte(`#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"image","text":""},{"type":"text","text":"diagnostic from second block"}],"isError":true}}'
      ;;
  esac
done
`), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	_, err := client.FetchPage("fixture")
	if err == nil || err.Error() != "diagnostic from second block" {
		t.Fatalf("FetchPage error = %v, want later text diagnostic", err)
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
      printf '%%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"%s"}],"isError":true}}'
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

func TestRPCIgnoresMismatchedResponses(t *testing.T) {
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	script := `#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":99,"result":{"protocolVersion":"wrong"}}'
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":98,"result":{"content":[]}}'
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"expected tool failure"}],"isError":true}}'
      ;;
  esac
done
`
	if err := os.WriteFile(fixture, []byte(script), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	_, err := client.FetchPage("fixture")
	if err == nil || err.Error() != "expected tool failure" {
		t.Fatalf("FetchPage error = %v, want mismatched responses ignored", err)
	}
}

func TestStructuredToolResultFallback(t *testing.T) {
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	script := `#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[],"structuredContent":{"result":{"title":"Structured page","regions":[]}}}}'
      ;;
  esac
done
`
	if err := os.WriteFile(fixture, []byte(script), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	som, err := client.FetchPage("fixture")
	if err != nil {
		t.Fatalf("FetchPage error = %v, want structured result decoded", err)
	}
	if som.Title != "Structured page" {
		t.Fatalf("FetchPage title = %q, want structured page", som.Title)
	}
}

func TestStructuredToolResultPreferredOverText(t *testing.T) {
	dir := t.TempDir()
	fixture := filepath.Join(dir, "mcp-fixture.sh")
	script := `#!/bin/sh
while IFS= read -r request; do
  case "$request" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"human-readable summary"}],"structuredContent":{"result":{"title":"Canonical page","regions":[]}}}}'
      ;;
  esac
done
`
	if err := os.WriteFile(fixture, []byte(script), 0o700); err != nil {
		t.Fatalf("write fixture: %v", err)
	}

	client := NewClient(WithBinary(fixture))
	defer client.Close()
	som, err := client.FetchPage("fixture")
	if err != nil {
		t.Fatalf("FetchPage error = %v, want structured result decoded", err)
	}
	if som.Title != "Canonical page" {
		t.Fatalf("FetchPage title = %q, want canonical structured page", som.Title)
	}
}
