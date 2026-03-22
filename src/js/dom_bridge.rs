//! DOM Bridge: bidirectional connection between V8 JavaScript and the rcdom tree.
//!
//! This module provides a NodeRegistry that maps integer IDs to rcdom Handle nodes,
//! allowing JavaScript to create, modify, and query DOM nodes through Rust callbacks.
//! After all scripts execute, the modified tree is re-compiled into a SOM.
//!
//! Architecture: Plasmate is a utility. The SOM is the output paradigm.
//! This bridge extends HOW we build the DOM tree, not WHAT the SOM is.

use html5ever::{LocalName, QualName, Namespace, Prefix, ns, local_name, namespace_url};
use markup5ever_rcdom::{Handle, Node, NodeData, SerializableHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::debug;

/// Maps integer IDs to rcdom tree nodes for V8 interop.
pub struct NodeRegistry {
    nodes: HashMap<u32, Handle>,
    reverse: HashMap<usize, u32>, // ptr address -> id (for dedup)
    next_id: u32,
    document_id: Option<u32>,
    body_id: Option<u32>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        NodeRegistry {
            nodes: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 1,
            document_id: None,
            body_id: None,
        }
    }

    /// Register an existing rcdom node. Returns the assigned ID.
    /// If the node is already registered, returns its existing ID.
    pub fn register(&mut self, handle: &Handle) -> u32 {
        let ptr = Rc::as_ptr(handle) as usize;
        if let Some(&existing_id) = self.reverse.get(&ptr) {
            return existing_id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, handle.clone());
        self.reverse.insert(ptr, id);
        id
    }

    /// Get a node handle by ID.
    pub fn get(&self, id: u32) -> Option<&Handle> {
        self.nodes.get(&id)
    }

    /// Create a new element node and register it.
    pub fn create_element(&mut self, tag: &str) -> u32 {
        let name = QualName::new(
            None::<Prefix>,
            ns!(html),
            LocalName::from(tag.to_lowercase()),
        );
        let node = Node::new(NodeData::Element {
            name,
            attrs: RefCell::new(vec![]),
            template_contents: RefCell::new(None),
            mathml_annotation_xml_integration_point: false,
        });
        self.register(&node)
    }

    /// Create a new text node and register it.
    pub fn create_text(&mut self, text: &str) -> u32 {
        let node = Node::new(NodeData::Text {
            contents: RefCell::new(text.into()),
        });
        self.register(&node)
    }

    /// Append a child node to a parent.
    pub fn append_child(&self, parent_id: u32, child_id: u32) -> Result<(), String> {
        let parent = self.nodes.get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let child = self.nodes.get(&child_id)
            .ok_or_else(|| format!("Child node {} not found", child_id))?;

        // Remove from previous parent if any
        if let Some(prev_parent) = child.parent.take() {
            if let Some(pp) = prev_parent.upgrade() {
                pp.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, child));
            }
        }

        child.parent.set(Some(Rc::downgrade(parent)));
        parent.children.borrow_mut().push(child.clone());
        Ok(())
    }

    /// Remove a child from its parent.
    pub fn remove_child(&self, parent_id: u32, child_id: u32) -> Result<(), String> {
        let parent = self.nodes.get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let child = self.nodes.get(&child_id)
            .ok_or_else(|| format!("Child node {} not found", child_id))?;

        parent.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, child));
        child.parent.set(None);
        Ok(())
    }

    /// Set text content of a node (clears children, adds single text node).
    pub fn set_text_content(&mut self, id: u32, text: &str) -> Result<(), String> {
        let node = self.nodes.get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?
            .clone();

        // Clear existing children
        node.children.borrow_mut().clear();

        // Add text node
        if !text.is_empty() {
            let text_id = self.create_text(text);
            self.append_child(id, text_id)?;
        }
        Ok(())
    }

    /// Get text content of a node (concatenation of all descendant text nodes).
    pub fn get_text_content(&self, id: u32) -> String {
        let node = match self.nodes.get(&id) {
            Some(n) => n,
            None => return String::new(),
        };
        Self::collect_text(node)
    }

    fn collect_text(handle: &Handle) -> String {
        match &handle.data {
            NodeData::Text { contents } => contents.borrow().to_string(),
            _ => {
                let mut result = String::new();
                for child in handle.children.borrow().iter() {
                    result.push_str(&Self::collect_text(child));
                }
                result
            }
        }
    }

    /// Set an attribute on an element.
    pub fn set_attribute(&self, id: u32, name: &str, value: &str) -> Result<(), String> {
        let node = self.nodes.get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?;

        if let NodeData::Element { ref attrs, .. } = node.data {
            let attr_name = QualName::new(None::<Prefix>, ns!(), LocalName::from(name));
            let mut attrs_mut = attrs.borrow_mut();

            // Update existing or add new
            if let Some(attr) = attrs_mut.iter_mut().find(|a| a.name.local.as_ref() == name) {
                attr.value = value.into();
            } else {
                attrs_mut.push(html5ever::Attribute {
                    name: attr_name,
                    value: value.into(),
                });
            }
            Ok(())
        } else {
            Err(format!("Node {} is not an element", id))
        }
    }

    /// Get an attribute value from an element.
    pub fn get_attribute(&self, id: u32, name: &str) -> Option<String> {
        let node = self.nodes.get(&id)?;
        if let NodeData::Element { ref attrs, .. } = node.data {
            attrs.borrow().iter()
                .find(|a| a.name.local.as_ref() == name)
                .map(|a| a.value.to_string())
        } else {
            None
        }
    }

    /// Get the tag name of an element (uppercase).
    pub fn get_tag_name(&self, id: u32) -> Option<String> {
        let node = self.nodes.get(&id)?;
        if let NodeData::Element { ref name, .. } = node.data {
            Some(name.local.as_ref().to_uppercase())
        } else {
            None
        }
    }

    /// Get the class name of an element.
    pub fn get_class_name(&self, id: u32) -> Option<String> {
        self.get_attribute(id, "class")
    }

    /// Get children IDs of a node.
    pub fn get_children(&self, id: u32) -> Vec<u32> {
        let node = match self.nodes.get(&id) {
            Some(n) => n,
            None => return vec![],
        };
        node.children.borrow().iter()
            .filter_map(|child| {
                let ptr = Rc::as_ptr(child) as usize;
                self.reverse.get(&ptr).copied()
            })
            .collect()
    }

    /// Get parent ID of a node.
    pub fn get_parent(&self, id: u32) -> Option<u32> {
        let node = self.nodes.get(&id)?;
        let parent_weak = node.parent.take();
        let result = parent_weak.as_ref().and_then(|w| {
            w.upgrade().and_then(|parent| {
                let ptr = Rc::as_ptr(&parent) as usize;
                self.reverse.get(&ptr).copied()
            })
        });
        node.parent.set(parent_weak);
        result
    }

    /// Get the node type (1=Element, 3=Text, 9=Document, 11=DocumentFragment).
    pub fn get_node_type(&self, id: u32) -> u32 {
        match self.nodes.get(&id).map(|n| &n.data) {
            Some(NodeData::Element { .. }) => 1,
            Some(NodeData::Text { .. }) => 3,
            Some(NodeData::Document) => 9,
            _ => 0,
        }
    }

    /// Walk and register the entire tree starting from root.
    pub fn register_tree(&mut self, root: &Handle) {
        let root_id = self.register(root);

        // Check if this is the document
        if matches!(root.data, NodeData::Document) {
            self.document_id = Some(root_id);
        }

        // Check if this is <body>
        if let NodeData::Element { ref name, .. } = root.data {
            if name.local.as_ref() == "body" {
                self.body_id = Some(root_id);
            }
        }

        // Recursively register children
        for child in root.children.borrow().iter() {
            self.register_tree(child);
        }
    }

    /// Get the document node ID.
    pub fn document_id(&self) -> Option<u32> {
        self.document_id
    }

    /// Get the body element ID.
    pub fn body_id(&self) -> Option<u32> {
        self.body_id
    }

    /// Get the root document handle for SOM re-compilation.
    pub fn root(&self) -> Option<&Handle> {
        self.document_id.and_then(|id| self.nodes.get(&id))
    }

    /// Basic querySelector - matches tag, class, ID, and attribute selectors.
    pub fn query_selector(&self, root_id: u32, selector: &str) -> Option<u32> {
        let root = self.nodes.get(&root_id)?;
        self.find_match(root, selector.trim())
    }

    /// querySelectorAll - returns all matching node IDs.
    pub fn query_selector_all(&self, root_id: u32, selector: &str) -> Vec<u32> {
        let root = match self.nodes.get(&root_id) {
            Some(r) => r,
            None => return vec![],
        };
        let mut results = vec![];
        self.find_all_matches(root, selector.trim(), &mut results);
        results
    }

    /// Recursively find the first matching node.
    fn find_match(&self, node: &Handle, selector: &str) -> Option<u32> {
        // Handle comma-separated selectors
        if selector.contains(',') {
            for part in selector.split(',') {
                if let Some(id) = self.find_match(node, part.trim()) {
                    return Some(id);
                }
            }
            return None;
        }

        for child in node.children.borrow().iter() {
            if self.matches_selector(child, selector) {
                let ptr = Rc::as_ptr(child) as usize;
                if let Some(&id) = self.reverse.get(&ptr) {
                    return Some(id);
                }
            }
            if let Some(id) = self.find_match(child, selector) {
                return Some(id);
            }
        }
        None
    }

    /// Recursively find all matching nodes.
    fn find_all_matches(&self, node: &Handle, selector: &str, results: &mut Vec<u32>) {
        // Handle comma-separated selectors
        let selectors: Vec<&str> = if selector.contains(',') {
            selector.split(',').map(|s| s.trim()).collect()
        } else {
            vec![selector]
        };

        for child in node.children.borrow().iter() {
            for sel in &selectors {
                if self.matches_selector(child, sel) {
                    let ptr = Rc::as_ptr(child) as usize;
                    if let Some(&id) = self.reverse.get(&ptr) {
                        if !results.contains(&id) {
                            results.push(id);
                        }
                    }
                }
            }
            self.find_all_matches(child, selector, results);
        }
    }

    /// Check if a node matches a simple CSS selector.
    /// Supports: tag, .class, #id, [attr], [attr=value], tag.class, tag#id
    fn matches_selector(&self, node: &Handle, selector: &str) -> bool {
        let selector = selector.trim();
        if selector.is_empty() {
            return false;
        }

        if let NodeData::Element { ref name, ref attrs, .. } = node.data {
            let tag = name.local.as_ref();
            let attrs_ref = attrs.borrow();
            let class_attr = attrs_ref.iter()
                .find(|a| a.name.local.as_ref() == "class")
                .map(|a| a.value.to_string())
                .unwrap_or_default();
            let id_attr = attrs_ref.iter()
                .find(|a| a.name.local.as_ref() == "id")
                .map(|a| a.value.to_string())
                .unwrap_or_default();

            // Handle compound selectors like "div.header" or "div#main"
            // Split on . and # boundaries
            let parts = split_compound_selector(selector);

            for part in &parts {
                let matched = match part.chars().next() {
                    Some('#') => id_attr == &part[1..],
                    Some('.') => {
                        let cls = &part[1..];
                        class_attr.split_whitespace().any(|c| c == cls)
                    },
                    Some('[') => {
                        // [attr] or [attr=value]
                        let inner = part.trim_start_matches('[').trim_end_matches(']');
                        if let Some(eq_pos) = inner.find('=') {
                            let attr_name = &inner[..eq_pos];
                            let attr_val = inner[eq_pos+1..].trim_matches('"').trim_matches('\'');
                            attrs_ref.iter().any(|a| a.name.local.as_ref() == attr_name && a.value.as_ref() == attr_val)
                        } else {
                            attrs_ref.iter().any(|a| a.name.local.as_ref() == inner)
                        }
                    },
                    Some('*') => true, // universal selector
                    _ => tag.eq_ignore_ascii_case(part),
                };
                if !matched {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Total registered node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Split a compound selector like "div.header#main[data-x]" into parts.
/// Returns: ["div", ".header", "#main", "[data-x]"]
fn split_compound_selector(selector: &str) -> Vec<&str> {
    let mut parts = vec![];
    let mut start = 0;
    let bytes = selector.as_bytes();
    let len = bytes.len();

    for i in 1..len {
        let ch = bytes[i] as char;
        if ch == '.' || ch == '#' || ch == '[' {
            if i > start {
                parts.push(&selector[start..i]);
            }
            start = i;
        }
    }
    if start < len {
        parts.push(&selector[start..]);
    }
    if parts.is_empty() {
        parts.push(selector);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry_with_tree() -> NodeRegistry {
        let mut reg = NodeRegistry::new();

        // Create a simple tree: <div id="root"><p class="intro">Hello</p><span>World</span></div>
        let div_id = reg.create_element("div");
        reg.set_attribute(div_id, "id", "root").unwrap();

        let p_id = reg.create_element("p");
        reg.set_attribute(p_id, "class", "intro").unwrap();
        let text1_id = reg.create_text("Hello");
        reg.append_child(p_id, text1_id).unwrap();

        let span_id = reg.create_element("span");
        let text2_id = reg.create_text("World");
        reg.append_child(span_id, text2_id).unwrap();

        reg.append_child(div_id, p_id).unwrap();
        reg.append_child(div_id, span_id).unwrap();

        reg
    }

    #[test]
    fn test_create_and_append() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("div");
        let child = reg.create_element("span");
        assert!(reg.append_child(parent, child).is_ok());
        assert_eq!(reg.get_children(parent), vec![child]);
    }

    #[test]
    fn test_text_content() {
        let mut reg = NodeRegistry::new();
        let div = reg.create_element("div");
        reg.set_text_content(div, "Hello World").unwrap();
        assert_eq!(reg.get_text_content(div), "Hello World");
    }

    #[test]
    fn test_attributes() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("a");
        reg.set_attribute(el, "href", "https://example.com").unwrap();
        assert_eq!(reg.get_attribute(el, "href"), Some("https://example.com".to_string()));
        assert_eq!(reg.get_attribute(el, "missing"), None);
    }

    #[test]
    fn test_tag_name() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");
        assert_eq!(reg.get_tag_name(el), Some("DIV".to_string()));
    }

    #[test]
    fn test_query_selector_by_tag() {
        let reg = make_registry_with_tree();
        // Find first 'p' inside div (id=1)
        let result = reg.query_selector(1, "p");
        assert!(result.is_some());
        assert_eq!(reg.get_tag_name(result.unwrap()), Some("P".to_string()));
    }

    #[test]
    fn test_query_selector_by_class() {
        let reg = make_registry_with_tree();
        let result = reg.query_selector(1, ".intro");
        assert!(result.is_some());
        assert_eq!(reg.get_class_name(result.unwrap()), Some("intro".to_string()));
    }

    #[test]
    fn test_query_selector_by_id() {
        let reg = make_registry_with_tree();
        let result = reg.query_selector(1, "#root");
        // #root is the div itself, which is the search root, so it checks children
        // The div has id=root but querySelector searches descendants, not self
        assert!(result.is_none()); // root is the starting point, not a descendant
    }

    #[test]
    fn test_query_selector_all() {
        let reg = make_registry_with_tree();
        let results = reg.query_selector_all(1, "span");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_remove_child() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("div");
        let child = reg.create_element("span");
        reg.append_child(parent, child).unwrap();
        assert_eq!(reg.get_children(parent).len(), 1);
        reg.remove_child(parent, child).unwrap();
        assert_eq!(reg.get_children(parent).len(), 0);
    }

    #[test]
    fn test_get_parent() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("div");
        let child = reg.create_element("span");
        reg.append_child(parent, child).unwrap();
        assert_eq!(reg.get_parent(child), Some(parent));
    }

    #[test]
    fn test_compound_selector() {
        let mut reg = NodeRegistry::new();
        let root = reg.create_element("div");
        let el = reg.create_element("p");
        reg.set_attribute(el, "class", "intro highlight").unwrap();
        reg.append_child(root, el).unwrap();

        assert!(reg.query_selector(root, "p.intro").is_some());
        assert!(reg.query_selector(root, "p.highlight").is_some());
        assert!(reg.query_selector(root, "div.intro").is_none());
    }

    #[test]
    fn test_node_count() {
        let mut reg = NodeRegistry::new();
        assert_eq!(reg.node_count(), 0);
        reg.create_element("div");
        reg.create_element("span");
        reg.create_text("hello");
        assert_eq!(reg.node_count(), 3);
    }
}
