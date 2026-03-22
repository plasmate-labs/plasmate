// Package plasmate provides a Go SDK for the Plasmate agent-native headless browser.
//
// It communicates with the `plasmate mcp` process over stdio using JSON-RPC 2.0
// and provides typed access to the Semantic Object Model (SOM).
package plasmate

import "encoding/json"

// Som represents a complete Semantic Object Model document.
type Som struct {
	SOMVersion     string          `json:"som_version"`
	URL            string          `json:"url"`
	Title          string          `json:"title"`
	Lang           string          `json:"lang"`
	Regions        []Region        `json:"regions"`
	Meta           SomMeta         `json:"meta"`
	StructuredData *StructuredData `json:"structured_data,omitempty"`
}

// Region represents a semantic region within the page.
type Region struct {
	ID       string    `json:"id"`
	Role     string    `json:"role"`
	Label    *string   `json:"label,omitempty"`
	Action   *string   `json:"action,omitempty"`
	Method   *string   `json:"method,omitempty"`
	Elements []Element `json:"elements"`
}

// Element represents a semantic element within a region.
type Element struct {
	ID       string         `json:"id"`
	Role     string         `json:"role"`
	Text     *string        `json:"text,omitempty"`
	Label    *string        `json:"label,omitempty"`
	Actions  []string       `json:"actions,omitempty"`
	Attrs    *ElementAttrs  `json:"attrs,omitempty"`
	Children []Element      `json:"children,omitempty"`
	Hints    []string       `json:"hints,omitempty"`
}

// ElementAttrs holds role-specific attributes for an element.
type ElementAttrs struct {
	Href         *string        `json:"href,omitempty"`
	InputType    *string        `json:"input_type,omitempty"`
	Value        *string        `json:"value,omitempty"`
	Placeholder  *string        `json:"placeholder,omitempty"`
	Required     *bool          `json:"required,omitempty"`
	Disabled     *bool          `json:"disabled,omitempty"`
	Checked      *bool          `json:"checked,omitempty"`
	Group        *string        `json:"group,omitempty"`
	Multiple     *bool          `json:"multiple,omitempty"`
	Options      []SelectOption `json:"options,omitempty"`
	Level        *int           `json:"level,omitempty"`
	Alt          *string        `json:"alt,omitempty"`
	Src          *string        `json:"src,omitempty"`
	Ordered      *bool          `json:"ordered,omitempty"`
	Items        []ListItem     `json:"items,omitempty"`
	Headers      []string       `json:"headers,omitempty"`
	Rows         [][]string     `json:"rows,omitempty"`
	SectionLabel *string        `json:"section_label,omitempty"`
}

// SelectOption represents an option within a select element.
type SelectOption struct {
	Value    string `json:"value"`
	Text     string `json:"text"`
	Selected *bool  `json:"selected,omitempty"`
}

// ListItem represents an item within a list element.
type ListItem struct {
	Text string `json:"text"`
}

// SomMeta contains metadata about the SOM compilation.
type SomMeta struct {
	HTMLBytes        int `json:"html_bytes"`
	SOMBytes         int `json:"som_bytes"`
	ElementCount     int `json:"element_count"`
	InteractiveCount int `json:"interactive_count"`
}

// StructuredData holds structured data extracted from the page head.
type StructuredData struct {
	JSONLD      []map[string]interface{} `json:"json_ld,omitempty"`
	OpenGraph   map[string]string        `json:"open_graph,omitempty"`
	TwitterCard map[string]string        `json:"twitter_card,omitempty"`
	Meta        map[string]string        `json:"meta,omitempty"`
	Links       []LinkElement            `json:"links,omitempty"`
}

// LinkElement represents a semantically meaningful <link> element.
type LinkElement struct {
	Rel      string  `json:"rel"`
	Href     string  `json:"href"`
	Type     *string `json:"type,omitempty"`
	Hreflang *string `json:"hreflang,omitempty"`
}

// Parse unmarshals JSON data into a Som struct.
func Parse(data []byte) (*Som, error) {
	var som Som
	if err := json.Unmarshal(data, &som); err != nil {
		return nil, err
	}
	return &som, nil
}
