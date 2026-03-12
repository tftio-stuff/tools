//! High level project operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{
        MemberPermission, Project, ProjectCreateRequest, ProjectListParams, ProjectMember,
        ProjectMembers, ProjectSort, ProjectStatus, ProjectUpdateRequest,
    },
};
use futures_util::{StreamExt, pin_mut};
use serde::{Deserialize, Serialize};

/// Retrieve projects according to the supplied parameters.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn list_projects(
    client: &ApiClient,
    mut params: ProjectListParams,
) -> Result<Vec<Project>, ApiError> {
    ensure_default_fields(&mut params);

    let query = params.to_query();
    let max_items = params.limit;
    let stream = client.paginate_with_limit::<Project>("/projects", query, max_items);
    pin_mut!(stream);

    let mut projects = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        projects.append(&mut page);
    }

    if !params.filters.is_empty() {
        projects.retain(|project| project.matches(&params.filters));
    }

    if let Some(sort) = params.sort {
        sort_projects(&mut projects, sort);
    }

    Ok(projects)
}

/// Retrieve a single project by gid.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn get_project(
    client: &ApiClient,
    gid: &str,
    fields: Vec<String>,
) -> Result<Project, ApiError> {
    let mut query = Vec::new();
    if !fields.is_empty() {
        query.push(("opt_fields".into(), fields.join(",")));
    }

    let response: SingleProjectResponse = client
        .get_json_with_pairs(&format!("/projects/{gid}"), query)
        .await?;
    Ok(response.data)
}

/// Create a project using the provided payload.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn create_project(
    client: &ApiClient,
    request: ProjectCreateRequest,
) -> Result<Project, ApiError> {
    let response: SingleProjectResponse = client.post_json("/projects", &request).await?;
    Ok(response.data)
}

/// Update a project using the given payload.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn update_project(
    client: &ApiClient,
    gid: &str,
    request: ProjectUpdateRequest,
) -> Result<Project, ApiError> {
    let response: SingleProjectResponse = client
        .put_json(&format!("/projects/{gid}"), &request)
        .await?;
    Ok(response.data)
}

/// Delete a project permanently.
///
/// # Errors
///
/// Returns an error if the API request fails or if the response is invalid.
pub async fn delete_project(client: &ApiClient, gid: &str) -> Result<(), ApiError> {
    client.delete(&format!("/projects/{gid}"), Vec::new()).await
}

/// List project members.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn list_members(client: &ApiClient, gid: &str) -> Result<ProjectMembers, ApiError> {
    let response: MembersResponse = client
        .get_json_with_pairs(&format!("/projects/{gid}/members"), vec![])
        .await?;
    Ok(ProjectMembers {
        project_gid: gid.to_string(),
        members: response.data,
    })
}

/// Retrieve recent project status updates.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn list_statuses(
    client: &ApiClient,
    gid: &str,
    limit: Option<usize>,
) -> Result<Vec<ProjectStatus>, ApiError> {
    let response: StatusResponse = client
        .get_json_with_pairs(&format!("/projects/{gid}/project_statuses"), vec![])
        .await?;
    let mut statuses = response.data;
    if let Some(limit) = limit
        && statuses.len() > limit
    {
        statuses.truncate(limit);
    }
    Ok(statuses)
}

/// Add members to a project.
///
/// # Errors
///
/// Returns an error if the API request fails or if the response is invalid.
pub async fn add_members(
    client: &ApiClient,
    gid: &str,
    members: Vec<String>,
    role: Option<MemberPermission>,
) -> Result<(), ApiError> {
    let payload = ModifyMembersRequest {
        data: ModifyMembersData { members, role },
    };
    client
        .post_void(&format!("/projects/{gid}/addMembers"), &payload)
        .await
}

/// Remove members from a project.
///
/// # Errors
///
/// Returns an error if the API request fails or if the response is invalid.
pub async fn remove_members(
    client: &ApiClient,
    gid: &str,
    members: Vec<String>,
) -> Result<(), ApiError> {
    let payload = ModifyMembersRequest {
        data: ModifyMembersData {
            members,
            role: None,
        },
    };
    client
        .post_void(&format!("/projects/{gid}/removeMembers"), &payload)
        .await
}

/// Update a specific project member's permission level.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn update_member(
    client: &ApiClient,
    membership_gid: &str,
    role: MemberPermission,
) -> Result<ProjectMember, ApiError> {
    let payload = UpdateMemberRequest {
        data: MemberRoleUpdate { role },
    };
    let response: SingleMemberResponse = client
        .put_json(&format!("/project_memberships/{membership_gid}"), &payload)
        .await?;
    Ok(response.data)
}

fn ensure_default_fields(params: &mut ProjectListParams) {
    let defaults = [
        "gid",
        "name",
        "archived",
        "color",
        "created_at",
        "modified_at",
        "workspace.name",
        "workspace.gid",
        "team.name",
        "team.gid",
        "owner.name",
        "owner.gid",
        "due_on",
        "start_on",
    ];
    for field in defaults {
        params.fields.insert(field.to_string());
    }
}

fn sort_projects(projects: &mut [Project], sort: ProjectSort) {
    match sort {
        ProjectSort::Name => projects.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        }),
        ProjectSort::CreatedAt => projects.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        ProjectSort::ModifiedAt => projects.sort_by(|a, b| a.modified_at.cmp(&b.modified_at)),
    }
}

#[derive(Debug, Deserialize)]
struct SingleProjectResponse {
    data: Project,
}

#[derive(Debug, Deserialize)]
struct MembersResponse {
    data: Vec<ProjectMember>,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    data: Vec<ProjectStatus>,
}

#[derive(Debug, Serialize)]
struct ModifyMembersRequest {
    data: ModifyMembersData,
}

#[derive(Debug, Serialize)]
struct ModifyMembersData {
    members: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<MemberPermission>,
}

#[derive(Debug, Serialize)]
struct UpdateMemberRequest {
    data: MemberRoleUpdate,
}

#[derive(Debug, Serialize)]
struct MemberRoleUpdate {
    role: MemberPermission,
}

#[derive(Debug, Deserialize)]
struct SingleMemberResponse {
    data: ProjectMember,
}
