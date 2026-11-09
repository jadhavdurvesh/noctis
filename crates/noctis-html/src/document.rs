//! Arena-backed DOM document with stable node identifiers and explicit parent/sibling links.
//!
//! Removed nodes remain allocated and detached. This keeps previously-issued
//! identifiers stable; reclamation/generational handles can be added later.

use crate::{Attribute, Node};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomNodeId(u32);

impl DomNodeId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomNodeKind {
    Document,
    Element {
        name: String,
        attributes: Vec<Attribute>,
    },
    Text(String),
    Comment(String),
    Doctype(String),
}

impl DomNodeKind {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Element { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(text) | Self::Comment(text) | Self::Doctype(text) => Some(text),
            _ => None,
        }
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        let Self::Element { attributes, .. } = self else {
            return None;
        };
        attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .and_then(|attribute| attribute.value.as_deref())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        let Self::Element { attributes, .. } = self else {
            return false;
        };
        attributes
            .iter()
            .any(|attribute| attribute.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomNode {
    kind: DomNodeKind,
    parent: Option<DomNodeId>,
    children: Vec<DomNodeId>,
}

impl DomNode {
    pub fn kind(&self) -> &DomNodeKind {
        &self.kind
    }

    pub fn parent(&self) -> Option<DomNodeId> {
        self.parent
    }

    pub fn children(&self) -> &[DomNodeId] {
        &self.children
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomError {
    InvalidNode(DomNodeId),
    ParentCannotHaveChildren(DomNodeId),
    HierarchyRequest,
    ReferenceIsNotChild {
        parent: DomNodeId,
        reference: DomNodeId,
    },
    NodeIdSpaceExhausted,
}

impl fmt::Display for DomError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode(id) => write!(formatter, "invalid DOM node {}", id.0),
            Self::ParentCannotHaveChildren(id) => {
                write!(formatter, "DOM node {} cannot have children", id.0)
            }
            Self::HierarchyRequest => write!(formatter, "DOM operation would create a cycle"),
            Self::ReferenceIsNotChild { parent, reference } => write!(
                formatter,
                "reference node {} is not a child of parent {}",
                reference.0, parent.0
            ),
            Self::NodeIdSpaceExhausted => write!(formatter, "DOM node identifier space exhausted"),
        }
    }
}

impl std::error::Error for DomError {}

#[derive(Debug, Clone)]
pub struct DomDocument {
    nodes: Vec<DomNode>,
    document_node: DomNodeId,
}

impl Default for DomDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl DomDocument {
    pub fn new() -> Self {
        let document_node = DomNodeId(0);
        Self {
            nodes: vec![DomNode {
                kind: DomNodeKind::Document,
                parent: None,
                children: Vec::new(),
            }],
            document_node,
        }
    }

    pub fn from_tree(tree: &Node) -> Result<Self, DomError> {
        fn insert_tree(document: &mut DomDocument, tree: &Node) -> Result<DomNodeId, DomError> {
            let kind = match tree {
                Node::Document(_) => DomNodeKind::Document,
                Node::Element {
                    name, attributes, ..
                } => DomNodeKind::Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                },
                Node::Text(text) => DomNodeKind::Text(text.clone()),
                Node::Comment(text) => DomNodeKind::Comment(text.clone()),
                Node::Doctype(name) => DomNodeKind::Doctype(name.clone()),
            };
            let id = document.allocate(kind)?;
            for child in tree.children() {
                let child_id = insert_tree(document, child)?;
                document.append_child(id, child_id)?;
            }
            Ok(id)
        }

        let mut document = Self::new();
        match tree {
            Node::Document(children) => {
                for child in children {
                    let child_id = insert_tree(&mut document, child)?;
                    document.append_child(document.document_node, child_id)?;
                }
            }
            other => {
                let child_id = insert_tree(&mut document, other)?;
                document.append_child(document.document_node, child_id)?;
            }
        }
        Ok(document)
    }

    fn allocate(&mut self, kind: DomNodeKind) -> Result<DomNodeId, DomError> {
        let index = u32::try_from(self.nodes.len()).map_err(|_| DomError::NodeIdSpaceExhausted)?;
        let id = DomNodeId(index);
        self.nodes.push(DomNode {
            kind,
            parent: None,
            children: Vec::new(),
        });
        Ok(id)
    }

