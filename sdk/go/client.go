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
	SessionID string `json:"session_id"`
	Som       Som    `json:"som"`
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
	JSONRPC string           `json:"jsonrpc"`
	ID      *int             `json:"id"`
	Result  json.RawMessage  `json:"result,omitempty"`
	Error   *jsonRPCError    `json:"error,omitempty"`
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
			"version": "0.1.0",
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

func (c *Client) readResponse() (*jsonRPCResponse, error) {
	for c.scanner.Scan() {
		line := c.scanner.Text()
		if line == "" {
			continue
		}
		var resp jsonRPCResponse
		if err := json.Unmarshal([]byte(line), &resp); err != nil {
			continue // skip non-JSON lines
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

	resp, err := c.readResponse()
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
		msg := "unknown error"
		if len(result.Content) > 0 {
			msg = result.Content[0].Text
		}
		return nil, errors.New(msg)
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
	Budget     *int  // Maximum output tokens
	JavaScript *bool // Enable JS execution
}

// FetchPageWithOptions fetches a page with additional options.
func (c *Client) FetchPageWithOptions(url string, opts FetchPageOptions) (*Som, error) {
	args := map[string]interface{}{"url": url}
	if opts.Budget != nil {
		args["budget"] = *opts.Budget
	}
	if opts.JavaScript != nil {
		args["javascript"] = *opts.JavaScript
	}
	raw, err := c.callTool("fetch_page", args)
	if err != nil {
		return nil, err
	}
	return Parse(raw)
}

// OpenPage opens a page in a persistent browser session.
func (c *Client) OpenPage(url string) (*PageSession, error) {
	raw, err := c.callTool("open_page", map[string]interface{}{"url": url})
	if err != nil {
		return nil, err
	}
	var session PageSession
	if err := json.Unmarshal(raw, &session); err != nil {
		return nil, fmt.Errorf("plasmate: unmarshal session: %w", err)
	}
	return &session, nil
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

// ClosePage closes a browser session and frees resources.
func (c *Client) ClosePage(sessionID string) error {
	_, err := c.callTool("close_page", map[string]interface{}{
		"session_id": sessionID,
	})
	return err
}

// ExtractText fetches a page and returns clean, readable text.
func (c *Client) ExtractText(url string) (string, error) {
	raw, err := c.callTool("extract_text", map[string]interface{}{"url": url})
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
