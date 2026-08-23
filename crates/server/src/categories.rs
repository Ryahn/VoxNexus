//! Channel category CRUD (F026).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use uuid::Uuid;
use voxnexus_auth::{
    can_view_space, create_category as persist_create, delete_category as persist_delete,
    get_category, get_community, get_membership, get_space, list_categories as persist_list,
    update_category as persist_update, CategoryPatch, CreateCategoryInput,
};
use voxnexus_domain::{ChannelCategory, CommunityMemberRole};
use voxnexus_protocol::error_codes;
use voxnexus_protocol::{
    CategoryListResponse, CategoryResponse, CreateCategoryRequest, ListCategoriesQuery,
    ReorderCategoriesRequest, UpdateCategoryRequest,
};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::extract_auth::AuthUser;
use crate::http::{request_id_from_headers, AppState};

/// Create a category (owner / manage_channels until F029).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/categories",
    operation_id = "createCategory",
    tag = "categories",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn create_category(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(&state, community_id, user.account_id, &request_id).await?;
    let name = body.name.trim().to_owned();
    if name.is_empty() {
        return Err(validation(request_id, "Name is required."));
    }
    if let Some(space_id) = body.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        require_space_visible(&state, &space, user.account_id, &request_id).await?;
    }
    let category = persist_create(
        &state.pool,
        community_id,
        CreateCategoryInput {
            name,
            space_id: body.space_id,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok((StatusCode::CREATED, Json(to_response(&category))))
}

/// List categories in a community scope (`space_id` query = Space; omit for community-level).
#[utoipa::path(
    get,
    path = "/api/v1/communities/{community_id}/categories",
    operation_id = "listCategories",
    tag = "categories",
    params(
        ("community_id" = Uuid, Path, description = "Community id"),
        ListCategoriesQuery
    ),
    responses(
        (status = 200, description = "Category list", body = CategoryListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn list_categories(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<CategoryListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_member(&state, community_id, user.account_id, &request_id).await?;
    if let Some(space_id) = query.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "get space failed");
                internal(request_id.clone())
            })?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if space.community_id != community_id {
            return Err(not_found(request_id));
        }
        require_space_visible(&state, &space, user.account_id, &request_id).await?;
    }
    let categories = persist_list(&state.pool, community_id, query.space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list categories failed");
            internal(request_id)
        })?;
    Ok(Json(CategoryListResponse {
        categories: categories.iter().map(to_response).collect(),
    }))
}

/// Reorder categories in a scope by id list (owner).
#[utoipa::path(
    post,
    path = "/api/v1/communities/{community_id}/categories/reorder",
    operation_id = "reorderCategories",
    tag = "categories",
    params(("community_id" = Uuid, Path, description = "Community id")),
    request_body = ReorderCategoriesRequest,
    responses(
        (status = 200, description = "Reordered", body = CategoryListResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody),
        (status = 422, description = "Validation error", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn reorder_categories(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(community_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ReorderCategoriesRequest>,
) -> Result<Json<CategoryListResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    require_manage_channels(&state, community_id, user.account_id, &request_id).await?;
    let first = get_category(&state.pool, body.category_ids[0])
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    if first.community_id != community_id {
        return Err(not_found(request_id));
    }
    let scope_space_id = first.space_id;
    for (index, category_id) in body.category_ids.iter().enumerate() {
        let current = get_category(&state.pool, *category_id)
            .await
            .map_err(|error| map_auth(&error, request_id.clone()))?
            .ok_or_else(|| not_found(request_id.clone()))?;
        if current.community_id != community_id || current.space_id != scope_space_id {
            return Err(validation(
                request_id.clone(),
                "All categories must belong to the same scope.",
            ));
        }
        let position = i32::try_from(index).map_err(|_| {
            validation(
                request_id.clone(),
                "Category order index is out of range.",
            )
        })?;
        persist_update(
            &state.pool,
            *category_id,
            CategoryPatch {
                position: Some(position),
                ..CategoryPatch::default()
            },
        )
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    }
    let categories = persist_list(&state.pool, community_id, scope_space_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "list categories failed");
            internal(request_id)
        })?;
    Ok(Json(CategoryListResponse {
        categories: categories.iter().map(to_response).collect(),
    }))
}

/// Get one category.
#[utoipa::path(
    get,
    path = "/api/v1/categories/{category_id}",
    operation_id = "getCategory",
    tag = "categories",
    params(("category_id" = Uuid, Path, description = "Category id")),
    responses(
        (status = 200, description = "Category", body = CategoryResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not a member", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn get_category_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let category = get_category(&state.pool, category_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "get category failed");
            internal(request_id.clone())
        })?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_member(&state, category.community_id, user.account_id, &request_id).await?;
    if let Some(space_id) = category.space_id {
        let space = get_space(&state.pool, space_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "get space failed");
                internal(request_id.clone())
            })?
            .ok_or_else(|| not_found(request_id.clone()))?;
        require_space_visible(&state, &space, user.account_id, &request_id).await?;
    }
    Ok(Json(to_response(&category)))
}

