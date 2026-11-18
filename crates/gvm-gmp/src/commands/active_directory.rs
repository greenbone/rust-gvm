// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

//! Live Active Directory discovery; this command does not persist targets.

use crate::common::set_optional_bool_attr;
use crate::responses::active_directory::GetActiveDirectoryObjectsResponse;
use crate::{EntityId, GmpCommand, GmpRequest, GmpRequestCodec, GmpRequestError, GmpVersion};
use gvm_protocol::{Request as _, XmlCommand};

/// LDAP search scope relative to the selected base DN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActiveDirectorySearchScope {
    /// Search only the base object.
    Base,
    /// Search immediate children (the server default).
    OneLevel,
    /// Search the base and all descendants.
    Subtree,
}

impl ActiveDirectorySearchScope {
    /// GMP attribute value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::OneLevel => "onelevel",
            Self::Subtree => "subtree",
        }
    }
}

/// An Active Directory object category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActiveDirectoryObjectType {
    /// Organizational unit.
    Ou,
    /// Directory container (not included in the server's default selection).
    Container,
    /// Computer account.
    Computer,
    /// Directory group.
    Group,
}

impl ActiveDirectoryObjectType {
    /// GMP object-type value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ou => "ou",
            Self::Container => "container",
            Self::Computer => "computer",
            Self::Group => "group",
        }
    }
}

/// Discover live directory objects using a saved AD credential.
///
/// Omitted options retain server defaults. DN containment, direct group
/// membership, destination policy and resource visibility are enforced by gvmd.
#[derive(Debug, Clone)]
pub struct GetActiveDirectoryObjectsRequest {
    /// Saved Active Directory credential identifier.
    pub credential_id: EntityId,
    /// Defaults to onelevel when omitted.
    pub scope: Option<ActiveDirectorySearchScope>,
    /// Fetch limit, from 1 through 1000; omitted uses the server default.
    pub max_results: Option<u32>,
    /// Include disabled computers; the server defaults to true.
    pub include_disabled_computers: Option<bool>,
    /// Expand readable target matches; defaults to false.
    pub targets: Option<bool>,
    /// Expand readable tasks/configurations; requires targets=true.
    pub tasks: Option<bool>,
    /// Expand OS version and service principal names; defaults to false.
    pub details: Option<bool>,
    /// Optional nonempty search base; defaults to the credential base DN.
    pub base_dn: Option<String>,
    /// Optional nonempty DN selecting explicit direct group members.
    pub group_dn: Option<String>,
    /// Nonempty selection; omitted defaults to OU, computer and group.
    pub object_types: Option<Vec<ActiveDirectoryObjectType>>,
}

impl GetActiveDirectoryObjectsRequest {
    /// Create a discovery request retaining all server defaults.
    #[must_use]
    pub fn new(credential_id: EntityId) -> Self {
        Self {
            credential_id,
            scope: None,
            max_results: None,
            include_disabled_computers: None,
            targets: None,
            tasks: None,
            details: None,
            base_dn: None,
            group_dn: None,
            object_types: None,
        }
    }
}

impl GmpRequestCodec for GetActiveDirectoryObjectsRequest {
    fn validate(&self) -> Result<(), GmpRequestError> {
        if self
            .max_results
            .is_some_and(|limit| !(1..=1000).contains(&limit))
        {
            return Err(GmpRequestError::invalid_field(
                "max_results",
                "must be between 1 and 1000",
            ));
        }
        for (field, value) in [("base_dn", &self.base_dn), ("group_dn", &self.group_dn)] {
            if value.as_ref().is_some_and(String::is_empty) {
                return Err(GmpRequestError::invalid_field(
                    field,
                    "must not be empty when supplied",
                ));
            }
        }
        if self.object_types.as_ref().is_some_and(Vec::is_empty) {
            return Err(GmpRequestError::invalid_field(
                "object_types",
                "must contain at least one object type",
            ));
        }
        if self.tasks == Some(true) && self.targets != Some(true) {
            return Err(GmpRequestError::invalid_combination(
                &["tasks", "targets"],
                "tasks requires targets=true",
            ));
        }
        Ok(())
    }

    fn command(&self) -> Option<GmpCommand> {
        Some(GmpCommand::new("get_active_directory_objects"))
    }

    fn encode(&self, _version: GmpVersion) -> Result<Vec<u8>, GmpRequestError> {
        self.validate()?;
        let mut cmd = XmlCommand::new("get_active_directory_objects")
            .attribute("credential_id", self.credential_id.as_str());
        if let Some(scope) = self.scope {
            cmd = cmd.attribute("scope", scope.as_str());
        }
        if let Some(limit) = self.max_results {
            cmd = cmd.attribute("max_results", &limit.to_string());
        }
        set_optional_bool_attr(
            &mut cmd,
            "include_disabled_computers",
            self.include_disabled_computers,
        );
        set_optional_bool_attr(&mut cmd, "targets", self.targets);
        set_optional_bool_attr(&mut cmd, "tasks", self.tasks);
        set_optional_bool_attr(&mut cmd, "details", self.details);
        if let Some(dn) = &self.base_dn {
            cmd.add_element_with_text("base_dn", dn);
        }
        if let Some(dn) = &self.group_dn {
            cmd.add_element_with_text("group_dn", dn);
        }
        if let Some(types) = &self.object_types {
            let node = cmd.add_element("object_types");
            // gvmd interprets selection as a bitmask; preserve first occurrence order.
            for (index, kind) in types.iter().enumerate() {
                if !types[..index].contains(kind) {
                    node.add_child_with_text("object_type", kind.as_str());
                }
            }
        }
        Ok(cmd.to_bytes())
    }
}

impl GmpRequest for GetActiveDirectoryObjectsRequest {
    type Response = GetActiveDirectoryObjectsResponse;
}