    pub fn create_element(&mut self, name: &str) -> Result<DomNodeId, DomError> {
        self.allocate(DomNodeKind::Element {
            name: name.to_ascii_lowercase(),
            attributes: Vec::new(),
        })
    }

    pub fn create_text_node(&mut self, text: &str) -> Result<DomNodeId, DomError> {
        self.allocate(DomNodeKind::Text(text.to_owned()))
    }

    pub fn create_comment(&mut self, text: &str) -> Result<DomNodeId, DomError> {
        self.allocate(DomNodeKind::Comment(text.to_owned()))
    }

    pub fn document_node(&self) -> DomNodeId {
        self.document_node
    }

    pub fn node(&self, id: DomNodeId) -> Result<&DomNode, DomError> {
        self.nodes.get(id.index()).ok_or(DomError::InvalidNode(id))
    }

    pub fn node_mut(&mut self, id: DomNodeId) -> Result<&mut DomNode, DomError> {
        self.nodes
            .get_mut(id.index())
            .ok_or(DomError::InvalidNode(id))
    }

    pub fn kind(&self, id: DomNodeId) -> Result<&DomNodeKind, DomError> {
        Ok(self.node(id)?.kind())
    }

    pub fn parent_node(&self, id: DomNodeId) -> Result<Option<DomNodeId>, DomError> {
        Ok(self.node(id)?.parent())
    }

    pub fn child_nodes(&self, id: DomNodeId) -> Result<&[DomNodeId], DomError> {
        Ok(self.node(id)?.children())
    }

    pub fn first_child(&self, id: DomNodeId) -> Result<Option<DomNodeId>, DomError> {
        Ok(self.node(id)?.children.first().copied())
    }

    pub fn last_child(&self, id: DomNodeId) -> Result<Option<DomNodeId>, DomError> {
        Ok(self.node(id)?.children.last().copied())
    }

    pub fn previous_sibling(&self, id: DomNodeId) -> Result<Option<DomNodeId>, DomError> {
        let node = self.node(id)?;
        let Some(parent) = node.parent else {
            return Ok(None);
        };
        let siblings = self.node(parent)?.children();
        let position = siblings
            .iter()
            .position(|sibling| *sibling == id)
            .ok_or(DomError::HierarchyRequest)?;
        Ok(position.checked_sub(1).map(|index| siblings[index]))
    }

    pub fn next_sibling(&self, id: DomNodeId) -> Result<Option<DomNodeId>, DomError> {
        let node = self.node(id)?;
        let Some(parent) = node.parent else {
            return Ok(None);
        };
        let siblings = self.node(parent)?.children();
        let position = siblings
            .iter()
            .position(|sibling| *sibling == id)
            .ok_or(DomError::HierarchyRequest)?;
        Ok(siblings.get(position + 1).copied())
    }

    fn ensure_container(&self, parent: DomNodeId) -> Result<(), DomError> {
        match self.kind(parent)? {
            DomNodeKind::Document | DomNodeKind::Element { .. } => Ok(()),
            _ => Err(DomError::ParentCannotHaveChildren(parent)),
        }
    }

    fn ensure_no_cycle(&self, parent: DomNodeId, child: DomNodeId) -> Result<(), DomError> {
        let mut cursor = Some(parent);
        while let Some(current) = cursor {
            if current == child {
                return Err(DomError::HierarchyRequest);
            }
            cursor = self.node(current)?.parent();
        }
        Ok(())
    }

    fn detach(&mut self, child: DomNodeId) -> Result<(), DomError> {
        let parent = self.node(child)?.parent();
        if let Some(parent_id) = parent {
            let parent_node = self.node_mut(parent_id)?;
            let index = parent_node
                .children
                .iter()
                .position(|candidate| *candidate == child)
                .ok_or(DomError::HierarchyRequest)?;
            parent_node.children.remove(index);
            self.node_mut(child)?.parent = None;
        }
        Ok(())
    }

    pub fn append_child(
        &mut self,
        parent: DomNodeId,
        child: DomNodeId,
    ) -> Result<(), DomError> {
        self.ensure_container(parent)?;
        self.node(child)?;
        self.ensure_no_cycle(parent, child)?;
        if parent == child {
            return Err(DomError::HierarchyRequest);
        }
        self.detach(child)?;
        self.node_mut(parent)?.children.push(child);
        self.node_mut(child)?.parent = Some(parent);
        Ok(())
    }

