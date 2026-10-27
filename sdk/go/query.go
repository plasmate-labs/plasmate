package plasmate

import (
	"encoding/json"
	"fmt"
	"hash/fnv"
	"sort"
	"strings"
)

// FindByRole returns all regions matching the given role.
func FindByRole(som *Som, role string) []Region {
	if som == nil {
		return nil
	}
	var result []Region
	role = normalizeRegionRoleAlias(role)
	for _, r := range som.Regions {
		if r.Role == role {
			result = append(result, r)
		}
	}
	return result
}

// normalizeRegionRoleAlias keeps region lookups tolerant of common HTML/ARIA
// names while preserving the canonical SOM roles.
func normalizeRegionRoleAlias(role string) string {
	role = normalizeRoleQuery(role)
	aliases := map[string]string{
		"article":            "content",
		"articles":           "content",
		"asides":             "aside",
		"banner":             "header",
		"complementary":      "aside",
		"contentinfo":        "footer",
		"contents":           "content",
		"content_region":     "content",
		"content_regions":    "content",
		"dialog":             "dialog",
		"dialogs":            "dialog",
		"footers":            "footer",
		"form":               "form",
		"forms":              "form",
		"headers":            "header",
		"main_content":       "main",
		"main_contents":      "main",
		"main_region":        "main",
		"main_regions":       "main",
		"maincontent":        "main",
		"maincontents":       "main",
		"nav":                "navigation",
		"navs":               "navigation",
		"navigation":         "navigation",
		"navigations":        "navigation",
		"nav_region":         "navigation",
		"nav_regions":        "navigation",
		"navigation_region":  "navigation",
		"navigation_regions": "navigation",
		"region":             "content",
		"regions":            "content",
		"search":             "navigation",
		"searches":           "navigation",
		"search_nav":         "navigation",
		"search_navigation":  "navigation",
		"search_navigations": "navigation",
		"sidebar":            "aside",
	}
	if canonical, ok := aliases[role]; ok {
		return canonical
	}
	return role
}

// FindByRegionID searches top-level regions for the given SOM region ID.
// Returns nil if not found.
func FindByRegionID(som *Som, id string) *Region {
	if som == nil {
		return nil
	}
	for i := range som.Regions {
		if som.Regions[i].ID == id {
			return &som.Regions[i]
		}
	}
	return nil
}

// FindByID searches all regions for an element with the given ID.
// Returns nil if not found.
func FindByID(som *Som, id string) *Element {
	if som == nil {
		return nil
	}
	for _, r := range som.Regions {
		if el := findByIDInSlice(r.Elements, id); el != nil {
			return el
		}
	}
	return nil
}

// FindByHTMLID searches all regions for an element with the original HTML id.
// Returns nil if not found.
func FindByHTMLID(som *Som, htmlID string) *Element {
	if som == nil {
		return nil
	}
	for i := range som.Regions {
		if el := findByHTMLIDInSlice(som.Regions[i].Elements, htmlID); el != nil {
			return el
		}
	}
	return nil
}

func findByHTMLIDInSlice(elements []Element, htmlID string) *Element {
	for i := range elements {
		if elements[i].HTMLID != nil && *elements[i].HTMLID == htmlID {
			return &elements[i]
		}
		if el := findByHTMLIDInSlice(elements[i].Children, htmlID); el != nil {
			return el
		}
		if elements[i].Shadow != nil {
			if el := findByHTMLIDInSlice(elements[i].Shadow.Elements, htmlID); el != nil {
				return el
			}
		}
	}
	return nil
}

func findByIDInSlice(elements []Element, id string) *Element {
	for i := range elements {
		if elements[i].ID == id {
			return &elements[i]
		}
		if el := findByIDInSlice(elements[i].Children, id); el != nil {
			return el
		}
		if elements[i].Shadow != nil {
			if el := findByIDInSlice(elements[i].Shadow.Elements, id); el != nil {
				return el
			}
		}
	}
	return nil
}

// FindByTag returns all elements whose role matches the given tag.
// This searches element roles (e.g. "link", "button", "heading").
func FindByTag(som *Som, tag string) []Element {
	if som == nil {
		return nil
	}
	var result []Element
	tag = normalizeElementRoleAlias(tag)
	if sourceRole, ok := mediaTagAliases[tag]; ok {
		for _, element := range FlatElements(som) {
			if element.Attrs != nil && element.Attrs.SourceRole != nil && *element.Attrs.SourceRole == sourceRole {
				result = append(result, element)
			}
		}
		return result
	}
	if mediaGroupAliases[tag] {
		for _, element := range FlatElements(som) {
			if element.Role == "image" || (element.Attrs != nil && element.Attrs.SourceRole != nil && mediaSourceRoles[*element.Attrs.SourceRole]) {
				result = append(result, element)
			}
		}
		return result
	}
	for _, r := range som.Regions {
		collectByRole(r.Elements, tag, &result)
	}
	return result
}

var mediaTagAliases = map[string]string{
	"audio":        "audio",
	"audios":       "audio",
	"audio_item":   "audio",
	"audio_items":  "audio",
	"canvas":       "canvas",
	"canvases":     "canvas",
	"canvas_item":  "canvas",
	"canvas_items": "canvas",
	"video":        "video",
	"videos":       "video",
	"video_item":   "video",
	"video_items":  "video",
}

