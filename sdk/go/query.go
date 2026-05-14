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
	ID                string      `json:"id"`
	CacheKey          string      `json:"cache_key"`
	Role              string      `json:"role"`
	Actions           []string    `json:"actions"`
	Enabled           bool        `json:"enabled"`
	Label             *string     `json:"label,omitempty"`
	Href              *string     `json:"href,omitempty"`
	Target            *string     `json:"target,omitempty"`
	Rel               *string     `json:"rel,omitempty"`
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
	Autofocus         *bool       `json:"autofocus,omitempty"`
	Spellcheck        interface{} `json:"spellcheck,omitempty"`
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
			item.Autofocus = el.Attrs.Autofocus
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
