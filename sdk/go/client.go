package plasmate

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os/exec"
	"sync"
)

// SDKVersion is the independently published Go module version. Go releases
// use the submodule tag sdk/go/v<SDKVersion>.
const SDKVersion = "0.1.0"

// ClientOption configures a Client.
type ClientOption func(*Client)

// WithBinary sets the path to the plasmate binary. Default: "plasmate".
func WithBinary(path string) ClientOption {
	return func(c *Client) { c.binary = path }
}

// Client communicates with the plasmate MCP server over stdio using JSON-RPC 2.0.
type Client struct {
	binary string

	mu          sync.Mutex
	cmd         *exec.Cmd
	stdin       io.WriteCloser
	scanner     *bufio.Scanner
	nextID      int
	initialized bool
}

// NewClient creates a new Plasmate client. The subprocess is not started
// until the first API call.
func NewClient(opts ...ClientOption) *Client {
	c := &Client{
		binary: "plasmate",
		nextID: 1,
	}
	for _, opt := range opts {
		opt(c)
	}
	return c
}

// PageSession holds the result of opening an interactive page.
type PageSession struct {
	SessionID     string `json:"session_id"`
	Som           Som    `json:"som"`
	CacheRestored bool   `json:"cache_restored"`
}

// jsonRPCRequest is a JSON-RPC 2.0 request.
type jsonRPCRequest struct {
	JSONRPC string      `json:"jsonrpc"`
	ID      int         `json:"id"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params,omitempty"`
}