    pub fn insert_before(
        &mut self,
        parent: DomNodeId,
        child: DomNodeId,
        reference: Option<DomNodeId>,
    ) -> Result<(), DomError> {
        self.ensure_container(parent)?;
        self.node(child)?;
        self.ensure_no_cycle(parent, child)?;
        if let Some(reference_id) = reference {
            self.node(reference_id)?;
            if child == reference_id {
                return Ok(());
            }
            if self.node(reference_id)?.parent() != Some(parent) {
                return Err(DomError::ReferenceIsNotChild {
                    parent,
                    reference: reference_id,
                });
            }
        }

        self.detach(child)?;
        let position = match reference {
            Some(reference_id) => self
                .node(parent)?
                .children()
                .iter()
                .position(|candidate| *candidate == reference_id)
                .ok_or(DomError::ReferenceIsNotChild {
                    parent,
                    reference: reference_id,
                })?,
            None => self.node(parent)?.children().len(),
        };
        self.node_mut(parent)?.children.insert(position, child);
        self.node_mut(child)?.parent = Some(parent);
        Ok(())
    }

    pub fn remove_child(
        &mut self,
        parent: DomNodeId,
        child: DomNodeId,
    ) -> Result<DomNodeId, DomError> {
        let position = self
            .node(parent)?
            .children()
            .iter()
            .position(|candidate| *candidate == child)
            .ok_or(DomError::ReferenceIsNotChild {
                parent,
                reference: child,
            })?;
        self.node_mut(parent)?.children.remove(position);
        self.node_mut(child)?.parent = None;
        Ok(child)
    }

    pub fn set_attribute(
        &mut self,
        element: DomNodeId,
        name: &str,
        value: Option<&str>,
    ) -> Result<(), DomError> {
        let node = self.node_mut(element)?;
        let DomNodeKind::Element { attributes, .. } = &mut node.kind else {
            return Err(DomError::ParentCannotHaveChildren(element));
        };
        let name = name.to_ascii_lowercase();
        if let Some(attribute) = attributes.iter_mut().find(|attribute| attribute.name == name) {
            attribute.value = value.map(str::to_owned);
        } else {
            attributes.push(Attribute {
                name,
                value: value.map(str::to_owned),
            });
        }
        Ok(())
    }

    pub fn get_attribute(
        &self,
        element: DomNodeId,
        name: &str,
    ) -> Result<Option<&str>, DomError> {
        Ok(self.kind(element)?.attribute(name))
    }

    pub fn remove_attribute(
        &mut self,
        element: DomNodeId,
        name: &str,
    ) -> Result<Option<Attribute>, DomError> {
        let node = self.node_mut(element)?;
        let DomNodeKind::Element { attributes, .. } = &mut node.kind else {
            return Err(DomError::ParentCannotHaveChildren(element));
        };
        let Some(index) = attributes
            .iter()
            .position(|attribute| attribute.name.eq_ignore_ascii_case(name))
        else {
            return Ok(None);
        };
        Ok(Some(attributes.remove(index)))
    }

    pub fn find_by_id(&self, id: &str) -> Option<DomNodeId> {
        let mut pending = vec![self.document_node];
        while let Some(current) = pending.pop() {
            let node = self.nodes.get(current.index())?;
            if node.kind.attribute("id") == Some(id) {
                return Some(current);
            }
            pending.extend(node.children.iter().rev().copied());
        }
        None
    }

    pub fn elements_by_tag_name(&self, tag_name: &str) -> Vec<DomNodeId> {
        let mut matches = Vec::new();
        let mut pending = vec![self.document_node];
        while let Some(current) = pending.pop() {
            let Ok(node) = self.node(current) else {
                continue;
            };
            if node
                .kind
                .name()
                .is_some_and(|name| name.eq_ignore_ascii_case(tag_name))
            {
                matches.push(current);
            }
            pending.extend(node.children.iter().rev().copied());
        }
        matches
    }