var mediaGroupAliases = map[string]bool{
	"media":       true,
	"media_item":  true,
	"mediaitem":   true,
	"media_items": true,
}

var mediaSourceRoles = map[string]bool{
	"audio":  true,
	"video":  true,
	"canvas": true,
}

// normalizeElementRoleAlias keeps Go SDK role lookups aligned with the
// common HTML/ARIA aliases accepted by the native selector surface.
func normalizeElementRoleAlias(role string) string {
	role = normalizeRoleQuery(role)
	aliases := map[string]string{
		"a":                  "link",
		"article":            "section",
		"button":             "button",
		"buttons":            "button",
		"checkboxes":         "checkbox",
		"check_box":          "checkbox",
		"check_boxes":        "checkbox",
		"combobox":           "select",
		"comboboxes":         "select",
		"combo_box":          "select",
		"combo_boxes":        "select",
		"definition_list":    "list",
		"data_table":         "table",
		"data_tables":        "table",
		"div":                "group",
		"details":            "details",
		"fieldset":           "group",
		"fieldsets":          "group",
		"figure":             "group",
		"figures":            "group",
		"headings":           "heading",
		"hr":                 "separator",
		"img":                "image",
		"images":             "image",
		"input":              "text_input",
		"inputs":             "text_input",
		"links":              "link",
		"list_box":           "select",
		"list_boxes":         "select",
		"listbox":            "select",
		"listboxes":          "select",
		"lists":              "list",
		"menu":               "list",
		"menu_item":          "button",
		"menuitem":           "button",
		"menuitemcheckbox":   "checkbox",
		"menu_item_checkbox": "checkbox",
		"menuitemradio":      "radio",
		"menu_item_radio":    "radio",
		"ol":                 "list",
		"option":             "button",
		"options":            "button",
		"ordered_list":       "list",
		"p":                  "paragraph",
		"blockquote":         "paragraph",
		"blockquotes":        "paragraph",
		"quote":              "paragraph",
		"quotes":             "paragraph",
		"paragraphs":         "paragraph",
		"progress":           "group",
		"progressbar":        "group",
		"progressbars":       "group",
		"meter":              "group",
		"meters":             "group",
		"query":              "text_input",
		"queries":            "text_input",
		"query_box":          "text_input",
		"query_boxes":        "text_input",
		"querybox":           "text_input",
		"queryboxes":         "text_input",
		"query_control":      "text_input",
		"query_controls":     "text_input",
		"query_field":        "text_input",
		"query_fields":       "text_input",
		"query_input":        "text_input",
		"query_inputs":       "text_input",
		"radio_buttons":      "radio",
		"radio_button":       "radio",
		"radios":             "radio",
		"radiobutton":        "radio",
		"radiobuttons":       "radio",
		"range":              "text_input",
		"radiogroup":         "group",
		"radiogroups":        "group",
		"search":             "text_input",
		"search_box":         "text_input",
		"search_boxes":       "text_input",
		"search_control":     "text_input",
		"search_controls":    "text_input",
		"search_field":       "text_input",
		"search_fields":      "text_input",
		"search_input":       "text_input",
		"search_inputs":      "text_input",
		"searchbox":          "text_input",
		"searchboxes":        "text_input",
		"sections":           "section",
		"selects":            "select",
		"slider":             "text_input",
		"sliders":            "text_input",
		"spin_button":        "text_input",
		"spin_buttons":       "text_input",
		"spinbutton":         "text_input",
		"spinbuttons":        "text_input",
		"summary":            "details",
		"switch":             "checkbox",
		"switches":           "checkbox",
		"tab":                "button",
		"tabs":               "button",
		"tables":             "table",
		"textarea":           "textarea",
		"textareas":          "textarea",
		"textboxes":          "text_input",
		"text_inputs":        "text_input",
		"textbox":            "text_input",
		"tree_item":          "button",
		"treeitem":           "button",
		"tree_items":         "button",
		"treeitems":          "button",
		"ul":                 "list",
		"unordered_list":     "list",
		"groups":             "group",
		"iframes":            "iframe",
	}
	for alias, canonical := range map[string]string{
		"menu_items":           "button",
		"menuitems":            "button",
		"menuitemcheckboxes":   "checkbox",
		"menu_item_checkboxes": "checkbox",
		"menuitemradios":       "radio",
		"menu_item_radios":     "radio",
	} {
		aliases[alias] = canonical
	}
	if canonical, ok := aliases[role]; ok {
		return canonical
	}
	return role
}

func normalizeRoleQuery(value string) string {
	value = strings.TrimSpace(strings.ToLower(value))
	if len(value) >= 2 && ((value[0] == '"' && value[len(value)-1] == '"') ||
		(value[0] == '\'' && value[len(value)-1] == '\'')) {
		value = strings.TrimSpace(value[1 : len(value)-1])
	}
	if strings.HasPrefix(value, "role") {
		rest := strings.TrimSpace(value[len("role"):])
		if strings.HasPrefix(rest, ":") || strings.HasPrefix(rest, "=") {
			value = strings.TrimSpace(rest[1:])
			if len(value) >= 2 && ((value[0] == '"' && value[len(value)-1] == '"') ||
				(value[0] == '\'' && value[len(value)-1] == '\'')) {
				value = strings.TrimSpace(value[1 : len(value)-1])
			}
		}
	}
	return strings.Join(strings.Fields(strings.ReplaceAll(value, "-", " ")), "_")
}

