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
	ID            string    `json:"id"`
	Role          string    `json:"role"`
	Label         *string   `json:"label,omitempty"`
	Action        *string   `json:"action,omitempty"`
	Method        *string   `json:"method,omitempty"`
	Target        *string   `json:"target,omitempty"`
	Enctype       *string   `json:"enctype,omitempty"`
	NoValidate    *bool     `json:"novalidate,omitempty"`
	AcceptCharset *string   `json:"accept_charset,omitempty"`
	Autocomplete  *string   `json:"autocomplete,omitempty"`
	Elements      []Element `json:"elements"`
}

// Element represents a semantic element within a region.
type Element struct {
	ID       string         `json:"id"`
	Role     string         `json:"role"`
	HTMLID   *string        `json:"html_id,omitempty"`
	Text     *string        `json:"text,omitempty"`
	Label    *string        `json:"label,omitempty"`
	Actions  []string       `json:"actions,omitempty"`
	Attrs    *ElementAttrs  `json:"attrs,omitempty"`
	Children []Element      `json:"children,omitempty"`
	Hints    []string       `json:"hints,omitempty"`
	Shadow   *SomShadowRoot `json:"shadow,omitempty"`
}

// ElementAttrs holds role-specific attributes for an element.
type ElementAttrs struct {
	Href            *string        `json:"href,omitempty"`
	Target          *string        `json:"target,omitempty"`
	Rel             *string        `json:"rel,omitempty"`
	HrefLang        *string        `json:"hreflang,omitempty"`
	Type            *string        `json:"type,omitempty"`
	ReferrerPolicy  *string        `json:"referrerpolicy,omitempty"`
	Download        interface{}    `json:"download,omitempty"`
	InputType       *string        `json:"input_type,omitempty"`
	Value           *string        `json:"value,omitempty"`
	Placeholder     *string        `json:"placeholder,omitempty"`
	Required        *bool          `json:"required,omitempty"`
	Readonly        *bool          `json:"readonly,omitempty"`
	Disabled        *bool          `json:"disabled,omitempty"`
	Inert           *bool          `json:"inert,omitempty"`
	Checked         *bool          `json:"checked,omitempty"`
	Group           *string        `json:"group,omitempty"`
	Multiple        *bool          `json:"multiple,omitempty"`
	Options         []SelectOption `json:"options,omitempty"`
	SelectedValues  []string       `json:"selected_values,omitempty"`
	Size            interface{}    `json:"size,omitempty"`
	Level           *int           `json:"level,omitempty"`
	Alt             *string        `json:"alt,omitempty"`
	Src             *string        `json:"src,omitempty"`
	Ordered         *bool          `json:"ordered,omitempty"`
	Items           []ListItem     `json:"items,omitempty"`
	Headers         []string       `json:"headers,omitempty"`
	Rows            [][]string     `json:"rows,omitempty"`
	SectionLabel    *string        `json:"section_label,omitempty"`
	Legend          *string        `json:"legend,omitempty"`
	Open            *bool          `json:"open,omitempty"`
	Summary         *string        `json:"summary,omitempty"`
	ContentEditable interface{}    `json:"contenteditable,omitempty"`
	TabIndex        interface{}    `json:"tabindex,omitempty"`
	AccessKey       *string        `json:"accesskey,omitempty"`
	Title           *string        `json:"title,omitempty"`
	SourceRole      *string        `json:"source_role,omitempty"`
	TestID          *string        `json:"test_id,omitempty"`
	Spellcheck      interface{}    `json:"spellcheck,omitempty"`
	Draggable       interface{}    `json:"draggable,omitempty"`
	Name            *string        `json:"name,omitempty"`
	Accept          *string        `json:"accept,omitempty"`
	Capture         interface{}    `json:"capture,omitempty"`
	Autocomplete    *string        `json:"autocomplete,omitempty"`
	InputMode       *string        `json:"inputmode,omitempty"`
	EnterKeyHint    *string        `json:"enterkeyhint,omitempty"`
	AutoCapitalize  *string        `json:"autocapitalize,omitempty"`
	DirName         *string        `json:"dirname,omitempty"`
	Form            *string        `json:"form,omitempty"`
	List            *string        `json:"list,omitempty"`
	PopoverTarget   *string        `json:"popovertarget,omitempty"`
	PopoverAction   *string        `json:"popovertargetaction,omitempty"`
	CommandFor      *string        `json:"commandfor,omitempty"`
	Command         *string        `json:"command,omitempty"`
	Popover         *string        `json:"popover,omitempty"`
	ButtonType      *string        `json:"button_type,omitempty"`
	FormAction      *string        `json:"formaction,omitempty"`
	FormMethod      *string        `json:"formmethod,omitempty"`
	FormEnctype     *string        `json:"formenctype,omitempty"`
	FormTarget      *string        `json:"formtarget,omitempty"`
	FormNoValidate  *bool          `json:"formnovalidate,omitempty"`
	MinLength       interface{}    `json:"minlength,omitempty"`
	MaxLength       interface{}    `json:"maxlength,omitempty"`
	Min             interface{}    `json:"min,omitempty"`
	Max             interface{}    `json:"max,omitempty"`
	Step            *string        `json:"step,omitempty"`
	Pattern         *string        `json:"pattern,omitempty"`
	Description     *string        `json:"description,omitempty"`
	Aria            *AriaState     `json:"aria,omitempty"`
	HasSrcdoc       *bool          `json:"has_srcdoc,omitempty"`
	SrcdocPreview   *string        `json:"srcdoc_preview,omitempty"`
	Sandbox         *string        `json:"sandbox,omitempty"`
	Allow           *string        `json:"allow,omitempty"`
	Width           *string        `json:"width,omitempty"`
	Height          *string        `json:"height,omitempty"`
}

