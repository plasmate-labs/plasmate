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
	var result []Region
	for _, r := range som.Regions {
		if r.Role == role {
			result = append(result, r)
		}
	}
	return result
}

// FindByID searches all regions for an element with the given ID.
// Returns nil if not found.
func FindByID(som *Som, id string) *Element {
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
	for _, el := range FlatElements(som) {
		if el.HTMLID != nil && *el.HTMLID == htmlID {
			return &el
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
	var result []Element
	for _, r := range som.Regions {
		collectByRole(r.Elements, tag, &result)
	}
	return result
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

// FindByText returns all elements whose text contains the given substring
// (case-insensitive). Labels are searched as well because many controls expose
// their human-facing text through label instead of text.
func FindByText(som *Som, text string) []Element {
	lower := strings.ToLower(text)
	var result []Element
	for _, r := range som.Regions {
		collectByText(r.Elements, lower, &result)
	}
	return result
}

// FindByTextExact returns all elements whose text or label exactly matches text.
// Exact matches are case-sensitive.
func FindByTextExact(som *Som, text string) []Element {
	var result []Element
	for _, r := range som.Regions {
		collectByTextExact(r.Elements, text, &result)
	}
	return result
}

func collectByText(elements []Element, lowerText string, result *[]Element) {
	for _, el := range elements {
		if el.Text != nil && strings.Contains(strings.ToLower(*el.Text), lowerText) {
			*result = append(*result, el)
		} else if el.Label != nil && strings.Contains(strings.ToLower(*el.Label), lowerText) {
			*result = append(*result, el)
		}
		collectByText(el.Children, lowerText, result)
		if el.Shadow != nil {
			collectByText(el.Shadow.Elements, lowerText, result)
		}
	}
}

func collectByTextExact(elements []Element, text string, result *[]Element) {
	for _, el := range elements {
		if el.Text != nil && *el.Text == text {
			*result = append(*result, el)
		} else if el.Label != nil && *el.Label == text {
			*result = append(*result, el)
		}
		collectByTextExact(el.Children, text, result)
		if el.Shadow != nil {
			collectByTextExact(el.Shadow.Elements, text, result)
		}
	}
}

// FindByAction returns all elements that expose a specific action.
func FindByAction(som *Som, action string) []Element {
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
	ID                string         `json:"id"`
	CacheKey          string         `json:"cache_key"`
	Role              string         `json:"role"`
	Actions           []string       `json:"actions"`
	Enabled           bool           `json:"enabled"`
	HTMLID            *string        `json:"html_id,omitempty"`
	Label             *string        `json:"label,omitempty"`
	Href              *string        `json:"href,omitempty"`
	Target            *string        `json:"target,omitempty"`
	Rel               *string        `json:"rel,omitempty"`
	Download          interface{}    `json:"download,omitempty"`
	Alt               *string        `json:"alt,omitempty"`
	Src               *string        `json:"src,omitempty"`
	Name              *string        `json:"name,omitempty"`
	Accept            *string        `json:"accept,omitempty"`
	Capture           interface{}    `json:"capture,omitempty"`
	Multiple          *bool          `json:"multiple,omitempty"`
	Options           []SelectOption `json:"options,omitempty"`
	SelectedValues    []string       `json:"selected_values,omitempty"`
	Size              interface{}    `json:"size,omitempty"`
	Autocomplete      *string        `json:"autocomplete,omitempty"`
	InputMode         *string        `json:"inputmode,omitempty"`
	EnterKeyHint      *string        `json:"enterkeyhint,omitempty"`
	AutoCapitalize    *string        `json:"autocapitalize,omitempty"`
	DirName           *string        `json:"dirname,omitempty"`
	Dir               *string        `json:"dir,omitempty"`
	Lang              *string        `json:"lang,omitempty"`
	Form              *string        `json:"form,omitempty"`
	FormAction        *string        `json:"form_action,omitempty"`
	FormMethod        *string        `json:"form_method,omitempty"`
	FormTarget        *string        `json:"form_target,omitempty"`
	FormEnctype       *string        `json:"form_enctype,omitempty"`
	FormNoValidate    *bool          `json:"form_novalidate,omitempty"`
	FormAcceptCharset *string        `json:"form_accept_charset,omitempty"`
	FormAutocomplete  *string        `json:"form_autocomplete,omitempty"`
	List              *string        `json:"list,omitempty"`
	PopoverTarget     *string        `json:"popovertarget,omitempty"`
	PopoverAction     *string        `json:"popovertargetaction,omitempty"`
	CommandFor        *string        `json:"commandfor,omitempty"`
	Command           *string        `json:"command,omitempty"`
	Popover           *string        `json:"popover,omitempty"`
	ButtonType        *string        `json:"button_type,omitempty"`
	SubmitFormAction  *string        `json:"formaction,omitempty"`
	SubmitFormMethod  *string        `json:"formmethod,omitempty"`
	SubmitFormEnctype *string        `json:"formenctype,omitempty"`
	SubmitFormTarget  *string        `json:"formtarget,omitempty"`
	SubmitNoValidate  *bool          `json:"formnovalidate,omitempty"`
	AccessKey         *string        `json:"accesskey,omitempty"`
	Title             *string        `json:"title,omitempty"`
	AriaLabel         *string        `json:"aria_label,omitempty"`
	AriaDescription   *string        `json:"aria_description,omitempty"`
	LabelledBy        *string        `json:"labelledby,omitempty"`
	DescribedBy       *string        `json:"describedby,omitempty"`
	Spellcheck        interface{}    `json:"spellcheck,omitempty"`
	InputType         *string        `json:"input_type,omitempty"`
	Value             *string        `json:"value,omitempty"`
	Placeholder       *string        `json:"placeholder,omitempty"`
	MinLength         interface{}    `json:"minlength,omitempty"`
	MaxLength         interface{}    `json:"maxlength,omitempty"`
	Min               interface{}    `json:"min,omitempty"`
	Max               interface{}    `json:"max,omitempty"`
	Step              *string        `json:"step,omitempty"`
	Pattern           *string        `json:"pattern,omitempty"`
	Description       *string        `json:"description,omitempty"`
	TestID            *string        `json:"test_id,omitempty"`
	DataAction        *string        `json:"data_action,omitempty"`
	DataState         *string        `json:"data_state,omitempty"`
	Checked           interface{}    `json:"checked,omitempty"`
	Expanded          *bool          `json:"expanded,omitempty"`
	Pressed           *bool          `json:"pressed,omitempty"`
	Selected          *bool          `json:"selected,omitempty"`
	Multiline         *bool          `json:"multiline,omitempty"`
	MultiSelectable   *bool          `json:"multiselectable,omitempty"`
	Current           interface{}    `json:"current,omitempty"`
	Controls          *string        `json:"controls,omitempty"`
	HasPopup          interface{}    `json:"haspopup,omitempty"`
	Invalid           interface{}    `json:"invalid,omitempty"`
	AriaPlaceholder   *string        `json:"aria_placeholder,omitempty"`
	AriaAutocomplete  *string        `json:"aria_autocomplete,omitempty"`
	ActiveDescendant  *string        `json:"active_descendant,omitempty"`
	ErrorMessage      *string        `json:"errormessage,omitempty"`
	KeyShortcuts      *string        `json:"keyshortcuts,omitempty"`
	RoleDescription   *string        `json:"roledescription,omitempty"`
	Busy              *bool          `json:"busy,omitempty"`
	Live              *string        `json:"live,omitempty"`
	Atomic            *bool          `json:"atomic,omitempty"`
	Relevant          *string        `json:"relevant,omitempty"`
	Owns              *string        `json:"owns,omitempty"`
	FlowTo            *string        `json:"flowto,omitempty"`
	Details           *string        `json:"details,omitempty"`
	Orientation       *string        `json:"orientation,omitempty"`
	Sort              *string        `json:"sort,omitempty"`
	Level             *string        `json:"level,omitempty"`
	PosInSet          *string        `json:"posinset,omitempty"`
	SetSize           *string        `json:"setsize,omitempty"`
	ValueMin          *string        `json:"valuemin,omitempty"`
	ValueMax          *string        `json:"valuemax,omitempty"`
	ValueNow          *string        `json:"valuenow,omitempty"`
	ValueText         *string        `json:"valuetext,omitempty"`
	Required          *bool          `json:"required,omitempty"`
	Readonly          *bool          `json:"readonly,omitempty"`
	Disabled          *bool          `json:"disabled,omitempty"`
	Inert             *bool          `json:"inert,omitempty"`
	BlockedReason     *string        `json:"blocked_reason,omitempty"`
	Group             *string        `json:"group,omitempty"`
}

// ActionPlanIndex groups compact action targets for O(1) replay lookups.
type ActionPlanIndex struct {
	ByID               map[string]ActionPlanItem   `json:"by_id"`
	ByCacheKey         map[string]ActionPlanItem   `json:"by_cache_key"`
	ByCacheKeyAll      map[string][]ActionPlanItem `json:"by_cache_key_all"`
	ByHTMLID           map[string]ActionPlanItem   `json:"by_html_id"`
	ByHTMLIDAll        map[string][]ActionPlanItem `json:"by_html_id_all"`
	DuplicateCacheKeys []string                    `json:"duplicate_cache_keys"`
	DuplicateHTMLIDs   []string                    `json:"duplicate_html_ids"`
}

// ActionPlanSummary gives callers a compact replay-validation view of a plan.
type ActionPlanSummary struct {
	Fingerprint        string         `json:"fingerprint"`
	EnabledFingerprint string         `json:"enabled_fingerprint"`
	Total              int            `json:"total"`
	Enabled            int            `json:"enabled"`
	Disabled           int            `json:"disabled"`
	WithCacheKey       int            `json:"with_cache_key"`
	UniqueCacheKeys    int            `json:"unique_cache_keys"`
	DuplicateCacheKeys []string       `json:"duplicate_cache_keys"`
	WithHTMLID         int            `json:"with_html_id"`
	DuplicateHTMLIDs   []string       `json:"duplicate_html_ids"`
	WithTestID         int            `json:"with_test_id"`
	WithDataAction     int            `json:"with_data_action"`
	WithDataState      int            `json:"with_data_state"`
	ByRole             map[string]int `json:"by_role"`
	BlockedReasons     map[string]int `json:"blocked_reasons"`
}

func compactString(value *string) interface{} {
	if value != nil && *value != "" {
		return *value
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
	parts := []interface{}{
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
	for _, value := range []interface{}{compactString(item.TestID), compactString(item.DataAction)} {
		if value != nil {
			parts = append(parts, value)
		}
	}
	return parts
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
			HTMLID:  el.HTMLID,
		}
		if el.Label != nil {
			item.Label = el.Label
		} else if el.Text != nil {
			item.Label = el.Text
		}
		if el.Attrs != nil {
			item.Href = el.Attrs.Href
			item.Target = el.Attrs.Target
			item.Rel = el.Attrs.Rel
			item.Download = el.Attrs.Download
			item.Alt = el.Attrs.Alt
			item.Src = el.Attrs.Src
			item.Name = el.Attrs.Name
			item.Accept = el.Attrs.Accept
			item.Capture = el.Attrs.Capture
			item.Multiple = el.Attrs.Multiple
			item.Options = el.Attrs.Options
			item.SelectedValues = el.Attrs.SelectedValues
			item.Size = el.Attrs.Size
			item.Autocomplete = el.Attrs.Autocomplete
			item.InputMode = el.Attrs.InputMode
			item.EnterKeyHint = el.Attrs.EnterKeyHint
			item.AutoCapitalize = el.Attrs.AutoCapitalize
			item.DirName = el.Attrs.DirName
			item.Dir = el.Attrs.Dir
			item.Lang = el.Attrs.Lang
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
			item.AriaLabel = el.Attrs.AriaLabel
			item.AriaDescription = el.Attrs.AriaDescription
			item.LabelledBy = el.Attrs.LabelledBy
			item.DescribedBy = el.Attrs.DescribedBy
			item.Spellcheck = el.Attrs.Spellcheck
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
			item.TestID = el.Attrs.TestID
			item.DataAction = el.Attrs.DataAction
			item.DataState = el.Attrs.DataState
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

// FindActionTargetByCacheKey returns the compact action target matching a deterministic cache key.
// Pass true as the optional third argument to search only currently enabled targets.
func FindActionTargetByCacheKey(som *Som, cacheKey string, enabledOnly ...bool) *ActionPlanItem {
	plan := GetActionPlan(som)
	if len(enabledOnly) > 0 && enabledOnly[0] {
		plan = EnabledActionPlan(som)
	}
	for _, item := range plan {
		if item.CacheKey == cacheKey {
			matched := item
			return &matched
		}
	}
	return nil
}

// EnabledActionPlan returns compact action targets that are currently available.
func EnabledActionPlan(som *Som) []ActionPlanItem {
	var result []ActionPlanItem
	for _, item := range GetActionPlan(som) {
		if item.Enabled {
			result = append(result, item)
		}
	}
	return result
}

// GetActionPlanIndex returns action targets indexed by SOM id, cache key, and original HTML id.
func GetActionPlanIndex(som *Som, enabledOnly ...bool) ActionPlanIndex {
	onlyEnabled := len(enabledOnly) > 0 && enabledOnly[0]
	plan := GetActionPlan(som)
	if onlyEnabled {
		plan = EnabledActionPlan(som)
	}
	index := ActionPlanIndex{
		ByID:          map[string]ActionPlanItem{},
		ByCacheKey:    map[string]ActionPlanItem{},
		ByCacheKeyAll: map[string][]ActionPlanItem{},
		ByHTMLID:      map[string]ActionPlanItem{},
		ByHTMLIDAll:   map[string][]ActionPlanItem{},
	}
	for _, item := range plan {
		if _, ok := index.ByID[item.ID]; !ok {
			index.ByID[item.ID] = item
		}
		if _, ok := index.ByCacheKey[item.CacheKey]; !ok {
			index.ByCacheKey[item.CacheKey] = item
		}
		index.ByCacheKeyAll[item.CacheKey] = append(index.ByCacheKeyAll[item.CacheKey], item)
		if item.HTMLID != nil {
			if _, ok := index.ByHTMLID[*item.HTMLID]; !ok {
				index.ByHTMLID[*item.HTMLID] = item
			}
			index.ByHTMLIDAll[*item.HTMLID] = append(index.ByHTMLIDAll[*item.HTMLID], item)
		}
	}
	for cacheKey, items := range index.ByCacheKeyAll {
		if len(items) > 1 {
			index.DuplicateCacheKeys = append(index.DuplicateCacheKeys, cacheKey)
		}
	}
	sort.Strings(index.DuplicateCacheKeys)
	for htmlID, items := range index.ByHTMLIDAll {
		if len(items) > 1 {
			index.DuplicateHTMLIDs = append(index.DuplicateHTMLIDs, htmlID)
		}
	}
	sort.Strings(index.DuplicateHTMLIDs)
	return index
}

// GetActionPlanFingerprint returns a deterministic fingerprint for the current compact action plan.
func GetActionPlanFingerprint(som *Som, enabledOnly ...bool) string {
	onlyEnabled := len(enabledOnly) > 0 && enabledOnly[0]
	plan := GetActionPlan(som)
	if onlyEnabled {
		plan = EnabledActionPlan(som)
	}
	rows := make([][]interface{}, 0, len(plan))
	for _, item := range plan {
		rows = append(rows, []interface{}{item.CacheKey, item.Enabled, item.BlockedReason})
	}
	sort.Slice(rows, func(i, j int) bool {
		return rows[i][0].(string) < rows[j][0].(string)
	})
	encoded, err := json.Marshal(rows)
	if err != nil {
		return "plasmate-plan:v1:00000000"
	}
	hash := fnv.New32a()
	_, _ = hash.Write(encoded)
	return fmt.Sprintf("plasmate-plan:v1:%08x", hash.Sum32())
}

// GetActionPlanSummary returns compact action-plan counts and fingerprints for replay validation.
func GetActionPlanSummary(som *Som) ActionPlanSummary {
	plan := GetActionPlan(som)
	summary := ActionPlanSummary{
		Fingerprint:        GetActionPlanFingerprint(som),
		EnabledFingerprint: GetActionPlanFingerprint(som, true),
		Total:              len(plan),
		ByRole:             map[string]int{},
		BlockedReasons:     map[string]int{},
	}
	cacheKeyCounts := map[string]int{}
	htmlIDCounts := map[string]int{}
	for _, item := range plan {
		summary.ByRole[item.Role]++
		if item.CacheKey != "" {
			summary.WithCacheKey++
			cacheKeyCounts[item.CacheKey]++
		}
		if item.HTMLID != nil && *item.HTMLID != "" {
			summary.WithHTMLID++
			htmlIDCounts[*item.HTMLID]++
		}
		if item.TestID != nil && *item.TestID != "" {
			summary.WithTestID++
		}
		if item.DataAction != nil && *item.DataAction != "" {
			summary.WithDataAction++
		}
		if item.DataState != nil && *item.DataState != "" {
			summary.WithDataState++
		}
		if item.Enabled {
			summary.Enabled++
		} else {
			summary.Disabled++
			reason := "unknown"
			if item.BlockedReason != nil && *item.BlockedReason != "" {
				reason = *item.BlockedReason
			}
			summary.BlockedReasons[reason]++
		}
	}
	summary.UniqueCacheKeys = len(cacheKeyCounts)
	for cacheKey, count := range cacheKeyCounts {
		if count > 1 {
			summary.DuplicateCacheKeys = append(summary.DuplicateCacheKeys, cacheKey)
		}
	}
	sort.Strings(summary.DuplicateCacheKeys)
	for htmlID, count := range htmlIDCounts {
		if count > 1 {
			summary.DuplicateHTMLIDs = append(summary.DuplicateHTMLIDs, htmlID)
		}
	}
	sort.Strings(summary.DuplicateHTMLIDs)
	return summary
}

// FindActionTargetByID returns the compact action target matching a SOM element id.
// Pass true as the optional third argument to search only currently enabled targets.
func FindActionTargetByID(som *Som, id string, enabledOnly ...bool) *ActionPlanItem {
	plan := GetActionPlan(som)
	if len(enabledOnly) > 0 && enabledOnly[0] {
		plan = EnabledActionPlan(som)
	}
	for _, item := range plan {
		if item.ID == id {
			matched := item
			return &matched
		}
	}
	return nil
}

// FindActionTargetByHTMLID returns the compact action target matching an original HTML id.
// Pass true as the optional third argument to search only currently enabled targets.
func FindActionTargetByHTMLID(som *Som, htmlID string, enabledOnly ...bool) *ActionPlanItem {
	plan := GetActionPlan(som)
	if len(enabledOnly) > 0 && enabledOnly[0] {
		plan = EnabledActionPlan(som)
	}
	for _, item := range plan {
		if item.HTMLID != nil && *item.HTMLID == htmlID {
			matched := item
			return &matched
		}
	}
	return nil
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
// Uses the heuristic of SOM bytes / 4.
func TokenEstimate(som *Som) int {
	if som.Meta.SOMBytes == 0 {
		return 0
	}
	return som.Meta.SOMBytes / 4
}
