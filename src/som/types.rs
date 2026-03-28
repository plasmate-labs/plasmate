use serde::{Deserialize, Serialize};

/// Top-level SOM (Semantic Object Model) snapshot for a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Som {
    pub som_version: String,
    pub url: String,
    pub title: String,
    pub lang: String,
    pub regions: Vec<Region>,
    pub meta: SomMeta,
    /// Structured data extracted from the page (JSON-LD, OpenGraph, Twitter Cards, meta).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_data: Option<super::metadata::StructuredData>,
}

/// Metadata about the SOM compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomMeta {
    pub html_bytes: usize,
    pub som_bytes: usize,
    pub element_count: usize,
    pub interactive_count: usize,
}

/// A semantic region within the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub role: RegionRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    pub elements: Vec<Element>,
}

/// Region roles as defined by the SOM spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RegionRole {
    Navigation,
    Main,
    Aside,
    Header,
    Footer,
    Form,
    Dialog,
    Content,
}

/// A semantic element within a region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: String,
    pub role: ElementRole,
    /// The original HTML `id` attribute, when present on the source element.
    /// Enables agents to resolve back to the DOM for interaction (e.g. via
    /// `document.getElementById()` or CSS selector `#id`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Element>>,
    /// Semantic hints inferred from CSS classes (e.g. "primary", "danger", "disabled").
    /// Helps agents understand element importance without seeing raw CSS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<String>>,
}

/// Element roles as defined by the SOM spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ElementRole {
    Link,
    Button,
    TextInput,
    Textarea,
    Select,
    Checkbox,
    Radio,
    Heading,
    Image,
    List,
    Table,
    Paragraph,
    Section,
    Separator,
}

impl ElementRole {
    /// Whether this role represents an interactive element.
    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            ElementRole::Link
                | ElementRole::Button
                | ElementRole::TextInput
                | ElementRole::Textarea
                | ElementRole::Select
                | ElementRole::Checkbox
                | ElementRole::Radio
        )
    }

    /// Default actions for this role.
    pub fn default_actions(&self) -> Vec<String> {
        match self {
            ElementRole::Link => vec!["click".into()],
            ElementRole::Button => vec!["click".into()],
            ElementRole::TextInput => vec!["type".into(), "clear".into()],
            ElementRole::Textarea => vec!["type".into(), "clear".into()],
            ElementRole::Select => vec!["select".into()],
            ElementRole::Checkbox => vec!["toggle".into()],
            ElementRole::Radio => vec!["select".into()],
            _ => vec![],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ElementRole::Link => "link",
            ElementRole::Button => "button",
            ElementRole::TextInput => "text_input",
            ElementRole::Textarea => "textarea",
            ElementRole::Select => "select",
            ElementRole::Checkbox => "checkbox",
            ElementRole::Radio => "radio",
            ElementRole::Heading => "heading",
            ElementRole::Image => "image",
            ElementRole::List => "list",
            ElementRole::Table => "table",
            ElementRole::Paragraph => "paragraph",
            ElementRole::Section => "section",
            ElementRole::Separator => "separator",
        }
    }
}