func collectByRole(elements []Element, role string, result *[]Element) {
	for _, el := range elements {
		if el.Role == role {
			*result = append(*result, el)
		}
		collectByRole(el.Children, role, result)
		if el.Shadow != nil {
			collectByRole(el.Shadow.Elements, role, result)
		}
	}
}

// FindInteractive returns all elements that have at least one action.
func FindInteractive(som *Som) []Element {
	if som == nil {
		return nil
	}
	var result []Element
	for _, r := range som.Regions {
		collectInteractive(r.Elements, &result)
	}
	return result
}

func collectInteractive(elements []Element, result *[]Element) {
	for _, el := range elements {
		if len(el.Actions) > 0 {
			*result = append(*result, el)
		}
		collectInteractive(el.Children, result)
		if el.Shadow != nil {
			collectInteractive(el.Shadow.Elements, result)
		}
	}
}

// FindByText returns all elements whose text contains the given whitespace-
// normalized substring
// (case-insensitive). User-facing metadata such as labels, alt text,
// placeholders, titles, values, descriptions, and ARIA value text is searched
// alongside compiled list-item copy, select options, and table content because
// those values can be the only authored text available for an element.
func FindByText(som *Som, text string) []Element {
	if som == nil || text == "" {
		return nil
	}
	lower := normalizeQueryText(text)
	if lower == "" {
		return nil
	}
	var result []Element
	for _, r := range som.Regions {
		collectByText(r.Elements, lower, &result)
	}
	return result
}

// FindByLabel returns all elements whose accessible label matches label.
// Matching is case-insensitive and uses substring matching.
func FindByLabel(som *Som, label string, exact ...bool) []Element {
	if som == nil {
		return nil
	}
	needle := normalizeQueryText(label)
	if needle == "" {
		return nil
	}
	exactMatch := len(exact) > 0 && exact[0]
	var result []Element
	for _, r := range som.Regions {
		collectByLabel(r.Elements, needle, exactMatch, &result)
	}
	return result
}

func collectByLabel(elements []Element, lowerLabel string, exact bool, result *[]Element) {
	for _, el := range elements {
		if elementMatchesLabel(el, lowerLabel, exact) {
			*result = append(*result, el)
		}
		collectByLabel(el.Children, lowerLabel, exact, result)
		if el.Shadow != nil {
			collectByLabel(el.Shadow.Elements, lowerLabel, exact, result)
		}
	}
}

// elementMatchesLabel checks the compiled accessible label and the attributes
// that can supply a label when the page has no explicit accessible name.
func elementMatchesLabel(el Element, lowerLabel string, exact bool) bool {
	matches := func(value *string) bool {
		if value == nil {
			return false
		}
		candidate := normalizeQueryText(*value)
		return (exact && candidate == lowerLabel) || (!exact && strings.Contains(candidate, lowerLabel))
	}

	if matches(el.Label) {
		return true
	}
	if len(el.Actions) > 0 && matches(el.Text) {
		return true
	}
	if el.Attrs == nil {
		return false
	}
	return matches(el.Attrs.Title) ||
		matches(el.Attrs.Placeholder) ||
		matches(el.Attrs.Alt) ||
		(el.Attrs.Aria != nil && matches(el.Attrs.Aria.Label))
}

func normalizeQueryText(value string) string {
	value = strings.NewReplacer("\u200b", " ", "\u2060", " ", "\ufeff", " ").Replace(value)
	return strings.ToLower(strings.Join(strings.Fields(value), " "))
}

func collectByText(elements []Element, lowerText string, result *[]Element) {
	for _, el := range elements {
		if elementMatchesText(el, lowerText) {
			*result = append(*result, el)
		}
		collectByText(el.Children, lowerText, result)
		if el.Shadow != nil {
			collectByText(el.Shadow.Elements, lowerText, result)
		}
	}
}

func elementMatchesText(el Element, lowerText string) bool {
	matches := func(value *string) bool {
		return value != nil && strings.Contains(normalizeQueryText(*value), lowerText)
	}
	matchValue := func(value string) bool {
		return strings.Contains(normalizeQueryText(value), lowerText)
	}

	if matches(el.Text) {
		return true
	}
	if matches(el.Label) {
		return true
	}
	if el.Attrs != nil {
		for _, value := range []*string{
			el.Attrs.Alt,
			el.Attrs.Placeholder,
			el.Attrs.Title,
			el.Attrs.Value,
			el.Attrs.Description,
			el.Attrs.SectionLabel,
			el.Attrs.Legend,
			el.Attrs.Summary,
		} {
			if matches(value) {
				return true
			}
		}
		if aria := el.Attrs.Aria; aria != nil {
			for _, value := range []*string{
				aria.Label,
				aria.Placeholder,
				aria.ValueText,
				aria.RoleDescription,
			} {
				if matches(value) {
					return true
				}
			}
		}
		for _, item := range el.Attrs.Items {
			if matchValue(item.Text) {
				return true
			}
		}
		if matches(el.Attrs.Caption) {
			return true
		}
		for _, option := range el.Attrs.Options {
			if matchValue(option.Text) {
				return true
			}
		}
		for _, header := range el.Attrs.Headers {
			if matchValue(header) {
				return true
			}
		}
		for _, row := range el.Attrs.Rows {
			for _, cell := range row {
				if matchValue(cell) {
					return true
				}
			}
		}
	}
	return false
}

