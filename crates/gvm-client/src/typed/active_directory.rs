// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

use crate::{GmpClient, GvmError};
use gvm_connection::GvmConnection;
use gvm_gmp::commands::active_directory::GetActiveDirectoryObjectsRequest;
use gvm_gmp::responses::active_directory::GetActiveDirectoryObjectsResponse;

impl<C: GvmConnection + Send> GmpClient<C> {
    /// Discover live Active Directory objects without persisting targets.
    ///
    /// # Errors
    /// Returns validation, capability, transport, server, or response parsing errors.
    pub async fn get_active_directory_objects(
        &mut self,
        request: GetActiveDirectoryObjectsRequest,
    ) -> Result<GetActiveDirectoryObjectsResponse, GvmError> {
        self.execute(request).await
    }
}
