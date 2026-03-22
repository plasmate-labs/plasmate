package plasmate

import (
	"encoding/json"
	"testing"
)

var testSOM = `{
  "som_version": "1.0",
  "url": "https://example.com",
  "title": "Example Page",
  "lang": "en",
  "regions": [
    {
      "id": "r_navigation",
      "role": "navigation",
      "label": "Main nav",
      "elements": [
        {
          "id": "e1",
          "role": "link",
          "text": "Home",
          "actions": ["click"],
          "attrs": {"href": "/"}
        },
        {
          "id": "e2",
          "role": "link",
          "text": "About Us",
          "actions": ["click"],
          "attrs": {"href": "/about"}
        }
      ]
    },
    {
      "id": "r_main",
      "role": "main",
      "elements": [
        {
          "id": "e3",
          "role": "heading",
          "text": "Welcome",
          "attrs": {"level": 1}
        },
        {
          "id": "e4",
          "role": "paragraph",
          "text": "Hello world, this is a test page."
        },
        {
          "id": "e5",
          "role": "text_input",
          "label": "Search",
          "actions": ["click", "type", "clear"],
          "attrs": {"input_type": "text", "placeholder": "Search..."}
        },
        {
          "id": "e6",
          "role": "button",
          "text": "Submit",
          "actions": ["click"]
        },
        {
          "id": "e7",
          "role": "image",
          "attrs": {"src": "/hero.png", "alt": "Hero image"}
        }
      ]
    },
    {
      "id": "r_footer",
      "role": "footer",
      "elements": [
        {
          "id": "e8",
          "role": "paragraph",
          "text": "Copyright 2025"
        }
      ]
    }
  ],
  "meta": {
    "html_bytes": 4096,
    "som_bytes": 1200,
    "element_count": 8,
    "interactive_count": 4
  },
  "structured_data": {
    "open_graph": {"og:title": "Example Page"},
    "meta": {"description": "A test page"}
  }
}`

func mustParse(t *testing.T) *Som {
	t.Helper()
	som, err := Parse([]byte(testSOM))
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	return som
}

func TestParse(t *testing.T) {
	som := mustParse(t)

	if som.SOMVersion != "1.0" {
		t.Errorf("SOMVersion = %q, want %q", som.SOMVersion, "1.0")
	}
	if som.URL != "https://example.com" {
		t.Errorf("URL = %q, want %q", som.URL, "https://example.com")
	}
	if som.Title != "Example Page" {
		t.Errorf("Title = %q, want %q", som.Title, "Example Page")
	}
	if som.Lang != "en" {
		t.Errorf("Lang = %q, want %q", som.Lang, "en")
	}
	if len(som.Regions) != 3 {
		t.Fatalf("len(Regions) = %d, want 3", len(som.Regions))
	}
	if som.Meta.HTMLBytes != 4096 {
		t.Errorf("Meta.HTMLBytes = %d, want 4096", som.Meta.HTMLBytes)
	}
	if som.Meta.SOMBytes != 1200 {
		t.Errorf("Meta.SOMBytes = %d, want 1200", som.Meta.SOMBytes)
	}
	if som.Meta.ElementCount != 8 {
		t.Errorf("Meta.ElementCount = %d, want 8", som.Meta.ElementCount)
	}
	if som.Meta.InteractiveCount != 4 {
		t.Errorf("Meta.InteractiveCount = %d, want 4", som.Meta.InteractiveCount)
	}
}

func TestParseStructuredData(t *testing.T) {
	som := mustParse(t)

	if som.StructuredData == nil {
		t.Fatal("StructuredData is nil")
	}
	if som.StructuredData.OpenGraph["og:title"] != "Example Page" {
		t.Errorf("OpenGraph og:title = %q, want %q", som.StructuredData.OpenGraph["og:title"], "Example Page")
	}
	if som.StructuredData.Meta["description"] != "A test page" {
		t.Errorf("Meta description = %q, want %q", som.StructuredData.Meta["description"], "A test page")
	}
}

func TestParseRegionFields(t *testing.T) {
	som := mustParse(t)

	nav := som.Regions[0]
	if nav.ID != "r_navigation" {
		t.Errorf("Region ID = %q, want %q", nav.ID, "r_navigation")
	}
	if nav.Role != "navigation" {
		t.Errorf("Region Role = %q, want %q", nav.Role, "navigation")
	}
	if nav.Label == nil || *nav.Label != "Main nav" {
		t.Errorf("Region Label = %v, want %q", nav.Label, "Main nav")
	}
	if len(nav.Elements) != 2 {
		t.Fatalf("len(Elements) = %d, want 2", len(nav.Elements))
	}
}