// jsonRPCResponse is a JSON-RPC 2.0 response.
type jsonRPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      *int            `json:"id"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *jsonRPCError   `json:"error,omitempty"`
}

type jsonRPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type toolResult struct {
	Content []toolContent `json:"content"`
	IsError bool          `json:"isError"`
}

type toolContent struct {
	Type string `json:"type"`
	Text string `json:"text"`
}

const maxToolErrorChars = 200

func boundedToolError(text string) string {
	if text == "" {
		return "unknown error"
	}
	runes := []rune(text)
	if len(runes) <= maxToolErrorChars {
		return text
	}
	return string(runes[:maxToolErrorChars-1]) + "…"
}

func (c *Client) ensureStarted() error {
	if c.initialized {
		return nil
	}
	return c.start()
}

func (c *Client) start() error {
	c.cmd = exec.Command(c.binary, "mcp")
	var err error
	c.stdin, err = c.cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("plasmate: stdin pipe: %w", err)
	}
	stdout, err := c.cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("plasmate: stdout pipe: %w", err)
	}
	c.scanner = bufio.NewScanner(stdout)
	c.scanner.Buffer(make([]byte, 0, 1024*1024), 10*1024*1024)

	if err := c.cmd.Start(); err != nil {
		return fmt.Errorf("plasmate: start: %w", err)
	}

	// Initialize MCP session.
	_, err = c.rpc("initialize", map[string]interface{}{
		"protocolVersion": "2024-11-05",
		"capabilities":    map[string]interface{}{},
		"clientInfo": map[string]interface{}{
			"name":    "plasmate-go-sdk",
			"version": SDKVersion,
		},
	})
	if err != nil {
		return fmt.Errorf("plasmate: initialize: %w", err)
	}

	// Send initialized notification.
	if err := c.send(jsonRPCRequest{
		JSONRPC: "2.0",
		ID:      c.nextID,
		Method:  "notifications/initialized",
	}); err != nil {
		return fmt.Errorf("plasmate: initialized notification: %w", err)
	}
	c.nextID++

	c.initialized = true
	return nil
}

// Close shuts down the plasmate subprocess.
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.initialized = false
	if c.stdin != nil {
		c.stdin.Close()
	}
	if c.cmd != nil && c.cmd.Process != nil {
		c.cmd.Process.Kill()
		c.cmd.Wait()
	}
	return nil
}

func (c *Client) send(req jsonRPCRequest) error {
	data, err := json.Marshal(req)
	if err != nil {
		return err
	}
	data = append(data, '\n')
	_, err = c.stdin.Write(data)
	return err
}

func (c *Client) readResponse(expectedID int) (*jsonRPCResponse, error) {
	for c.scanner.Scan() {
		line := c.scanner.Text()
		if line == "" {
			continue
		}
		var resp jsonRPCResponse
		if err := json.Unmarshal([]byte(line), &resp); err != nil {
			continue // skip non-JSON lines
		}
		if resp.ID == nil || *resp.ID != expectedID {
			continue // skip notifications and responses for other requests
		}
		return &resp, nil
	}
	if err := c.scanner.Err(); err != nil {
		return nil, fmt.Errorf("plasmate: read: %w", err)
	}
	return nil, errors.New("plasmate: process closed unexpectedly")
}

func (c *Client) rpc(method string, params interface{}) (json.RawMessage, error) {
	id := c.nextID
	c.nextID++

	req := jsonRPCRequest{
		JSONRPC: "2.0",
		ID:      id,
		Method:  method,
		Params:  params,
	}
	if err := c.send(req); err != nil {
		return nil, err
	}

	resp, err := c.readResponse(id)
	if err != nil {
		return nil, err
	}
	if resp.Error != nil {
		return nil, fmt.Errorf("plasmate: rpc %s: %s", method, resp.Error.Message)
	}
	return resp.Result, nil
}

func (c *Client) callTool(name string, args map[string]interface{}) (json.RawMessage, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if err := c.ensureStarted(); err != nil {
		return nil, err
	}

	raw, err := c.rpc("tools/call", map[string]interface{}{
		"name":      name,
		"arguments": args,
	})
	if err != nil {
		return nil, err
	}

	var result toolResult
	if err := json.Unmarshal(raw, &result); err != nil {
		return nil, fmt.Errorf("plasmate: unmarshal tool result: %w", err)
	}
	if result.IsError {
		if len(result.Content) == 0 {
			return nil, errors.New("unknown error")
		}
		return nil, errors.New(boundedToolError(result.Content[0].Text))
	}
	if len(result.Content) == 0 || result.Content[0].Text == "" {
		return nil, nil
	}
	return json.RawMessage(result.Content[0].Text), nil
}

// FetchPage fetches a page and returns its Semantic Object Model.
func (c *Client) FetchPage(url string) (*Som, error) {
	raw, err := c.callTool("fetch_page", map[string]interface{}{"url": url})
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

// FetchPageOptions holds optional parameters for FetchPageWithOptions.
type FetchPageOptions struct {
	Budget     *int    // Maximum output tokens
	JavaScript *bool   // Enable JS execution
	Selector   *string // Optional SOM region, role, action, or element-id filter
}

func fetchPageArguments(url string, opts FetchPageOptions) map[string]interface{} {
	args := map[string]interface{}{"url": url}
	if opts.Budget != nil {
		args["budget"] = *opts.Budget
	}
	if opts.JavaScript != nil {
		args["javascript"] = *opts.JavaScript
	}
	if opts.Selector != nil {
		args["selector"] = *opts.Selector
	}
	return args
}

// FetchPageWithOptions fetches a page with additional options.
func (c *Client) FetchPageWithOptions(url string, opts FetchPageOptions) (*Som, error) {
	args := fetchPageArguments(url, opts)
	raw, err := c.callTool("fetch_page", args)
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

// OpenPage opens a page in a persistent browser session.
func (c *Client) OpenPage(url string) (*PageSession, error) {
	return c.OpenPageWithOptions(url, OpenPageOptions{})
}

// OpenPageOptions holds optional parameters for OpenPageWithOptions.
type OpenPageOptions struct {
	Selector *string // Optional SOM region, role, action, or element-id filter
	Trace    *bool   // Opt in to bounded, privacy-safe in-memory action tracing
}

func openPageArguments(url string, opts OpenPageOptions) map[string]interface{} {
	args := map[string]interface{}{"url": url}
	if opts.Selector != nil {
		args["selector"] = *opts.Selector
	}
	if opts.Trace != nil {
		args["trace"] = *opts.Trace
	}
	return args
}

// OpenPageWithOptions opens a page in a persistent browser session with
// optional initial SOM filtering and action tracing.
func (c *Client) OpenPageWithOptions(url string, opts OpenPageOptions) (*PageSession, error) {
	raw, err := c.callTool("open_page", openPageArguments(url, opts))
	if err != nil {
		return nil, err
	}
	session, err := decodePageSession(raw)
	if err != nil {
		return nil, err
	}
	return session, nil
}

// NavigateTo navigates an existing browser session to a new URL and returns
// the updated page SOM.
func (c *Client) NavigateTo(sessionID string, url string) (*PageSession, error) {
	raw, err := c.callTool("navigate_to", map[string]interface{}{
		"session_id": sessionID,
		"url":        url,
	})
	if err != nil {
		return nil, err
	}
	return decodePageSession(raw)
}

func decodePageSession(raw []byte) (*PageSession, error) {
	var payload struct {
		PageSession
		// The MCP response currently exposes SOM fields alongside session_id.
		// Keep accepting a nested som for compatibility with older clients.
		SOMVersion     string          `json:"som_version"`
		URL            string          `json:"url"`
		Title          string          `json:"title"`
		Lang           string          `json:"lang"`
		Regions        []Region        `json:"regions"`
		Meta           SomMeta         `json:"meta"`
		StructuredData *StructuredData `json:"structured_data,omitempty"`
	}
	if err := json.Unmarshal(raw, &payload); err != nil {
		return nil, fmt.Errorf("plasmate: unmarshal session: %w", err)
	}
	flatSOM := payload.Regions != nil || payload.Meta != (SomMeta{}) ||
		payload.SOMVersion != "" || payload.URL != "" || payload.Title != "" ||
		payload.Lang != "" || payload.StructuredData != nil
	if flatSOM {
		som := payload.PageSession.Som
		if payload.SOMVersion != "" {
			som.SOMVersion = payload.SOMVersion
		}
		if payload.URL != "" {
			som.URL = payload.URL
		}
		if payload.Title != "" {
			som.Title = payload.Title
		}
		if payload.Lang != "" {
			som.Lang = payload.Lang
		}
		if payload.Regions != nil {
			som.Regions = payload.Regions
		}
		if payload.Meta != (SomMeta{}) {
			som.Meta = payload.Meta
		}
		if payload.StructuredData != nil {
			som.StructuredData = payload.StructuredData
		}
		payload.PageSession.Som = som
	}
	return &payload.PageSession, nil
}

// Evaluate executes JavaScript in the page context and returns the result as a string.
func (c *Client) Evaluate(sessionID string, expression string) (string, error) {
	raw, err := c.callTool("evaluate", map[string]interface{}{
		"session_id": sessionID,
		"expression": expression,
	})
	if err != nil {
		return "", err
	}
	if raw == nil {
		return "", nil
	}
	// Try to return as string; if it's a JSON string, unquote it.
	var s string
	if err := json.Unmarshal(raw, &s); err == nil {
		return s, nil
	}
	return string(raw), nil
}

// Click clicks an element by its SOM element ID and returns the updated SOM.
func (c *Client) Click(sessionID string, elementID string) (*Som, error) {
	raw, err := c.callTool("click", map[string]interface{}{
		"session_id": sessionID,
		"element_id": elementID,
	})
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

// TypeText types text into a form control and returns the updated page SOM.
func (c *Client) TypeText(sessionID string, elementID string, text string) (*Som, error) {
	return c.TypeTextWithOptions(sessionID, elementID, text, TypeTextOptions{})
}

// TypeTextOptions holds optional parameters for TypeTextWithOptions.
type TypeTextOptions struct {
	Append *bool // Append to the existing value instead of replacing it
}

func typeTextArguments(sessionID string, elementID string, text string, opts TypeTextOptions) map[string]interface{} {
	args := map[string]interface{}{
		"session_id": sessionID,
		"element_id": elementID,
		"text":       text,
	}
	if opts.Append != nil {
		args["append"] = *opts.Append
	}
	return args
}

// TypeTextWithOptions types text into a form control with optional append
// behavior and returns the updated page SOM.
func (c *Client) TypeTextWithOptions(sessionID string, elementID string, text string, opts TypeTextOptions) (*Som, error) {
	raw, err := c.callTool("type_text", typeTextArguments(sessionID, elementID, text, opts))
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

// Clear clears a text input or textarea and returns the updated page SOM.
func (c *Client) Clear(sessionID string, elementID string) (*Som, error) {
	raw, err := c.callTool("clear", clearArguments(sessionID, elementID))
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

func clearArguments(sessionID string, elementID string) map[string]interface{} {
	return map[string]interface{}{
		"session_id": sessionID,
		"element_id": elementID,
	}
}

// SelectOption selects an option in a dropdown or native radio group and
// returns the updated page SOM.
func (c *Client) SelectOption(sessionID string, elementID string, value string) (*Som, error) {
	raw, err := c.callTool("select_option", selectOptionArguments(sessionID, elementID, value))
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

func selectOptionArguments(sessionID string, elementID string, value string) map[string]interface{} {
	return map[string]interface{}{
		"session_id": sessionID,
		"element_id": elementID,
		"value":      value,
	}
}

// ClosePage closes a browser session and frees resources.
func (c *Client) ClosePage(sessionID string) error {
	_, err := c.callTool("close_page", map[string]interface{}{
		"session_id": sessionID,
	})
	return err
}

// ExtractTextOptions holds optional parameters for ExtractTextWithOptions.
type ExtractTextOptions struct {
	MaxChars   *int    // Maximum characters to return
	JavaScript *bool   // Enable JavaScript execution (defaults to true)
	Selector   *string // Optional SOM region, role, action, or element-id filter
}

func extractTextArguments(url string, opts ExtractTextOptions) map[string]interface{} {
	args := map[string]interface{}{"url": url}
	if opts.MaxChars != nil {
		args["max_chars"] = *opts.MaxChars
	}
	if opts.JavaScript != nil {
		args["javascript"] = *opts.JavaScript
	}
	if opts.Selector != nil {
		args["selector"] = *opts.Selector
	}
	return args
}

// ExtractText fetches a page and returns clean, readable text.
func (c *Client) ExtractText(url string) (string, error) {
	return c.ExtractTextWithOptions(url, ExtractTextOptions{})
}

// ExtractTextWithOptions fetches a page and returns clean, readable text with
// optional character and SOM selector limits.
func (c *Client) ExtractTextWithOptions(url string, opts ExtractTextOptions) (string, error) {
	raw, err := c.callTool("extract_text", extractTextArguments(url, opts))
	if err != nil {
		return "", err
	}
	if raw == nil {
		return "", nil
	}
	var s string
	if err := json.Unmarshal(raw, &s); err == nil {
		return s, nil
	}
	return string(raw), nil
}

// ExtractLinksOptions holds optional parameters for ExtractLinksWithOptions.
type ExtractLinksOptions struct {
	JavaScript *bool   // Enable JavaScript execution (defaults to true)
	MaxLinks   *int    // Maximum number of URLs to return
	Selector   *string // Optional SOM region, role, action, or element-id filter
}

func extractLinksArguments(url string, opts ExtractLinksOptions) map[string]interface{} {
	args := map[string]interface{}{"url": url}
	if opts.JavaScript != nil {
		args["javascript"] = *opts.JavaScript
	}
	if opts.MaxLinks != nil {
		args["max_links"] = *opts.MaxLinks
	}
	if opts.Selector != nil {
		args["selector"] = *opts.Selector
	}
	return args
}

// ExtractLinks fetches a page and returns deduplicated outbound URLs.
func (c *Client) ExtractLinks(url string) ([]string, error) {
	return c.ExtractLinksWithOptions(url, ExtractLinksOptions{})
}

// ExtractLinksWithOptions fetches links with optional JavaScript and SOM
// selector and result-count limits.
func (c *Client) ExtractLinksWithOptions(url string, opts ExtractLinksOptions) ([]string, error) {
	raw, err := c.callTool("extract_links", extractLinksArguments(url, opts))
	if err != nil {
		return nil, err
	}
	if raw == nil {
		return nil, nil
	}
	var links []string
	if err := json.Unmarshal(raw, &links); err != nil {
		return nil, fmt.Errorf("plasmate: unmarshal extracted links: %w", err)
	}
	return links, nil
}