/// Update a category (owner until F029).
#[utoipa::path(
    patch,
    path = "/api/v1/categories/{category_id}",
    operation_id = "updateCategory",
    tag = "categories",
    params(("category_id" = Uuid, Path, description = "Category id")),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Updated category", body = CategoryResponse),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn update_category(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_category(&state.pool, category_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    if body.name.is_none() && body.position.is_none() && body.space_id.is_none() {
        return Err(validation(
            request_id,
            "Provide at least one field to update.",
        ));
    }
    let category = persist_update(
        &state.pool,
        category_id,
        CategoryPatch {
            name: body.name.map(|value| value.trim().to_owned()),
            position: body.position,
            space_id: body.space_id,
        },
    )
    .await
    .map_err(|error| map_auth(&error, request_id))?;
    Ok(Json(to_response(&category)))
}

/// Delete a category (owner until F029).
#[utoipa::path(
    delete,
    path = "/api/v1/categories/{category_id}",
    operation_id = "deleteCategory",
    tag = "categories",
    params(("category_id" = Uuid, Path, description = "Category id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Not authenticated", body = voxnexus_protocol::ErrorBody),
        (status = 403, description = "Not allowed", body = voxnexus_protocol::ErrorBody),
        (status = 404, description = "Not found", body = voxnexus_protocol::ErrorBody)
    )
)]
pub async fn delete_category(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let request_id = request_id_from_headers(&headers);
    let current = get_category(&state.pool, category_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?
        .ok_or_else(|| not_found(request_id.clone()))?;
    require_manage_channels(&state, current.community_id, user.account_id, &request_id).await?;
    let deleted = persist_delete(&state.pool, category_id)
        .await
        .map_err(|error| map_auth(&error, request_id.clone()))?;
    if !deleted {
        return Err(not_found(request_id));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn require_member(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal(request_id.to_owned())
        })?;
    if membership.is_some() {
        return Ok(());
    }
    if get_community(&state.pool, community_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        Err(not_found(request_id.to_owned()))
    } else {
        Err(ApiError::permission_denied(
            request_id.to_owned(),
            "You must be a community member to view categories.",
        ))
    }
}

async fn require_manage_channels(
    state: &AppState,
    community_id: Uuid,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    let membership = get_membership(&state.pool, community_id, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "membership lookup failed");
            internal(request_id.to_owned())
        })?;
    match membership {
        Some(member) if member.role == CommunityMemberRole::Owner => Ok(()),
        Some(_) => Err(ApiError::permission_denied(
            request_id.to_owned(),
            "Only the community owner can manage categories.",
        )),
        None => {
            if get_community(&state.pool, community_id)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                Err(not_found(request_id.to_owned()))
            } else {
                Err(ApiError::permission_denied(
                    request_id.to_owned(),
                    "Only the community owner can manage categories.",
                ))
            }
        }
    }
}

async fn require_space_visible(
    state: &AppState,
    space: &voxnexus_domain::Space,
    account_id: Uuid,
    request_id: &str,
) -> Result<(), ApiError> {
    let visible = can_view_space(&state.pool, space, account_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "space visibility check failed");
            internal(request_id.to_owned())
        })?;
    if !visible {
        return Err(not_found(request_id.to_owned()));
    }
    Ok(())
}

fn to_response(category: &ChannelCategory) -> CategoryResponse {
    CategoryResponse {
        id: category.id,
        community_id: category.community_id,
        space_id: category.space_id,
        name: category.name.clone(),
        position: category.position,
        created_at: category.created_at,
        updated_at: category.updated_at,
    }
}

fn map_auth(error: &voxnexus_auth::AuthError, request_id: String) -> ApiError {
    match error {
        voxnexus_auth::AuthError::CategoryScopeMismatch => ApiError::permission_denied(
            request_id,
            "Category cannot be moved to a space outside this community.",
        ),
        other => {
            tracing::error!(error = %other, "category auth error");
            internal(request_id)
        }
    }
}

fn validation(request_id: String, message: impl Into<String>) -> ApiError {
    ApiError::new(
        StatusCode::BAD_REQUEST,
        error_codes::VALIDATION_ERROR,
        message,
        None,
        request_id,
    )
}

fn not_found(request_id: String) -> ApiError {
    ApiError::not_found(request_id)
}

fn internal(request_id: String) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::INTERNAL,
        "Unexpected server error.",
        None,
        request_id,
    )
}