// FindByAction returns all elements that expose a specific action.
func FindByAction(som *Som, action string) []Element {
	if som == nil {
		return nil
	}
	var result []Element
	for _, el := range FlatElements(som) {
		if containsString(el.Actions, action) {
			result = append(result, el)
		}
	}
	return result
}

// FindByHint returns all elements tagged with a specific semantic hint.
func FindByHint(som *Som, hint string) []Element {
	if som == nil {
		return nil
	}
	var result []Element
	for _, el := range FlatElements(som) {
		if containsString(el.Hints, hint) {
			result = append(result, el)
		}
	}
	return result
}

func containsString(values []string, target string) bool {
	for _, value := range values {
		if value == target {
			return true
		}
	}
	return false
}

// FlatElements returns all elements from all regions flattened into a single slice,
// including nested children and shadow-root elements via depth-first traversal.
func FlatElements(som *Som) []Element {
	if som == nil {
		return nil
	}
	var result []Element
	for _, r := range som.Regions {
		flattenElements(r.Elements, &result)
	}
	return result
}

func flattenElements(elements []Element, result *[]Element) {
	for _, el := range elements {
		*result = append(*result, el)
		flattenElements(el.Children, result)
		if el.Shadow != nil {
			flattenElements(el.Shadow.Elements, result)
		}
	}
}

// ActionPlanItem is a compact action target for agent planning.
type ActionPlanItem struct {
	ID                string      `json:"id"`
	CacheKey          string      `json:"cache_key"`
	Role              string      `json:"role"`
	Actions           []string    `json:"actions"`
	Enabled           bool        `json:"enabled"`
	Label             *string     `json:"label,omitempty"`
	HTMLID            *string     `json:"html_id,omitempty"`
	Href              *string     `json:"href,omitempty"`
	Target            *string     `json:"target,omitempty"`
	Rel               *string     `json:"rel,omitempty"`
	HrefLang          *string     `json:"hreflang,omitempty"`
	Type              *string     `json:"type,omitempty"`
	ReferrerPolicy    *string     `json:"referrerpolicy,omitempty"`
	Download          interface{} `json:"download,omitempty"`
	Name              *string     `json:"name,omitempty"`
	Accept            *string     `json:"accept,omitempty"`
	Capture           interface{} `json:"capture,omitempty"`
	Multiple          *bool       `json:"multiple,omitempty"`
	SelectedValues    []string    `json:"selected_values,omitempty"`
	Size              interface{} `json:"size,omitempty"`
	Autocomplete      *string     `json:"autocomplete,omitempty"`
	InputMode         *string     `json:"inputmode,omitempty"`
	EnterKeyHint      *string     `json:"enterkeyhint,omitempty"`
	AutoCapitalize    *string     `json:"autocapitalize,omitempty"`
	DirName           *string     `json:"dirname,omitempty"`
	Form              *string     `json:"form,omitempty"`
	FormAction        *string     `json:"form_action,omitempty"`
	FormMethod        *string     `json:"form_method,omitempty"`
	FormTarget        *string     `json:"form_target,omitempty"`
	FormEnctype       *string     `json:"form_enctype,omitempty"`
	FormNoValidate    *bool       `json:"form_novalidate,omitempty"`
	FormAcceptCharset *string     `json:"form_accept_charset,omitempty"`
	FormAutocomplete  *string     `json:"form_autocomplete,omitempty"`
	List              *string     `json:"list,omitempty"`
	PopoverTarget     *string     `json:"popovertarget,omitempty"`
	PopoverAction     *string     `json:"popovertargetaction,omitempty"`
	CommandFor        *string     `json:"commandfor,omitempty"`
	Command           *string     `json:"command,omitempty"`
	Popover           *string     `json:"popover,omitempty"`
	ButtonType        *string     `json:"button_type,omitempty"`
	SubmitFormAction  *string     `json:"formaction,omitempty"`
	SubmitFormMethod  *string     `json:"formmethod,omitempty"`
	SubmitFormEnctype *string     `json:"formenctype,omitempty"`
	SubmitFormTarget  *string     `json:"formtarget,omitempty"`
	SubmitNoValidate  *bool       `json:"formnovalidate,omitempty"`
	AccessKey         *string     `json:"accesskey,omitempty"`
	Title             *string     `json:"title,omitempty"`
	Alt               *string     `json:"alt,omitempty"`
	SourceRole        *string     `json:"source_role,omitempty"`
	TestID            *string     `json:"test_id,omitempty"`
	Spellcheck        interface{} `json:"spellcheck,omitempty"`
	Draggable         interface{} `json:"draggable,omitempty"`
	InputType         *string     `json:"input_type,omitempty"`
	Value             *string     `json:"value,omitempty"`
	Placeholder       *string     `json:"placeholder,omitempty"`
	MinLength         interface{} `json:"minlength,omitempty"`
	MaxLength         interface{} `json:"maxlength,omitempty"`
	Min               interface{} `json:"min,omitempty"`
	Max               interface{} `json:"max,omitempty"`
	Step              *string     `json:"step,omitempty"`
	Pattern           *string     `json:"pattern,omitempty"`
	Description       *string     `json:"description,omitempty"`
	Checked           interface{} `json:"checked,omitempty"`
	Expanded          *bool       `json:"expanded,omitempty"`
	Pressed           *bool       `json:"pressed,omitempty"`
	Selected          *bool       `json:"selected,omitempty"`
	Multiline         *bool       `json:"multiline,omitempty"`
	MultiSelectable   *bool       `json:"multiselectable,omitempty"`
	Current           interface{} `json:"current,omitempty"`
	Controls          *string     `json:"controls,omitempty"`
	HasPopup          interface{} `json:"haspopup,omitempty"`
	Invalid           interface{} `json:"invalid,omitempty"`
	AriaPlaceholder   *string     `json:"aria_placeholder,omitempty"`
	AriaAutocomplete  *string     `json:"aria_autocomplete,omitempty"`
	AriaLabel         *string     `json:"aria_label,omitempty"`
	LabelledBy        *string     `json:"labelledby,omitempty"`
	DescribedBy       *string     `json:"describedby,omitempty"`
	ActiveDescendant  *string     `json:"active_descendant,omitempty"`
	ErrorMessage      *string     `json:"errormessage,omitempty"`
	KeyShortcuts      *string     `json:"keyshortcuts,omitempty"`
	RoleDescription   *string     `json:"roledescription,omitempty"`
	Busy              *bool       `json:"busy,omitempty"`
	Live              *string     `json:"live,omitempty"`
	Atomic            *bool       `json:"atomic,omitempty"`
	Relevant          *string     `json:"relevant,omitempty"`
	Owns              *string     `json:"owns,omitempty"`
	FlowTo            *string     `json:"flowto,omitempty"`
	Details           *string     `json:"details,omitempty"`
	Orientation       *string     `json:"orientation,omitempty"`
	Sort              *string     `json:"sort,omitempty"`
	Level             *string     `json:"level,omitempty"`
	PosInSet          *string     `json:"posinset,omitempty"`
	SetSize           *string     `json:"setsize,omitempty"`
	Grabbed           *bool       `json:"grabbed,omitempty"`
	DropEffect        *string     `json:"dropeffect,omitempty"`
	ValueMin          *string     `json:"valuemin,omitempty"`
	ValueMax          *string     `json:"valuemax,omitempty"`
	ValueNow          *string     `json:"valuenow,omitempty"`
	ValueText         *string     `json:"valuetext,omitempty"`
	Required          *bool       `json:"required,omitempty"`
	Readonly          *bool       `json:"readonly,omitempty"`
	Disabled          *bool       `json:"disabled,omitempty"`
	Inert             *bool       `json:"inert,omitempty"`
	BlockedReason     *string     `json:"blocked_reason,omitempty"`
	Group             *string     `json:"group,omitempty"`
}

