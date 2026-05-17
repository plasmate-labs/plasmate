package plasmate

import (
	"encoding/json"
	"os"
	"reflect"
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
        },
        {
          "id": "e9",
          "role": "section",
          "text": "Search widget",
          "shadow": {
            "mode": "open",
            "elements": [
              {
                "id": "e10",
                "role": "button",
                "html_id": "open-filters",
                "label": "Open filters",
                "actions": ["click"],
                "hints": ["primary"],
                "attrs": {
                  "name": "filters",
                  "description": "Shows advanced search filters",
                  "aria": {"expanded": false}
                }
              }
            ]
          }
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
    "element_count": 10,
    "interactive_count": 5
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
	if som.Meta.ElementCount != 10 {
		t.Errorf("Meta.ElementCount = %d, want 10", som.Meta.ElementCount)
	}
	if som.Meta.InteractiveCount != 5 {
		t.Errorf("Meta.InteractiveCount = %d, want 5", som.Meta.InteractiveCount)
	}
}

func TestParseGroupAttrs(t *testing.T) {
	payload := `{
	  "som_version": "1.0",
	  "url": "https://example.com",
	  "title": "Groups",
	  "lang": "en",
	  "regions": [{
	    "id": "r_form",
	    "role": "form",
	    "elements": [{
	      "id": "e_group",
	      "role": "group",
	      "label": "Contact preference",
	      "attrs": {"legend": "Contact preference", "disabled": true}
	    }]
	  }],
	  "meta": {"html_bytes": 100, "som_bytes": 80, "element_count": 1, "interactive_count": 0}
	}`

	som, err := Parse([]byte(payload))
	if err != nil {
		t.Fatalf("ParseSom group payload: %v", err)
	}
	group := som.Regions[0].Elements[0]
	if group.Role != "group" {
		t.Fatalf("Role = %q, want group", group.Role)
	}
	if group.Attrs == nil || group.Attrs.Legend == nil || *group.Attrs.Legend != "Contact preference" {
		t.Fatalf("Legend = %v, want Contact preference", group.Attrs)
	}
	if group.Attrs.Disabled == nil || !*group.Attrs.Disabled {
		t.Fatalf("Disabled = %v, want true", group.Attrs.Disabled)
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

	// Check recently added actionability attrs.
	shadowButton := FindByID(som, "e10")
	if shadowButton == nil {
		t.Fatal("shadow button not found")
	}
	if shadowButton.Attrs == nil {
		t.Fatal("shadow button attrs nil")
	}
	if shadowButton.Attrs.Name == nil || *shadowButton.Attrs.Name != "filters" {
		t.Errorf("name = %v, want filters", shadowButton.Attrs.Name)
	}
	if shadowButton.Attrs.Description == nil || *shadowButton.Attrs.Description != "Shows advanced search filters" {
		t.Errorf("description = %v, want Shows advanced search filters", shadowButton.Attrs.Description)
	}
	if shadowButton.Attrs.Aria == nil || shadowButton.Attrs.Aria.Expanded == nil || *shadowButton.Attrs.Aria.Expanded {
		t.Errorf("aria.expanded = %v, want false", shadowButton.Attrs.Aria)
	}
	if shadowButton.HTMLID == nil || *shadowButton.HTMLID != "open-filters" {
		t.Errorf("HTMLID = %v, want open-filters", shadowButton.HTMLID)
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

	shadowEl := FindByID(som, "e10")
	if shadowEl == nil {
		t.Fatal("FindByID(e10) should traverse shadow roots")
	}
	if shadowEl.Label == nil || *shadowEl.Label != "Open filters" {
		t.Errorf("Label = %v, want Open filters", shadowEl.Label)
	}
}

func TestFindByHTMLID(t *testing.T) {
	som := mustParse(t)

	el := FindByHTMLID(som, "open-filters")
	if el == nil {
		t.Fatal("FindByHTMLID(open-filters) returned nil")
	}
	if el.ID != "e10" {
		t.Errorf("ID = %q, want e10", el.ID)
	}
	if FindByHTMLID(som, "missing-html-id") != nil {
		t.Error("FindByHTMLID(missing-html-id) should return nil")
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
	if len(interactive) != 5 {
		t.Fatalf("FindInteractive = %d, want 5", len(interactive))
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

	results = FindByText(som, "filters")
	if len(results) != 1 {
		t.Fatalf("FindByText(filters) = %d, want 1", len(results))
	}
	if results[0].ID != "e10" {
		t.Errorf("ID = %q, want e10", results[0].ID)
	}
}

func TestFlatElements(t *testing.T) {
	som := mustParse(t)

	flat := FlatElements(som)
	if len(flat) != 10 {
		t.Fatalf("FlatElements = %d, want 10", len(flat))
	}
}

func TestFindByActionAndHint(t *testing.T) {
	som := mustParse(t)

	clickable := FindByAction(som, "click")
	if len(clickable) != 5 {
		t.Fatalf("FindByAction(click) = %d, want 5", len(clickable))
	}

	primary := FindByHint(som, "primary")
	if len(primary) != 1 {
		t.Fatalf("FindByHint(primary) = %d, want 1", len(primary))
	}
	if primary[0].ID != "e10" {
		t.Errorf("ID = %q, want e10", primary[0].ID)
	}
}

func TestGetActionPlan(t *testing.T) {
	som := mustParse(t)

	plan := GetActionPlan(som)
	if len(plan) != 5 {
		t.Fatalf("GetActionPlan = %d, want 5", len(plan))
	}

	var filters *ActionPlanItem
	for i := range plan {
		if plan[i].ID == "e10" {
			filters = &plan[i]
			break
		}
	}
	if filters == nil {
		t.Fatal("action plan missing shadow-root button")
	}
	if filters.Label == nil || *filters.Label != "Open filters" {
		t.Errorf("Label = %v, want Open filters", filters.Label)
	}
	if filters.HTMLID == nil || *filters.HTMLID != "open-filters" {
		t.Errorf("HTMLID = %v, want open-filters", filters.HTMLID)
	}
	if !filters.Enabled {
		t.Error("Enabled = false, want true")
	}
	if filters.CacheKey == "" {
		t.Error("CacheKey is empty")
	}
	if got := GetActionPlanCacheKey(*filters); got != filters.CacheKey {
		t.Errorf("GetActionPlanCacheKey = %q, want %q", got, filters.CacheKey)
	}
	if filters.Name == nil || *filters.Name != "filters" {
		t.Errorf("Name = %v, want filters", filters.Name)
	}
	if filters.Description == nil || *filters.Description != "Shows advanced search filters" {
		t.Errorf("Description = %v, want Shows advanced search filters", filters.Description)
	}
	if len(filters.Actions) != 1 || filters.Actions[0] != "click" {
		t.Errorf("Actions = %v, want [click]", filters.Actions)
	}
}

func TestGetActionPlanDisabledTarget(t *testing.T) {
	disabled := true
	text := "Archive"
	som := &Som{
		Regions: []Region{
			{
				ID:   "r_form",
				Role: "form",
				Elements: []Element{
					{
						ID:      "locked",
						Role:    "button",
						Text:    &text,
						Actions: []string{"click"},
						Attrs:   &ElementAttrs{Disabled: &disabled},
					},
				},
			},
		},
	}

	plan := GetActionPlan(som)
	if len(plan) != 1 {
		t.Fatalf("GetActionPlan = %d, want 1", len(plan))
	}
	item := plan[0]
	if item.Enabled {
		t.Error("Enabled = true, want false")
	}
	if item.Disabled == nil || !*item.Disabled {
		t.Errorf("Disabled = %v, want true", item.Disabled)
	}
	if item.BlockedReason == nil || *item.BlockedReason != "disabled" {
		t.Errorf("BlockedReason = %v, want disabled", item.BlockedReason)
	}
	if item.CacheKey != "plasmate-action:v1:2de92b9a" {
		t.Errorf("CacheKey = %q, want plasmate-action:v1:2de92b9a", item.CacheKey)
	}
}

func TestGetActionPlanMatchesSharedAvailabilityManifest(t *testing.T) {
	somBytes, err := os.ReadFile("../../integrations/fixtures/action-availability.som.json")
	if err != nil {
		t.Fatalf("ReadFile SOM fixture failed: %v", err)
	}
	som, err := Parse(somBytes)
	if err != nil {
		t.Fatalf("Parse fixture failed: %v", err)
	}

	expectedBytes, err := os.ReadFile("../../integrations/fixtures/action-availability.expected.json")
	if err != nil {
		t.Fatalf("ReadFile expected fixture failed: %v", err)
	}
	var expected struct {
		ActionTargets []map[string]interface{} `json:"action_targets"`
	}
	if err := json.Unmarshal(expectedBytes, &expected); err != nil {
		t.Fatalf("Unmarshal expected fixture failed: %v", err)
	}

	actualBytes, err := json.Marshal(GetActionPlan(som))
	if err != nil {
		t.Fatalf("Marshal action plan failed: %v", err)
	}
	var actual []map[string]interface{}
	if err := json.Unmarshal(actualBytes, &actual); err != nil {
		t.Fatalf("Unmarshal action plan failed: %v", err)
	}

	if !reflect.DeepEqual(actual, expected.ActionTargets) {
		t.Errorf("GetActionPlan() = %#v, want %#v", actual, expected.ActionTargets)
	}
}

func TestActionPlanLookupHelpers(t *testing.T) {
	somBytes, err := os.ReadFile("../../integrations/fixtures/action-availability.som.json")
	if err != nil {
		t.Fatalf("ReadFile SOM fixture failed: %v", err)
	}
	som, err := Parse(somBytes)
	if err != nil {
		t.Fatalf("Parse fixture failed: %v", err)
	}

	save := FindActionTargetByID(som, "e_save")
	if save == nil {
		t.Fatal("FindActionTargetByID missing e_save")
	}
	if save.HTMLID == nil || *save.HTMLID != "save-button" {
		t.Fatalf("HTMLID = %v, want save-button", save.HTMLID)
	}
	if save.TestID == nil || *save.TestID != "settings-save" {
		t.Fatalf("TestID = %v, want settings-save", save.TestID)
	}
	if byCache := FindActionTargetByCacheKey(som, save.CacheKey); byCache == nil || byCache.ID != save.ID {
		t.Fatalf("FindActionTargetByCacheKey = %#v, want %s", byCache, save.ID)
	}
	if byHTML := FindActionTargetByHTMLID(som, "save-button"); byHTML == nil || byHTML.ID != save.ID {
		t.Fatalf("FindActionTargetByHTMLID = %#v, want %s", byHTML, save.ID)
	}
	if byTest := FindActionTargetByTestID(som, "settings-save"); byTest == nil || byTest.ID != save.ID {
		t.Fatalf("FindActionTargetByTestID = %#v, want %s", byTest, save.ID)
	}
	for _, value := range []string{"e_save", save.CacheKey, "save-button", "settings-save"} {
		if found := FindActionTarget(som, value); found == nil || found.ID != save.ID {
			t.Fatalf("FindActionTarget(%q) = %#v, want %s", value, found, save.ID)
		}
	}
	if byTest := FindActionTarget(som, "settings-save", "test_id"); byTest == nil || byTest.ID != save.ID {
		t.Fatalf("FindActionTarget(test_id) = %#v, want %s", byTest, save.ID)
	}

	index := GetActionPlanIndex(som)
	if index.ByID["e_save"].ID != save.ID {
		t.Fatalf("ByID[e_save] = %#v, want %s", index.ByID["e_save"], save.ID)
	}
	if index.ByCacheKey[save.CacheKey].ID != save.ID {
		t.Fatalf("ByCacheKey[%s] missing save target", save.CacheKey)
	}
	if index.ByHTMLID["save-button"].ID != save.ID {
		t.Fatalf("ByHTMLID[save-button] missing save target")
	}
	if index.ByTestID["settings-save"].ID != save.ID {
		t.Fatalf("ByTestID[settings-save] missing save target")
	}
	if index.ByLabel["Plan"].ID != "e_plan" {
		t.Fatalf("ByLabel[Plan] = %#v, want e_plan", index.ByLabel["Plan"])
	}
	if got := index.ByRole["button"]; len(got) != 2 || got[0].ID != "e_save" || got[1].ID != "e_preview" {
		t.Fatalf("ByRole[button] = %#v, want e_save/e_preview", got)
	}
	if got := index.ByAction["click"]; len(got) != 3 || got[0].ID != "e_save" || got[1].ID != "e_preview" || got[2].ID != "e_billing" {
		t.Fatalf("ByAction[click] = %#v, want e_save/e_preview/e_billing", got)
	}
	if got := FindActionTargetsByRole(som, "button"); len(got) != 2 || got[0].ID != "e_save" || got[1].ID != "e_preview" {
		t.Fatalf("FindActionTargetsByRole(button) = %#v, want e_save/e_preview", got)
	}
	if got := FindActionTargetsByAction(som, "click"); len(got) != 3 || got[0].ID != "e_save" || got[1].ID != "e_preview" || got[2].ID != "e_billing" {
		t.Fatalf("FindActionTargetsByAction(click) = %#v, want e_save/e_preview/e_billing", got)
	}
	if got := FindActionTargets(som, ActionTargetFilter{Role: "button", Action: "click", Label: "preview"}); len(got) != 1 || got[0].ID != "e_preview" {
		t.Fatalf("FindActionTargets(button/click/preview) = %#v, want e_preview", got)
	}
	if got := FindActionTargets(som, ActionTargetFilter{Role: "button", Action: "click", Label: "Preview changes", ExactLabel: true, EnabledOnly: true}); len(got) != 0 {
		t.Fatalf("enabled-only exact FindActionTargets(preview) = %#v, want none", got)
	}
	if found := FindActionTargetByLabel(som, "Plan"); found == nil || found.ID != "e_plan" {
		t.Fatalf("FindActionTargetByLabel(Plan) = %#v, want e_plan", found)
	}
	if found := FindActionTarget(som, "Plan", "label"); found == nil || found.ID != "e_plan" {
		t.Fatalf("FindActionTarget(label) = %#v, want e_plan", found)
	}
	if got := FindActionTargetsByLabel(som, "billing", false); len(got) != 2 || got[0].ID != "e_annual" || got[1].ID != "e_billing" {
		t.Fatalf("FindActionTargetsByLabel(billing) = %#v, want e_annual/e_billing", got)
	}
	if got := FindActionTargetsByLabel(som, "Billing settings", true); len(got) != 1 || got[0].ID != "e_billing" {
		t.Fatalf("FindActionTargetsByLabel(exact Billing settings) = %#v, want e_billing", got)
	}
	if found := FindActionTargetInIndex(index, save.CacheKey); found == nil || found.ID != save.ID {
		t.Fatalf("FindActionTargetInIndex(cache_key) = %#v, want %s", found, save.ID)
	}
	if found := FindActionTargetInIndex(index, "Plan", "label"); found == nil || found.ID != "e_plan" {
		t.Fatalf("FindActionTargetInIndex(label) = %#v, want e_plan", found)
	}
}

func TestEnabledActionPlanIndexFiltersBlockedTargets(t *testing.T) {
	somBytes, err := os.ReadFile("../../integrations/fixtures/action-availability.som.json")
	if err != nil {
		t.Fatalf("ReadFile SOM fixture failed: %v", err)
	}
	som, err := Parse(somBytes)
	if err != nil {
		t.Fatalf("Parse fixture failed: %v", err)
	}

	for _, item := range EnabledActionPlan(som) {
		if !item.Enabled {
			t.Fatalf("EnabledActionPlan included disabled target: %#v", item)
		}
	}
	index := GetActionPlanIndex(som, true)
	if _, ok := index.ByID["e_save"]; ok {
		t.Fatal("enabled-only index included blocked e_save target")
	}
	if _, ok := index.ByTestID["settings-save"]; ok {
		t.Fatal("enabled-only index included blocked settings-save target")
	}
	if _, ok := index.ByLabel["Preview changes"]; ok {
		t.Fatal("enabled-only index included blocked Preview changes label")
	}
	if _, ok := index.ByID["e_plan"]; !ok {
		t.Fatal("enabled-only index omitted enabled e_plan target")
	}
	if index.ByLabel["Plan"].ID != "e_plan" {
		t.Fatalf("enabled-only ByLabel[Plan] = %#v, want e_plan", index.ByLabel["Plan"])
	}
	if _, ok := index.ByRole["button"]; ok {
		t.Fatal("enabled-only index included blocked button targets")
	}
	if got := index.ByAction["click"]; len(got) != 1 || got[0].ID != "e_billing" {
		t.Fatalf("enabled-only ByAction[click] = %#v, want e_billing", got)
	}
	if got := FindActionTargetsByAction(som, "click", true); len(got) != 1 || got[0].ID != "e_billing" {
		t.Fatalf("enabled-only FindActionTargetsByAction(click) = %#v, want e_billing", got)
	}
	if got := FindActionTargetsByLabel(som, "preview", false, true); len(got) != 0 {
		t.Fatalf("enabled-only FindActionTargetsByLabel(preview) = %#v, want none", got)
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