func TestParseElementAttrs(t *testing.T) {
	som := mustParse(t)

	// Check heading level
	heading := som.Regions[1].Elements[0]
	if heading.Attrs == nil || heading.Attrs.Level == nil || *heading.Attrs.Level != 1 {
		t.Errorf("heading level = %v, want 1", heading.Attrs)
	}

	// Check link href
	link := som.Regions[0].Elements[0]
	if link.Attrs == nil || link.Attrs.Href == nil || *link.Attrs.Href != "/" {
		t.Errorf("link href = %v, want /", link.Attrs)
	}

	// Check input attrs
	input := som.Regions[1].Elements[2]
	if input.Attrs == nil {
		t.Fatal("input attrs nil")
	}
	if input.Attrs.InputType == nil || *input.Attrs.InputType != "text" {
		t.Errorf("input_type = %v, want text", input.Attrs.InputType)
	}
	if input.Attrs.Placeholder == nil || *input.Attrs.Placeholder != "Search..." {
		t.Errorf("placeholder = %v, want Search...", input.Attrs.Placeholder)
	}
}

func TestParseInvalidJSON(t *testing.T) {
	_, err := Parse([]byte(`{invalid`))
	if err == nil {
		t.Error("expected error for invalid JSON")
	}
}

func TestFindByRole(t *testing.T) {
	som := mustParse(t)

	navs := FindByRole(som, "navigation")
	if len(navs) != 1 {
		t.Fatalf("FindByRole(navigation) = %d, want 1", len(navs))
	}
	if navs[0].ID != "r_navigation" {
		t.Errorf("ID = %q, want r_navigation", navs[0].ID)
	}

	empty := FindByRole(som, "dialog")
	if len(empty) != 0 {
		t.Errorf("FindByRole(dialog) = %d, want 0", len(empty))
	}
}

func TestFindByID(t *testing.T) {
	som := mustParse(t)

	el := FindByID(som, "e5")
	if el == nil {
		t.Fatal("FindByID(e5) returned nil")
	}
	if el.Role != "text_input" {
		t.Errorf("Role = %q, want text_input", el.Role)
	}

	if FindByID(som, "e999") != nil {
		t.Error("FindByID(e999) should return nil")
	}
}

func TestFindByTag(t *testing.T) {
	som := mustParse(t)

	links := FindByTag(som, "link")
	if len(links) != 2 {
		t.Fatalf("FindByTag(link) = %d, want 2", len(links))
	}

	headings := FindByTag(som, "heading")
	if len(headings) != 1 {
		t.Fatalf("FindByTag(heading) = %d, want 1", len(headings))
	}
}

func TestFindInteractive(t *testing.T) {
	som := mustParse(t)

	interactive := FindInteractive(som)
	if len(interactive) != 4 {
		t.Fatalf("FindInteractive = %d, want 4", len(interactive))
	}

	// Verify all have actions
	for _, el := range interactive {
		if len(el.Actions) == 0 {
			t.Errorf("element %s has no actions", el.ID)
		}
	}
}

func TestFindByText(t *testing.T) {
	som := mustParse(t)

	results := FindByText(som, "hello")
	if len(results) != 1 {
		t.Fatalf("FindByText(hello) = %d, want 1", len(results))
	}
	if results[0].ID != "e4" {
		t.Errorf("ID = %q, want e4", results[0].ID)
	}

	// Case insensitive
	results = FindByText(som, "WELCOME")
	if len(results) != 1 {
		t.Fatalf("FindByText(WELCOME) = %d, want 1", len(results))
	}

	empty := FindByText(som, "nonexistent")
	if len(empty) != 0 {
		t.Errorf("FindByText(nonexistent) = %d, want 0", len(empty))
	}
}

func TestFlatElements(t *testing.T) {
	som := mustParse(t)

	flat := FlatElements(som)
	if len(flat) != 8 {
		t.Fatalf("FlatElements = %d, want 8", len(flat))
	}
}

func TestTokenEstimate(t *testing.T) {
	som := mustParse(t)

	est := TokenEstimate(som)
	if est != 300 { // 1200 / 4
		t.Errorf("TokenEstimate = %d, want 300", est)
	}
}

func TestTokenEstimateZero(t *testing.T) {
	som := &Som{}
	if TokenEstimate(som) != 0 {
		t.Errorf("TokenEstimate of empty SOM should be 0")
	}
}

func TestRoundTrip(t *testing.T) {
	som := mustParse(t)

	data, err := json.Marshal(som)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	som2, err := Parse(data)
	if err != nil {
		t.Fatalf("Parse round-trip failed: %v", err)
	}
	if som2.Title != som.Title {
		t.Errorf("Title mismatch after round-trip")
	}
	if len(som2.Regions) != len(som.Regions) {
		t.Errorf("Regions count mismatch after round-trip")
	}
	if som2.Meta.ElementCount != som.Meta.ElementCount {
		t.Errorf("ElementCount mismatch after round-trip")
	}
}