// ActionPlanIndex groups compact action targets for replay lookups and planning.
type ActionPlanIndex struct {
	ByID       map[string]ActionPlanItem   `json:"by_id"`
	ByCacheKey map[string]ActionPlanItem   `json:"by_cache_key"`
	ByHTMLID   map[string]ActionPlanItem   `json:"by_html_id"`
	ByTestID   map[string]ActionPlanItem   `json:"by_test_id"`
	ByLabel    map[string]ActionPlanItem   `json:"by_label"`
	ByRole     map[string][]ActionPlanItem `json:"by_role"`
	ByAction   map[string][]ActionPlanItem `json:"by_action"`
}

func compactString(value *string) interface{} {
	if value != nil && *value != "" {
		return *value
	}
	return nil
}

func actionPlanLabel(el Element) *string {
	candidates := []*string{el.Label, el.Text}
	if el.Attrs != nil {
		var ariaLabel *string
		if el.Attrs.Aria != nil {
			ariaLabel = el.Attrs.Aria.Label
		}
		candidates = append(candidates, ariaLabel, el.Attrs.Title, el.Attrs.Placeholder, el.Attrs.Alt)
	}
	for _, candidate := range candidates {
		if candidate != nil && normalizeQueryText(*candidate) != "" {
			return candidate
		}
	}
	return nil
}

func actionPlanCacheParts(item ActionPlanItem) []interface{} {
	actions := append([]string(nil), item.Actions...)
	sort.Strings(actions)
	actionList := strings.Join(actions, ",")
	var actionValue interface{}
	if actionList != "" {
		actionValue = actionList
	}
	return []interface{}{
		item.ID,
		item.Role,
		compactString(item.Label),
		actionValue,
		compactString(item.Name),
		compactString(item.Href),
		compactString(item.InputType),
		compactString(item.Group),
		compactString(item.Placeholder),
	}
}

