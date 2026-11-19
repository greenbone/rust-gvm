// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

//! Live AD discovery projections from the current gvmd collection response.

use crate::commands::active_directory::ActiveDirectoryObjectType;
use crate::responses::common::{
    parse_bool, parse_document, parse_entity_id, parse_u32, status_from_response, NamedEntity,
    ParseError, XmlNode,
};
use crate::{EntityId, GmpResponse, GmpVersion};
use gvm_protocol::Response;

/// Counts of computers in the fetched subset, not directory-wide totals.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryComputerCounts {
    pub total: u32,
    /// Omitted when disabled computers were excluded.
    pub disabled: Option<u32>,
}

/// Per-type counts within the fetched subset.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryObjectCounts {
    pub ou: u32,
    pub container: u32,
    pub group: u32,
    pub computer: ActiveDirectoryComputerCounts,
}

/// Readable non-trash task associated with a matching target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryTask {
    pub id: EntityId,
    pub name: String,
    /// Present only when the configuration is readable and non-trash.
    pub config: Option<NamedEntity>,
}

/// Readable existing target matched to a discovered computer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryTarget {
    pub id: EntityId,
    pub name: String,
    /// None means not requested; Some(empty) means no readable matching tasks.
    pub tasks: Option<Vec<ActiveDirectoryTask>>,
}

/// A live directory object, with optional computer-specific information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryObject {
    pub object_type: ActiveDirectoryObjectType,
    pub name: String,
    pub distinguished_name: String,
    /// Empty when unavailable.
    pub parent_dn: String,
    /// Empty when unavailable.
    pub description: String,
    pub dns_hostname: Option<String>,
    pub operating_system: Option<String>,
    pub operating_system_version: Option<String>,
    /// None means omitted; Some(empty) preserves an explicitly empty expansion.
    pub service_principal_names: Option<Vec<String>>,
    /// May be omitted when account-control information is unavailable.
    pub disabled: Option<bool>,
    /// None means not requested; Some(empty) means no readable target matches.
    pub targets: Option<Vec<ActiveDirectoryTarget>>,
}

/// The objects collection, preserving truncation even for an empty result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActiveDirectoryObjects {
    pub truncated: bool,
    pub counts: ActiveDirectoryObjectCounts,
    pub items: Vec<ActiveDirectoryObject>,
}

/// Discovery response. The schema permits omission of the objects collection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetActiveDirectoryObjectsResponse {
    pub status: u16,
    pub status_text: String,
    pub objects: Option<ActiveDirectoryObjects>,
}

fn required_child<'a>(node: &'a XmlNode, field: &str) -> Result<&'a XmlNode, ParseError> {
    node.child(field)
        .ok_or_else(|| ParseError::MissingElement(field.into()))
}

fn required_attr<'a>(node: &'a XmlNode, field: &str) -> Result<&'a str, ParseError> {
    node.attr(field)
        .ok_or_else(|| ParseError::MissingElement(field.into()))
}

fn reference(node: &XmlNode) -> Result<NamedEntity, ParseError> {
    Ok(NamedEntity {
        id: parse_entity_id(required_attr(node, "id")?, "id")?,
        name: node.required_child_text("name")?,
    })
}

impl ActiveDirectoryTask {
    fn from_node(node: &XmlNode) -> Result<Self, ParseError> {
        let entity = reference(node)?;
        Ok(Self {
            id: entity.id,
            name: entity.name,
            config: node.child("config").map(reference).transpose()?,
        })
    }
}

impl ActiveDirectoryTarget {
    fn from_node(node: &XmlNode) -> Result<Self, ParseError> {
        let entity = reference(node)?;
        Ok(Self {
            id: entity.id,
            name: entity.name,
            tasks: node
                .child("tasks")
                .map(|tasks| {
                    tasks
                        .children_named("task")
                        .map(ActiveDirectoryTask::from_node)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
        })
    }
}

impl ActiveDirectoryObject {
    fn from_node(node: &XmlNode) -> Result<Self, ParseError> {
        let kind = required_attr(node, "type")?;
        let object_type = match kind {
            "ou" => ActiveDirectoryObjectType::Ou,
            "container" => ActiveDirectoryObjectType::Container,
            "computer" => ActiveDirectoryObjectType::Computer,
            "group" => ActiveDirectoryObjectType::Group,
            _ => {
                return Err(ParseError::InvalidValue {
                    field: "object.type".into(),
                    value: kind.into(),
                })
            }
        };
        Ok(Self {
            object_type,
            name: node.required_child_text("name")?,
            distinguished_name: node.required_child_text("distinguished_name")?,
            parent_dn: node.required_child_text("parent_dn")?,
            description: node.required_child_text("description")?,
            dns_hostname: node.child_text("dns_hostname"),
            operating_system: node.child_text("operating_system"),
            operating_system_version: node.child_text("operating_system_version"),
            service_principal_names: node.child("service_principal_names").map(|names| {
                names
                    .children_named("service_principal_name")
                    .map(|name| name.text.clone())
                    .collect()
            }),
            disabled: node
                .child_text("disabled")
                .map(|v| parse_bool(&v, "object.disabled"))
                .transpose()?,
            targets: node
                .child("targets")
                .map(|targets| {
                    targets
                        .children_named("target")
                        .map(ActiveDirectoryTarget::from_node)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
        })
    }
}

impl ActiveDirectoryObjects {
    fn from_node(node: &XmlNode) -> Result<Self, ParseError> {
        let counts = required_child(node, "count")?;
        let computer = required_child(counts, "computer")?;
        Ok(Self {
            truncated: parse_bool(required_attr(node, "truncated")?, "objects.truncated")?,
            counts: ActiveDirectoryObjectCounts {
                ou: parse_u32(&counts.required_child_text("ou")?, "count.ou")?,
                container: parse_u32(&counts.required_child_text("container")?, "count.container")?,
                group: parse_u32(&counts.required_child_text("group")?, "count.group")?,
                computer: ActiveDirectoryComputerCounts {
                    total: parse_u32(
                        &computer.required_child_text("total")?,
                        "count.computer.total",
                    )?,
                    disabled: computer
                        .child_text("disabled")
                        .map(|v| parse_u32(&v, "count.computer.disabled"))
                        .transpose()?,
                },
            },
            items: node
                .children_named("object")
                .map(ActiveDirectoryObject::from_node)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl GetActiveDirectoryObjectsResponse {
    /// Decode a server response, retaining server error status and text.
    pub fn from_response(response: &Response) -> Result<Self, ParseError> {
        let (status, status_text) = status_from_response(response)?;
        let root = parse_document(response.data())?;
        Ok(Self {
            status,
            status_text,
            objects: root
                .child("objects")
                .map(ActiveDirectoryObjects::from_node)
                .transpose()?,
        })
    }
}

impl GmpResponse for GetActiveDirectoryObjectsResponse {
    fn decode(response: &Response, _version: GmpVersion) -> Result<Self, ParseError> {
        Self::from_response(response)
    }
}
