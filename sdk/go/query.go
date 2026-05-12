package plasmate

import "strings"

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
	ID          string   `json:"id"`
	Role        string   `json:"role"`
	Actions     []string `json:"actions"`
	Label       *string  `json:"label,omitempty"`
	Href        *string  `json:"href,omitempty"`
	Name        *string  `json:"name,omitempty"`
	InputType   *string  `json:"input_type,omitempty"`
	Placeholder *string  `json:"placeholder,omitempty"`
	Description *string  `json:"description,omitempty"`
	Required    *bool    `json:"required,omitempty"`
	Disabled    *bool    `json:"disabled,omitempty"`
	Group       *string  `json:"group,omitempty"`
}

// GetActionPlan returns compact action targets for agent planning.
func GetActionPlan(som *Som) []ActionPlanItem {
	items := []ActionPlanItem{}
	for _, el := range FindInteractive(som) {
		item := ActionPlanItem{
			ID:      el.ID,
			Role:    el.Role,
			Actions: append([]string(nil), el.Actions...),
		}
		if el.Label != nil {
			item.Label = el.Label
		} else if el.Text != nil {
			item.Label = el.Text
		}
		if el.Attrs != nil {
			item.Href = el.Attrs.Href
			item.Name = el.Attrs.Name
			item.InputType = el.Attrs.InputType
			item.Placeholder = el.Attrs.Placeholder
			item.Description = el.Attrs.Description
			item.Required = el.Attrs.Required
			item.Disabled = el.Attrs.Disabled
			item.Group = el.Attrs.Group
		}
		items = append(items, item)
	}
	return items
}

// TokenEstimate returns a rough estimate of the number of tokens in the SOM.
// Uses the heuristic of SOM bytes / 4.
func TokenEstimate(som *Som) int {
	if som.Meta.SOMBytes == 0 {
		return 0
	}
	return som.Meta.SOMBytes / 4
}