// GetActionPlanCacheKey returns a deterministic key for caching or comparing an action target.
func GetActionPlanCacheKey(item ActionPlanItem) string {
	encoded, err := json.Marshal(actionPlanCacheParts(item))
	if err != nil {
		return "plasmate-action:v1:00000000"
	}
	hash := fnv.New32a()
	_, _ = hash.Write(encoded)
	return fmt.Sprintf("plasmate-action:v1:%08x", hash.Sum32())
}

// GetActionPlan returns compact action targets for agent planning.
func GetActionPlan(som *Som) []ActionPlanItem {
	items := []ActionPlanItem{}
	if som == nil {
		return items
	}
	formContextByID := map[string]Region{}
	for _, region := range som.Regions {
		for _, el := range flattenRegionElements(region.Elements) {
			if len(el.Actions) > 0 {
				formContextByID[el.ID] = region
			}
		}
	}
	for _, el := range FindInteractive(som) {
		item := ActionPlanItem{
			ID:      el.ID,
			Role:    el.Role,
			Actions: append([]string(nil), el.Actions...),
			Enabled: true,
		}
		item.Label = actionPlanLabel(el)
		item.HTMLID = el.HTMLID
		if el.Attrs != nil {
			item.Href = el.Attrs.Href
			item.Target = el.Attrs.Target
			item.Rel = el.Attrs.Rel
			item.HrefLang = el.Attrs.HrefLang
			item.Type = el.Attrs.Type
			item.ReferrerPolicy = el.Attrs.ReferrerPolicy
			item.Download = el.Attrs.Download
			item.Name = el.Attrs.Name
			item.Accept = el.Attrs.Accept
			item.Capture = el.Attrs.Capture
			item.Multiple = el.Attrs.Multiple
			item.SelectedValues = el.Attrs.SelectedValues
			item.Size = el.Attrs.Size
			item.Autocomplete = el.Attrs.Autocomplete
			item.InputMode = el.Attrs.InputMode
			item.EnterKeyHint = el.Attrs.EnterKeyHint
			item.AutoCapitalize = el.Attrs.AutoCapitalize
			item.DirName = el.Attrs.DirName
			item.Form = el.Attrs.Form
			item.List = el.Attrs.List
			item.PopoverTarget = el.Attrs.PopoverTarget
			item.PopoverAction = el.Attrs.PopoverAction
			item.CommandFor = el.Attrs.CommandFor
			item.Command = el.Attrs.Command
			item.Popover = el.Attrs.Popover
			item.ButtonType = el.Attrs.ButtonType
			item.SubmitFormAction = el.Attrs.FormAction
			item.SubmitFormMethod = el.Attrs.FormMethod
			item.SubmitFormEnctype = el.Attrs.FormEnctype
			item.SubmitFormTarget = el.Attrs.FormTarget
			item.SubmitNoValidate = el.Attrs.FormNoValidate
			item.AccessKey = el.Attrs.AccessKey
			item.Title = el.Attrs.Title
			item.Alt = el.Attrs.Alt
			item.SourceRole = el.Attrs.SourceRole
			item.TestID = el.Attrs.TestID
			item.Spellcheck = el.Attrs.Spellcheck
			item.Draggable = el.Attrs.Draggable
			item.InputType = el.Attrs.InputType
			item.Value = el.Attrs.Value
			item.Placeholder = el.Attrs.Placeholder
			item.MinLength = el.Attrs.MinLength
			item.MaxLength = el.Attrs.MaxLength
			item.Min = el.Attrs.Min
			item.Max = el.Attrs.Max
			item.Step = el.Attrs.Step
			item.Pattern = el.Attrs.Pattern
			item.Description = el.Attrs.Description
			if el.Attrs.Checked != nil {
				item.Checked = *el.Attrs.Checked
			} else if el.Attrs.Aria != nil && el.Attrs.Aria.Checked != nil {
				item.Checked = el.Attrs.Aria.Checked
			}
			if el.Attrs.Aria != nil {
				item.Expanded = el.Attrs.Aria.Expanded
				if item.Readonly == nil {
					item.Readonly = el.Attrs.Aria.Readonly
				}
				item.Pressed = el.Attrs.Aria.Pressed
				item.Selected = el.Attrs.Aria.Selected
				item.Current = el.Attrs.Aria.Current
				item.Controls = el.Attrs.Aria.Controls
				item.HasPopup = el.Attrs.Aria.HasPopup
				item.Invalid = el.Attrs.Aria.Invalid
				item.AriaPlaceholder = el.Attrs.Aria.Placeholder
				item.AriaAutocomplete = el.Attrs.Aria.Autocomplete
				item.AriaLabel = el.Attrs.Aria.Label
				item.LabelledBy = el.Attrs.Aria.LabelledBy
				item.DescribedBy = el.Attrs.Aria.DescribedBy
				item.ActiveDescendant = el.Attrs.Aria.ActiveDescendant
				item.ErrorMessage = el.Attrs.Aria.ErrorMessage
				item.KeyShortcuts = el.Attrs.Aria.KeyShortcuts
				item.RoleDescription = el.Attrs.Aria.RoleDescription
				item.Busy = el.Attrs.Aria.Busy
				item.Live = el.Attrs.Aria.Live
				item.Atomic = el.Attrs.Aria.Atomic
				item.Relevant = el.Attrs.Aria.Relevant
				item.Owns = el.Attrs.Aria.Owns
				item.FlowTo = el.Attrs.Aria.FlowTo
				item.Details = el.Attrs.Aria.Details
				item.Multiline = el.Attrs.Aria.Multiline
				item.MultiSelectable = el.Attrs.Aria.MultiSelectable
				item.Orientation = el.Attrs.Aria.Orientation
				item.Sort = el.Attrs.Aria.Sort
				item.Level = el.Attrs.Aria.Level
				item.PosInSet = el.Attrs.Aria.PosInSet
				item.SetSize = el.Attrs.Aria.SetSize
				item.Grabbed = el.Attrs.Aria.Grabbed
				item.DropEffect = el.Attrs.Aria.DropEffect
				item.ValueMin = el.Attrs.Aria.ValueMin
				item.ValueMax = el.Attrs.Aria.ValueMax
				item.ValueNow = el.Attrs.Aria.ValueNow
				item.ValueText = el.Attrs.Aria.ValueText
			}
			item.Required = el.Attrs.Required
			if el.Attrs.Readonly != nil {
				item.Readonly = el.Attrs.Readonly
			}
			item.Disabled = el.Attrs.Disabled
			if el.Attrs.Disabled != nil && *el.Attrs.Disabled {
				item.Enabled = false
				reason := "disabled"
				item.BlockedReason = &reason
			}
			item.Inert = el.Attrs.Inert
			if el.Attrs.Inert != nil && *el.Attrs.Inert {
				item.Enabled = false
				reason := "inert"
				item.BlockedReason = &reason
			} else if item.Enabled && item.Readonly != nil && *item.Readonly {
				item.Enabled = false
				reason := "readonly"
				item.BlockedReason = &reason
			}
			item.Group = el.Attrs.Group
		}
		if region, ok := formContextByID[el.ID]; ok {
			copyFormContext(&item, region)
		}
		item.CacheKey = GetActionPlanCacheKey(item)
		items = append(items, item)
	}
	return items
}