    pub fn text_content(&self, root: DomNodeId) -> Result<String, DomError> {
        let mut output = String::new();
        let mut pending = vec![root];
        while let Some(current) = pending.pop() {
            let node = self.node(current)?;
            match &node.kind {
                DomNodeKind::Text(text) => output.push_str(text),
                DomNodeKind::Document | DomNodeKind::Element { .. } => {
                    pending.extend(node.children.iter().rev().copied());
                }
                DomNodeKind::Comment(_) | DomNodeKind::Doctype(_) => {}
            }
        }
        Ok(output)
    }

    pub fn to_tree(&self) -> Result<Node, DomError> {
        fn convert(document: &DomDocument, id: DomNodeId) -> Result<Node, DomError> {
            let node = document.node(id)?;
            let children = node
                .children
                .iter()
                .map(|child| convert(document, *child))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(match &node.kind {
                DomNodeKind::Document => Node::Document(children),
                DomNodeKind::Element { name, attributes } => Node::Element {
                    name: name.clone(),
                    attributes: attributes.clone(),
                    children,
                },
                DomNodeKind::Text(text) => Node::Text(text.clone()),
                DomNodeKind::Comment(text) => Node::Comment(text.clone()),
                DomNodeKind::Doctype(name) => Node::Doctype(name.clone()),
            })
        }

        convert(self, self.document_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_fragment, parse_document};

    #[test]
    fn stable_ids_survive_sibling_mutations() {
        let tree = parse_fragment("<main><a id='first'></a><b></b><c></c></main>");
        let mut document = DomDocument::from_tree(&tree).unwrap();
        let main = document.elements_by_tag_name("main")[0];
        let a = document.elements_by_tag_name("a")[0];
        let b = document.elements_by_tag_name("b")[0];
        let c = document.elements_by_tag_name("c")[0];

        assert_eq!(document.parent_node(a).unwrap(), Some(main));
        assert_eq!(document.previous_sibling(b).unwrap(), Some(a));
        assert_eq!(document.next_sibling(b).unwrap(), Some(c));

        let x = document.create_element("x").unwrap();
        document.insert_before(main, x, Some(b)).unwrap();
        assert_eq!(document.previous_sibling(b).unwrap(), Some(x));
        assert_eq!(document.node(a).unwrap().kind().name(), Some("a"));
    }

    #[test]
    fn moves_nodes_and_prevents_cycles() {
        let mut document = DomDocument::new();
        let parent = document.create_element("parent").unwrap();
        let child = document.create_element("child").unwrap();
        document.append_child(document.document_node(), parent).unwrap();
        document.append_child(parent, child).unwrap();

        assert_eq!(
            document.append_child(child, parent),
            Err(DomError::HierarchyRequest)
        );
        document.remove_child(parent, child).unwrap();
        assert_eq!(document.parent_node(child).unwrap(), None);
        assert_eq!(document.node(child).unwrap().kind().name(), Some("child"));
    }

    #[test]
    fn supports_attribute_and_subtree_queries() {
        let tree = parse_fragment("<main><p id='target'>hello</p></main>");
        let document = DomDocument::from_tree(&tree).unwrap();
        let target = document.find_by_id("target").unwrap();
        assert_eq!(document.text_content(target).unwrap(), "hello");
        assert_eq!(document.elements_by_tag_name("p"), vec![target]);
        assert_eq!(document.get_attribute(target, "id").unwrap(), Some("target"));
    }

    #[test]
    fn document_parser_inserts_html_head_and_body() {
        let tree = parse_document("<!doctype html><title>Test</title><p>Hello</p>");
        let html = tree.elements_by_tag_name("html");
        let head = tree.elements_by_tag_name("head");
        let body = tree.elements_by_tag_name("body");
        assert_eq!(html.len(), 1);
        assert_eq!(head.len(), 1);
        assert_eq!(body.len(), 1);
        assert_eq!(head[0].children()[0].name(), Some("title"));
        assert_eq!(body[0].children()[0].name(), Some("p"));
    }

    #[test]
    fn entity_table_handles_legacy_names_and_multi_codepoint_values() {
        assert_eq!(crate::decode_character_references("&copy © &NotEqualTilde;"), "© © ≂̸");
        assert_eq!(crate::decode_character_references("&NotEqualTilde;"), "≂̸");
        assert_eq!(
            crate::decode_character_references_in_attribute("&notit;"),
            "&notit;"
        );
    }
}
