package plasmate

import (
	"reflect"
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