// EnabledActionPlan returns compact action targets that are currently safe to offer.
func EnabledActionPlan(som *Som) []ActionPlanItem {
	var result []ActionPlanItem
	for _, item := range GetActionPlan(som) {
		if item.Enabled {
			result = append(result, item)
		}
	}
	return result
}

// GetActionPlanIndex returns action targets indexed by replay ids and grouped by role/action.
func GetActionPlanIndex(som *Som, enabledOnly ...bool) ActionPlanIndex {
	plan := GetActionPlan(som)
	if len(enabledOnly) > 0 && enabledOnly[0] {
		plan = EnabledActionPlan(som)
	}
	index := ActionPlanIndex{
		ByID:       map[string]ActionPlanItem{},
		ByCacheKey: map[string]ActionPlanItem{},
		ByHTMLID:   map[string]ActionPlanItem{},
		ByTestID:   map[string]ActionPlanItem{},
		ByLabel:    map[string]ActionPlanItem{},
		ByRole:     map[string][]ActionPlanItem{},
		ByAction:   map[string][]ActionPlanItem{},
	}
	for _, item := range plan {
		if _, ok := index.ByID[item.ID]; !ok {
			index.ByID[item.ID] = item
		}
		if _, ok := index.ByCacheKey[item.CacheKey]; !ok {
			index.ByCacheKey[item.CacheKey] = item
		}
		if item.HTMLID != nil {
			if _, ok := index.ByHTMLID[*item.HTMLID]; !ok {
				index.ByHTMLID[*item.HTMLID] = item
			}
		}
		if item.TestID != nil {
			if _, ok := index.ByTestID[*item.TestID]; !ok {
				index.ByTestID[*item.TestID] = item
			}
		}
		if item.Label != nil {
			label := normalizeQueryText(*item.Label)
			if label != "" {
				if _, ok := index.ByLabel[label]; !ok {
					index.ByLabel[label] = item
				}
			}
		}
		if item.Role != "" {
			index.ByRole[item.Role] = append(index.ByRole[item.Role], item)
		}
		for _, action := range item.Actions {
			if action != "" {
				index.ByAction[action] = append(index.ByAction[action], item)
			}
		}
	}
	return index
}

// FindActionTargetByID returns the compact action target matching a SOM element id.
func FindActionTargetByID(som *Som, id string) *ActionPlanItem {
	if item := FindActionTarget(som, id, "id"); item != nil {
		return item
	}
	return nil
}

// FindActionTarget returns the compact action target matching a replay id.
//
// The optional lookup key can be "auto", "id", "cache_key", "html_id", "test_id", or "label".
// Auto lookup checks SOM id, deterministic cache key, HTML id, then test id.
func FindActionTarget(som *Som, value string, by ...string) *ActionPlanItem {
	lookup := "auto"
	if len(by) > 0 && by[0] != "" {
		lookup = by[0]
	}
	index := GetActionPlanIndex(som)
	if lookup == "id" || lookup == "auto" {
		if item, ok := index.ByID[value]; ok {
			return &item
		}
	}
	if lookup == "cache_key" || lookup == "auto" {
		if item, ok := index.ByCacheKey[value]; ok {
			return &item
		}
	}
	if lookup == "html_id" || lookup == "auto" {
		if item, ok := index.ByHTMLID[value]; ok {
			return &item
		}
	}
	if lookup == "test_id" || lookup == "auto" {
		if item, ok := index.ByTestID[value]; ok {
			return &item
		}
	}
	if lookup == "label" {
		if matches := FindActionTargetsByLabel(som, value, true); len(matches) > 0 {
			return &matches[0]
		}
	}
	return nil
}

