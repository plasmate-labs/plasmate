//! DOM Bridge: bidirectional connection between V8 JavaScript and the rcdom tree.
//!
//! This module provides a NodeRegistry that maps integer IDs to rcdom Handle nodes,
//! allowing JavaScript to create, modify, and query DOM nodes through Rust callbacks.
//! After all scripts execute, the modified tree is re-compiled into a SOM.
//!
//! Architecture: Plasmate is a utility. The SOM is the output paradigm.
//! This bridge extends HOW we build the DOM tree, not WHAT the SOM is.

use html5ever::serialize::{serialize, SerializeOpts, TraversalScope};
use html5ever::tendril::TendrilSink;
use html5ever::{local_name, namespace_url, ns, LocalName, Namespace, Prefix, QualName};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};
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
        let parent = self
            .nodes
            .get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let child = self
            .nodes
            .get(&child_id)
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
        let parent = self
            .nodes
            .get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let child = self
            .nodes
            .get(&child_id)
            .ok_or_else(|| format!("Child node {} not found", child_id))?;

        parent
            .children
            .borrow_mut()
            .retain(|c| !Rc::ptr_eq(c, child));
        child.parent.set(None);
        Ok(())
    }

    /// Set text content of a node (clears children, adds single text node).
    pub fn set_text_content(&mut self, id: u32, text: &str) -> Result<(), String> {
        let node = self
            .nodes
            .get(&id)
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
        let node = self
            .nodes
            .get(&id)
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
            attrs
                .borrow()
                .iter()
                .find(|a| a.name.local.as_ref() == name)
                .map(|a| a.value.to_string())
        } else {
            None
        }
    }

    /// Get innerHTML by serializing this node's children.
    pub fn get_inner_html(&self, id: u32) -> Result<String, String> {
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?
            .clone();

        let mut bytes: Vec<u8> = Vec::new();
        let opts = SerializeOpts {
            traversal_scope: TraversalScope::ChildrenOnly(None),
            ..Default::default()
        };
        serialize(&mut bytes, &SerializableHandle::from(node), opts)
            .map_err(|e: std::io::Error| e.to_string())?;

        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Set innerHTML (parse HTML fragment and replace children).
    pub fn set_inner_html(&mut self, id: u32, html: &str) -> Result<(), String> {
        let target = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?
            .clone();

        // Clear existing children
        target.children.borrow_mut().clear();

        // Choose context element name and attrs
        let (ctx_name, ctx_attrs) = match target.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => (name.clone(), attrs.borrow().clone()),
            _ => (
                QualName::new(None::<Prefix>, ns!(html), LocalName::from("div")),
                vec![],
            ),
        };

        let dom = html5ever::parse_fragment(
            RcDom::default(),
            Default::default(),
            ctx_name,
            ctx_attrs,
            false,
        )
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e: std::io::Error| e.to_string())?;

        // The parsed fragment tree has a Document root; move its children to target.
        let children: Vec<Handle> = dom.document.children.borrow().iter().cloned().collect();

        for child in children {
            child.parent.set(None);
            child.parent.set(Some(Rc::downgrade(&target)));
            target.children.borrow_mut().push(child.clone());
            self.register_tree(&child);
        }

        Ok(())
    }

    /// Serialize the entire document to HTML.
    pub fn serialize_document(&self) -> Result<String, String> {
        let Some(doc_id) = self.document_id else {
            return Ok(String::new());
        };
        let doc = self
            .nodes
            .get(&doc_id)
            .ok_or_else(|| "document not found".to_string())?;

        // Find first <html> element under document.
        let mut html_el: Option<Handle> = None;
        for c in doc.children.borrow().iter() {
            if let NodeData::Element { ref name, .. } = c.data {
                if name.local.as_ref() == "html" {
                    html_el = Some(c.clone());
                    break;
                }
            }
        }
        let Some(html_el) = html_el else {
            return Ok(String::new());
        };

        let mut bytes: Vec<u8> = Vec::new();
        let opts = SerializeOpts {
            traversal_scope: TraversalScope::IncludeNode,
            ..Default::default()
        };
        serialize(&mut bytes, &SerializableHandle::from(html_el), opts)
            .map_err(|e: std::io::Error| e.to_string())?;

        Ok(format!(
            "<!DOCTYPE html>{}",
            String::from_utf8_lossy(&bytes)
        ))
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
        node.children
            .borrow()
            .iter()
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

    /// Insert a child before a reference node.
    pub fn insert_before(
        &self,
        parent_id: u32,
        new_child_id: u32,
        ref_child_id: u32,
    ) -> Result<(), String> {
        let parent = self
            .nodes
            .get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let new_child = self
            .nodes
            .get(&new_child_id)
            .ok_or_else(|| format!("New child node {} not found", new_child_id))?;
        let ref_child = self
            .nodes
            .get(&ref_child_id)
            .ok_or_else(|| format!("Ref child node {} not found", ref_child_id))?;

        // Remove from previous parent if any
        if let Some(prev_parent) = new_child.parent.take() {
            if let Some(pp) = prev_parent.upgrade() {
                pp.children
                    .borrow_mut()
                    .retain(|c| !Rc::ptr_eq(c, new_child));
            }
        }

        // Find ref_child position and insert before it
        let mut children = parent.children.borrow_mut();
        let pos = children
            .iter()
            .position(|c| Rc::ptr_eq(c, ref_child))
            .unwrap_or(children.len());
        new_child.parent.set(Some(Rc::downgrade(parent)));
        children.insert(pos, new_child.clone());
        Ok(())
    }

    /// Replace a child with a new node.
    pub fn replace_child(
        &self,
        parent_id: u32,
        new_child_id: u32,
        old_child_id: u32,
    ) -> Result<(), String> {
        let parent = self
            .nodes
            .get(&parent_id)
            .ok_or_else(|| format!("Parent node {} not found", parent_id))?;
        let new_child = self
            .nodes
            .get(&new_child_id)
            .ok_or_else(|| format!("New child node {} not found", new_child_id))?;
        let old_child = self
            .nodes
            .get(&old_child_id)
            .ok_or_else(|| format!("Old child node {} not found", old_child_id))?;

        // Remove new_child from its current parent
        if let Some(prev) = new_child.parent.take() {
            if let Some(pp) = prev.upgrade() {
                pp.children
                    .borrow_mut()
                    .retain(|c| !Rc::ptr_eq(c, new_child));
            }
        }

        let mut children = parent.children.borrow_mut();
        if let Some(pos) = children.iter().position(|c| Rc::ptr_eq(c, old_child)) {
            new_child.parent.set(Some(Rc::downgrade(parent)));
            old_child.parent.set(None);
            children[pos] = new_child.clone();
        }
        Ok(())
    }

    /// Clone a node. If `deep` is true, recursively clone children.
    pub fn clone_node(&mut self, id: u32, deep: bool) -> Option<u32> {
        // Extract all data we need from the source node before any mutation
        let node = self.nodes.get(&id)?;
        enum CloneSource {
            Element {
                name: QualName,
                attrs: Vec<html5ever::Attribute>,
                child_ids: Vec<u32>,
            },
            Text(String),
        }
        let source = match &node.data {
            NodeData::Element { name, attrs, .. } => {
                let child_ids = if deep {
                    node.children
                        .borrow()
                        .iter()
                        .filter_map(|child| {
                            let ptr = Rc::as_ptr(child) as usize;
                            self.reverse.get(&ptr).copied()
                        })
                        .collect()
                } else {
                    vec![]
                };
                CloneSource::Element {
                    name: name.clone(),
                    attrs: attrs.borrow().clone(),
                    child_ids,
                }
            }
            NodeData::Text { contents } => CloneSource::Text(contents.borrow().to_string()),
            _ => return None,
        };

        match source {
            CloneSource::Text(text) => {
                let new_id = self.create_text(&text);
                Some(new_id)
            }
            CloneSource::Element {
                name,
                attrs,
                child_ids,
            } => {
                let new_node = Node::new(NodeData::Element {
                    name,
                    attrs: RefCell::new(attrs),
                    template_contents: RefCell::new(None),
                    mathml_annotation_xml_integration_point: false,
                });
                let new_id = self.register(&new_node);
                for child_id in child_ids {
                    if let Some(cloned_child_id) = self.clone_node(child_id, true) {
                        let _ = self.append_child(new_id, cloned_child_id);
                    }
                }
                Some(new_id)
            }
        }
    }

    /// Check if an element has a specific class.
    pub fn has_class(&self, id: u32, class_name: &str) -> bool {
        self.get_class_name(id)
            .map(|classes| classes.split_whitespace().any(|c| c == class_name))
            .unwrap_or(false)
    }

    /// Add a class to an element.
    pub fn add_class(&self, id: u32, class_name: &str) -> Result<(), String> {
        let current = self.get_attribute(id, "class").unwrap_or_default();
        if !current.split_whitespace().any(|c| c == class_name) {
            let new_val = if current.is_empty() {
                class_name.to_string()
            } else {
                format!("{} {}", current, class_name)
            };
            self.set_attribute(id, "class", &new_val)
        } else {
            Ok(())
        }
    }

    /// Remove a class from an element.
    pub fn remove_class(&self, id: u32, class_name: &str) -> Result<(), String> {
        let current = self.get_attribute(id, "class").unwrap_or_default();
        let new_val: Vec<&str> = current
            .split_whitespace()
            .filter(|c| *c != class_name)
            .collect();
        self.set_attribute(id, "class", &new_val.join(" "))
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

        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = node.data
        {
            let tag = name.local.as_ref();
            let attrs_ref = attrs.borrow();
            let class_attr = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "class")
                .map(|a| a.value.to_string())
                .unwrap_or_default();
            let id_attr = attrs_ref
                .iter()
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
                    }
                    Some('[') => {
                        // [attr] or [attr=value]
                        let inner = part.trim_start_matches('[').trim_end_matches(']');
                        if let Some(eq_pos) = inner.find('=') {
                            let attr_name = &inner[..eq_pos];
                            let attr_val = inner[eq_pos + 1..].trim_matches('"').trim_matches('\'');
                            attrs_ref.iter().any(|a| {
                                a.name.local.as_ref() == attr_name && a.value.as_ref() == attr_val
                            })
                        } else {
                            attrs_ref.iter().any(|a| a.name.local.as_ref() == inner)
                        }
                    }
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

    /// Wait for a selector to match an element.
    ///
    /// Since Plasmate doesn't have a live event loop, we simply check
    /// whether the element currently exists in the final DOM state.
    pub fn wait_for_selector(&self, root_id: u32, selector: &str, _timeout_ms: u64) -> Option<u32> {
        self.query_selector(root_id, selector)
    }

    /// Click an element by ID.
    ///
    /// For `<a href="...">` elements, returns `ClickResult::Navigate` with the URL.
    /// For all other elements, returns `ClickResult::Clicked`.
    pub fn click(&self, id: u32) -> Result<ClickResult, String> {
        let _node = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?;

        // Check if it's a link with an href
        if let Some(href) = self.get_attribute(id, "href") {
            if !href.is_empty() {
                return Ok(ClickResult::Navigate(href));
            }
        }

        // For form submit buttons, check the parent form's action
        if let Some(tag) = self.get_tag_name(id) {
            let tag_lower = tag.to_lowercase();
            if tag_lower == "button" || tag_lower == "input" {
                let input_type = self.get_attribute(id, "type").unwrap_or_default();
                if input_type == "submit" {
                    if let Some(parent_id) = self.get_parent(id) {
                        if let Some(parent_tag) = self.get_tag_name(parent_id) {
                            if parent_tag.to_lowercase() == "form" {
                                if let Some(action) = self.get_attribute(parent_id, "action") {
                                    if !action.is_empty() {
                                        return Ok(ClickResult::FormSubmit(action));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ClickResult::Clicked)
    }

    /// Type text into an element by setting its value attribute.
    ///
    /// Works for `<input>` and `<textarea>` elements.
    pub fn type_text(&self, id: u32, text: &str) -> Result<(), String> {
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} not found", id))?;

        match &node.data {
            NodeData::Element { .. } => {
                self.set_attribute(id, "value", text)?;
                Ok(())
            }
            _ => Err(format!("Node {} is not an element", id)),
        }
    }

    /// Total registered node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Result of clicking a DOM element.
#[derive(Debug, Clone, PartialEq)]
pub enum ClickResult {
    /// The element was clicked (generic click).
    Clicked,
    /// The element is a link; navigate to this URL.
    Navigate(String),
    /// The element is a submit button in a form with this action URL.
    FormSubmit(String),
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
        reg.set_attribute(el, "href", "https://example.com")
            .unwrap();
        assert_eq!(
            reg.get_attribute(el, "href"),
            Some("https://example.com".to_string())
        );
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
        assert_eq!(
            reg.get_class_name(result.unwrap()),
            Some("intro".to_string())
        );
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

    // =========================================================================
    // insert_before / replace_child / clone_node / classList tests
    // =========================================================================

    #[test]
    fn test_insert_before() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("ul");
        let li1 = reg.create_element("li");
        let li2 = reg.create_element("li");
        let li_new = reg.create_element("li");

        reg.append_child(parent, li1).unwrap();
        reg.append_child(parent, li2).unwrap();
        assert_eq!(reg.get_children(parent), vec![li1, li2]);

        // Insert li_new before li2
        reg.insert_before(parent, li_new, li2).unwrap();
        assert_eq!(reg.get_children(parent), vec![li1, li_new, li2]);
        assert_eq!(reg.get_parent(li_new), Some(parent));
    }

    #[test]
    fn test_insert_before_moves_from_old_parent() {
        let mut reg = NodeRegistry::new();
        let parent_a = reg.create_element("div");
        let parent_b = reg.create_element("div");
        let child = reg.create_element("span");
        let ref_child = reg.create_element("p");

        reg.append_child(parent_a, child).unwrap();
        reg.append_child(parent_b, ref_child).unwrap();

        // Move child from parent_a to parent_b before ref_child
        reg.insert_before(parent_b, child, ref_child).unwrap();
        assert_eq!(reg.get_children(parent_a), Vec::<u32>::new());
        assert_eq!(reg.get_children(parent_b), vec![child, ref_child]);
    }

    #[test]
    fn test_replace_child() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("div");
        let old = reg.create_element("span");
        let new = reg.create_element("em");

        reg.append_child(parent, old).unwrap();
        assert_eq!(reg.get_children(parent), vec![old]);

        reg.replace_child(parent, new, old).unwrap();
        assert_eq!(reg.get_children(parent), vec![new]);
        assert_eq!(reg.get_parent(new), Some(parent));
        assert_eq!(reg.get_parent(old), None);
    }

    #[test]
    fn test_replace_child_preserves_position() {
        let mut reg = NodeRegistry::new();
        let parent = reg.create_element("ul");
        let a = reg.create_element("li");
        let b = reg.create_element("li");
        let c = reg.create_element("li");
        let replacement = reg.create_element("li");

        reg.append_child(parent, a).unwrap();
        reg.append_child(parent, b).unwrap();
        reg.append_child(parent, c).unwrap();

        // Replace b (middle) with replacement
        reg.replace_child(parent, replacement, b).unwrap();
        assert_eq!(reg.get_children(parent), vec![a, replacement, c]);
    }

    #[test]
    fn test_clone_node_shallow() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");
        reg.set_attribute(el, "class", "test").unwrap();
        let child = reg.create_element("span");
        reg.append_child(el, child).unwrap();

        let cloned = reg.clone_node(el, false).unwrap();
        assert_ne!(cloned, el);
        assert_eq!(reg.get_tag_name(cloned), Some("DIV".to_string()));
        assert_eq!(reg.get_attribute(cloned, "class"), Some("test".to_string()));
        // Shallow: no children
        assert_eq!(reg.get_children(cloned), Vec::<u32>::new());
    }

    #[test]
    fn test_clone_node_deep() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");
        let child = reg.create_element("span");
        let text = reg.create_text("hello");
        reg.append_child(child, text).unwrap();
        reg.append_child(el, child).unwrap();

        let cloned = reg.clone_node(el, true).unwrap();
        assert_ne!(cloned, el);
        let cloned_children = reg.get_children(cloned);
        assert_eq!(cloned_children.len(), 1);
        let cloned_span = cloned_children[0];
        assert_ne!(cloned_span, child);
        assert_eq!(reg.get_tag_name(cloned_span), Some("SPAN".to_string()));
        // Deep clone should copy text
        assert_eq!(reg.get_text_content(cloned_span), "hello");
    }

    #[test]
    fn test_clone_text_node() {
        let mut reg = NodeRegistry::new();
        let text = reg.create_text("hello world");
        let cloned = reg.clone_node(text, false).unwrap();
        assert_ne!(cloned, text);
        assert_eq!(reg.get_text_content(cloned), "hello world");
    }

    #[test]
    fn test_has_class() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");
        reg.set_attribute(el, "class", "foo bar baz").unwrap();

        assert!(reg.has_class(el, "foo"));
        assert!(reg.has_class(el, "bar"));
        assert!(reg.has_class(el, "baz"));
        assert!(!reg.has_class(el, "qux"));
        assert!(!reg.has_class(el, "fo")); // partial match should not count
    }

    #[test]
    fn test_add_class() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");

        // Add to empty
        reg.add_class(el, "alpha").unwrap();
        assert_eq!(reg.get_attribute(el, "class"), Some("alpha".to_string()));

        // Add second
        reg.add_class(el, "beta").unwrap();
        assert_eq!(
            reg.get_attribute(el, "class"),
            Some("alpha beta".to_string())
        );

        // No duplicate
        reg.add_class(el, "alpha").unwrap();
        assert_eq!(
            reg.get_attribute(el, "class"),
            Some("alpha beta".to_string())
        );
    }

    #[test]
    fn test_remove_class() {
        let mut reg = NodeRegistry::new();
        let el = reg.create_element("div");
        reg.set_attribute(el, "class", "a b c").unwrap();

        reg.remove_class(el, "b").unwrap();
        assert_eq!(reg.get_attribute(el, "class"), Some("a c".to_string()));

        reg.remove_class(el, "a").unwrap();
        assert_eq!(reg.get_attribute(el, "class"), Some("c".to_string()));

        reg.remove_class(el, "c").unwrap();
        assert_eq!(reg.get_attribute(el, "class"), Some("".to_string()));

        // Removing nonexistent class is fine
        reg.remove_class(el, "z").unwrap();
    }

    #[test]
    fn test_wait_for_selector_found() {
        let reg = make_registry_with_tree();
        let result = reg.wait_for_selector(1, "p", 5000);
        assert!(result.is_some());
    }

    #[test]
    fn test_wait_for_selector_not_found() {
        let reg = make_registry_with_tree();
        let result = reg.wait_for_selector(1, ".nonexistent", 5000);
        assert!(result.is_none());
    }

    #[test]
    fn test_click_link() {
        let mut reg = NodeRegistry::new();
        let root = reg.create_element("div");
        let link = reg.create_element("a");
        reg.set_attribute(link, "href", "https://example.com")
            .unwrap();
        reg.append_child(root, link).unwrap();

        match reg.click(link).unwrap() {
            ClickResult::Navigate(url) => assert_eq!(url, "https://example.com"),
            other => panic!("Expected Navigate, got {:?}", other),
        }
    }

    #[test]
    fn test_click_button() {
        let mut reg = NodeRegistry::new();
        let btn = reg.create_element("button");
        assert_eq!(reg.click(btn).unwrap(), ClickResult::Clicked);
    }

    #[test]
    fn test_click_not_found() {
        let reg = NodeRegistry::new();
        assert!(reg.click(999).is_err());
    }

    #[test]
    fn test_type_text() {
        let mut reg = NodeRegistry::new();
        let input = reg.create_element("input");
        reg.type_text(input, "hello world").unwrap();
        assert_eq!(
            reg.get_attribute(input, "value"),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_type_text_not_element() {
        let mut reg = NodeRegistry::new();
        let text = reg.create_text("plain text");
        assert!(reg.type_text(text, "new value").is_err());
    }
}