// AriaState holds common ARIA state attributes emitted by SOM.
type AriaState struct {
	Expanded         *bool       `json:"expanded,omitempty"`
	Selected         *bool       `json:"selected,omitempty"`
	Checked          interface{} `json:"checked,omitempty"`
	Readonly         *bool       `json:"readonly,omitempty"`
	Disabled         *bool       `json:"disabled,omitempty"`
	Current          interface{} `json:"current,omitempty"`
	Pressed          *bool       `json:"pressed,omitempty"`
	Hidden           *bool       `json:"hidden,omitempty"`
	Controls         *string     `json:"controls,omitempty"`
	HasPopup         interface{} `json:"haspopup,omitempty"`
	Invalid          interface{} `json:"invalid,omitempty"`
	Placeholder      *string     `json:"placeholder,omitempty"`
	Autocomplete     *string     `json:"autocomplete,omitempty"`
	ActiveDescendant *string     `json:"active_descendant,omitempty"`
	ErrorMessage     *string     `json:"errormessage,omitempty"`
	KeyShortcuts     *string     `json:"keyshortcuts,omitempty"`
	RoleDescription  *string     `json:"roledescription,omitempty"`
	Busy             *bool       `json:"busy,omitempty"`
	Live             *string     `json:"live,omitempty"`
	Atomic           *bool       `json:"atomic,omitempty"`
	Relevant         *string     `json:"relevant,omitempty"`
	Owns             *string     `json:"owns,omitempty"`
	FlowTo           *string     `json:"flowto,omitempty"`
	Details          *string     `json:"details,omitempty"`
	Multiline        *bool       `json:"multiline,omitempty"`
	MultiSelectable  *bool       `json:"multiselectable,omitempty"`
	Orientation      *string     `json:"orientation,omitempty"`
	Sort             *string     `json:"sort,omitempty"`
	Level            *string     `json:"level,omitempty"`
	PosInSet         *string     `json:"posinset,omitempty"`
	SetSize          *string     `json:"setsize,omitempty"`
	Grabbed          *bool       `json:"grabbed,omitempty"`
	DropEffect       *string     `json:"dropeffect,omitempty"`
	ValueMin         *string     `json:"valuemin,omitempty"`
	ValueMax         *string     `json:"valuemax,omitempty"`
	ValueNow         *string     `json:"valuenow,omitempty"`
	ValueText        *string     `json:"valuetext,omitempty"`
	Label            *string     `json:"label,omitempty"`
	LabelledBy       *string     `json:"labelledby,omitempty"`
	DescribedBy      *string     `json:"describedby,omitempty"`
}

// SomShadowRoot represents elements inside a web component shadow root.
type SomShadowRoot struct {
	Mode     string    `json:"mode"`
	Elements []Element `json:"elements"`
}

// SelectOption represents an option within a select element.
type SelectOption struct {
	Value    string  `json:"value"`
	Text     string  `json:"text"`
	Selected *bool   `json:"selected,omitempty"`
	Disabled *bool   `json:"disabled,omitempty"`
	Group    *string `json:"group,omitempty"`
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