// FindActionTargetInIndex returns the compact action target matching a replay id in an existing index.
func FindActionTargetInIndex(index ActionPlanIndex, value string, by ...string) *ActionPlanItem {
	lookup := "auto"
	if len(by) > 0 && by[0] != "" {
		lookup = by[0]
	}
	if lookup == "id" || lookup == "auto" {
		if item, ok := index.ByID[value]; ok {
			return &item
		}
	}
	if lookup == "cache_key" || lookup == "auto" {
		if item, ok := index.ByCacheKey[value]; ok {
			return &item
		}
	}
	if lookup == "html_id" || lookup == "auto" {
		if item, ok := index.ByHTMLID[value]; ok {
			return &item
		}
	}
	if lookup == "test_id" || lookup == "auto" {
		if item, ok := index.ByTestID[value]; ok {
			return &item
		}
	}
	if lookup == "label" {
		if item, ok := index.ByLabel[normalizeQueryText(value)]; ok {
			return &item
		}
	}
	return nil
}

// FindActionTargetByCacheKey returns the compact action target matching a deterministic cache key.
func FindActionTargetByCacheKey(som *Som, cacheKey string) *ActionPlanItem {
	return FindActionTarget(som, cacheKey, "cache_key")
}

// FindActionTargetByHTMLID returns the compact action target matching an original HTML id.
func FindActionTargetByHTMLID(som *Som, htmlID string) *ActionPlanItem {
	return FindActionTarget(som, htmlID, "html_id")
}

// FindActionTargetByTestID returns the compact action target matching a test locator attribute.
func FindActionTargetByTestID(som *Som, testID string) *ActionPlanItem {
	return FindActionTarget(som, testID, "test_id")
}

// FindActionTargetByLabel returns the first compact action target matching an exact accessible label.
func FindActionTargetByLabel(som *Som, label string) *ActionPlanItem {
	if matches := FindActionTargetsByLabel(som, label, true); len(matches) > 0 {
		return &matches[0]
	}
	return nil
}

// FindActionTargetsByLabel returns compact action targets whose accessible label matches text.
func FindActionTargetsByLabel(som *Som, label string, exact bool, enabledOnly ...bool) []ActionPlanItem {
	plan := GetActionPlan(som)
	if len(enabledOnly) > 0 && enabledOnly[0] {
		plan = EnabledActionPlan(som)
	}
	results := []ActionPlanItem{}
	needle := normalizeQueryText(label)
	if needle == "" {
		return results
	}
	for _, item := range plan {
		if actionItemMatchesLabel(item, needle, exact) {
			results = append(results, item)
		}
	}
	return results
}

func actionItemMatchesLabel(item ActionPlanItem, needle string, exact bool) bool {
	matches := func(value *string) bool {
		if value == nil {
			return false
		}
		candidate := normalizeQueryText(*value)
		if exact {
			return candidate == needle
		}
		return strings.Contains(candidate, needle)
	}

	return matches(item.Label) ||
		matches(item.Title) ||
		matches(item.Placeholder) ||
		matches(item.Alt) ||
		matches(item.AriaLabel)
}

// FindActionTargetsByRole returns compact action targets whose SOM role matches
// a canonical role or common HTML/ARIA alias.
func FindActionTargetsByRole(som *Som, role string, enabledOnly ...bool) []ActionPlanItem {
	index := GetActionPlanIndex(som, enabledOnly...)
	return append([]ActionPlanItem(nil), index.ByRole[normalizeElementRoleAlias(role)]...)
}

// FindActionTargetsByAction returns compact action targets that expose the requested action.
func FindActionTargetsByAction(som *Som, action string, enabledOnly ...bool) []ActionPlanItem {
	index := GetActionPlanIndex(som, enabledOnly...)
	return append([]ActionPlanItem(nil), index.ByAction[action]...)
}

func flattenRegionElements(elements []Element) []Element {
	var result []Element
	flattenElements(elements, &result)
	return result
}

func copyFormContext(item *ActionPlanItem, region Region) {
	item.FormAction = region.Action
	item.FormMethod = region.Method
	item.FormTarget = region.Target
	item.FormEnctype = region.Enctype
	item.FormNoValidate = region.NoValidate
	item.FormAcceptCharset = region.AcceptCharset
	item.FormAutocomplete = region.Autocomplete
}

// TokenEstimate returns a rough estimate of the number of tokens in the SOM.
// Uses the heuristic of SOM bytes / 4, rounding up partial tokens.
func TokenEstimate(som *Som) int {
	somBytes := som.Meta.SOMBytes
	if somBytes == 0 {
		return 0
	}
	estimate := somBytes / 4
	if somBytes > 0 && somBytes%4 != 0 {
		estimate++
	}
	return estimate
}
