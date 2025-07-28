// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::common::auth::ClientContext;
use crate::modules::error::code::ErrorCode;
use crate::modules::oauth2::entity::{OAuth2, OAuth2CreateRequest, OAuth2UpdateRequest};
use crate::modules::oauth2::flow::{AuthorizeUrlRequest, OAuth2Flow};
use crate::modules::oauth2::token::OAuth2AccessToken;
use crate::modules::rest::api::ApiTags;
use crate::modules::rest::response::DataPage;
use crate::modules::rest::ApiResult;
use crate::raise_error;
use poem::web::Path;
use poem_openapi::param::Query;
use poem_openapi::payload::{Json, PlainText};
use poem_openapi::OpenApi;

pub struct OAuth2Api;

#[OpenApi(prefix_path = "/api/v1", tag = "ApiTags::OAuth2")]
impl OAuth2Api {
    /// Retrieves the OAuth2 configuration for a specified name.
    ///
    /// Requires root privileges.
    /// This endpoint fetches the OAuth2 configuration identified by the given name.
    #[oai(
        path = "/oauth2/:id",
        method = "get",
        operation_id = "get_oauth2_config"
    )]
    async fn get_oauth2_config(
        &self,
        /// The name of the OAuth2 configuration to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<OAuth2>> {
        context.require_root()?;
        let id = id.0;
        Ok(Json(OAuth2::get(id).await?.ok_or_else(|| {
            raise_error!(
                format!("OAuth2 configuration id='{id}' not found"),
                ErrorCode::ResourceNotFound
            )
        })?))
    }

    /// Deletes an OAuth2 configuration by name.
    ///
    /// Requires root privileges.
    /// This endpoint removes the OAuth2 configuration identified by the specified name.
    #[oai(
        path = "/oauth2/:id",
        method = "delete",
        operation_id = "remove_oauth2_config"
    )]
    async fn remove_oauth2_config(
        &self,
        /// The name of the OAuth2 configuration to retrieve
        id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(OAuth2::delete(id.0).await?)
    }

    /// Creates a new OAuth2 configuration.
    ///
    /// Requires root privileges.
    /// This endpoint creates a new OAuth2 configuration based on the provided request data.
    #[oai(
        path = "/oauth2",
        method = "post",
        operation_id = "create_oauth2_config"
    )]
    async fn create_oauth2_config(
        &self,
        /// A JSON payload containing the details for the new OAuth2 configuration
        request: Json<OAuth2CreateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        let entity = OAuth2::new(request.0)?;
        Ok(entity.save().await?)
    }

    /// Updates an existing OAuth2 configuration.
    ///
    /// Requires root privileges.
    /// This endpoint updates the OAuth2 configuration identified by the specified name
    #[oai(
        path = "/oauth2/:id",
        method = "post",
        operation_id = "update_oauth2_config"
    )]
    async fn update_oauth2_config(
        &self,
        /// The name of the OAuth2 configuration to update
        id: Path<u64>,
        /// A JSON payload containing the updated configuration details
        payload: Json<OAuth2UpdateRequest>,
        context: ClientContext,
    ) -> ApiResult<()> {
        context.require_root()?;
        Ok(OAuth2::update(id.0, payload.0).await?)
    }

    /// Lists OAuth2 configurations with pagination and sorting options.
    ///
    /// This endpoint retrieves a paginated list of OAuth2 configurations, allowing for
    /// optional pagination and sorting parameters. It requires root access.
    #[oai(
        path = "/oauth2-list",
        method = "get",
        operation_id = "list_oauth2_config"
    )]
    async fn list_oauth2_config(
        &self,
        /// Optional. The page number to retrieve (starting from 1).
        page: Query<Option<u64>>,
        /// Optional. The number of items per page.
        page_size: Query<Option<u64>>,
        /// Optional. Whether to sort the list in descending order.
        desc: Query<Option<bool>>,
        context: ClientContext,
    ) -> ApiResult<Json<DataPage<OAuth2>>> {
        context.require_root()?;
        Ok(Json(
            OAuth2::paginate_list(page.0, page_size.0, desc.0).await?,
        ))
    }

    /// Generates an OAuth2 authorization URL for a specific account.
    ///
    /// This endpoint creates an authorization URL for the specified OAuth2 configuration
    /// and account ID. It requires root access and returns the URL as plain text.
    #[oai(
        path = "/oauth2-authorize-url",
        method = "post",
        operation_id = "create_oauth2_authorize_url"
    )]
    async fn create_oauth2_authorize_url(
        &self,
        /// A JSON payload containing the OAuth2 configuration name and account ID.
        request: Json<AuthorizeUrlRequest>,
        context: ClientContext,
    ) -> ApiResult<PlainText<String>> {
        context.require_root()?;
        let request = request.0;
        let flow = OAuth2Flow::new(request.oauth2_id);
        Ok(PlainText(flow.authorize_url(request.account_id).await?))
    }

    /// Retrieves OAuth2 access tokens for a specified account.
    ///
    /// This endpoint fetches the OAuth2 access tokens associated with the given account ID.
    #[oai(
        path = "/oauth2-tokens/:account_id",
        method = "get",
        operation_id = "get_oauth2_tokens"
    )]
    async fn get_oauth2_tokens(
        &self,
        /// The ID of the account to retrieve access tokens for
        account_id: Path<u64>,
        context: ClientContext,
    ) -> ApiResult<Json<OAuth2AccessToken>> {
        let account = account_id.0;
        context.require_account_access(account)?;
        Ok(Json(OAuth2AccessToken::get(account).await?.ok_or_else(
            || {
                raise_error!(
                    "OAuth2 access tokens not found".into(),
                    ErrorCode::ResourceNotFound
                )
            },
        )?))
    }
}
