use axum::{extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}, Json};
use jrd::{JsonResourceDescriptor, JsonResourceDescriptorLink};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use upub::{model, Context};

use crate::ApiError;

#[derive(serde::Serialize)]
pub struct NodeInfoDiscovery {
	pub links: Vec<NodeInfoDiscoveryRel>,
}

#[derive(serde::Serialize)]
pub struct NodeInfoDiscoveryRel {
	pub rel: String,
	pub href: String,
}

pub async fn nodeinfo_discovery(State(ctx): State<Context>) -> Json<NodeInfoDiscovery> {
	Json(NodeInfoDiscovery {
		links: vec![
			NodeInfoDiscoveryRel {
				rel: "http://nodeinfo.diaspora.software/ns/schema/2.0".into(),
				href: upub::url!(ctx, "/nodeinfo/2.0.json"),
			},
			NodeInfoDiscoveryRel {
				rel: "http://nodeinfo.diaspora.software/ns/schema/2.1".into(),
				href: upub::url!(ctx, "/nodeinfo/2.1.json"),
			},
		],
	})
}

// TODO either vendor or fork nodeinfo-rs because it still represents "repository" and "homepage"
// even if None! technically leads to invalid nodeinfo 2.0
pub async fn nodeinfo(State(ctx): State<Context>, Path(version): Path<String>) -> Result<Json<nodeinfo::NodeInfoOwned>, StatusCode> {
	// TODO it's unsustainable to count these every time, especially comments since it's a complex
	// filter! keep these numbers caches somewhere, maybe db, so that we can just look them up
	let total_users = model::actor::Entity::find().count(ctx.db()).await.ok();
	let total_posts = None;
	let total_comments = None; 
	let (software, version) = match version.as_str() {
		"2.0.json" | "2.0" => (
			nodeinfo::types::Software {
				name: "μpub".to_string(),
				version: Some(upub::VERSION.into()),
				repository: None,
				homepage: None,
			},
			"2.0".to_string()
		),
		"2.1.json" | "2.1" => (
			nodeinfo::types::Software {
				name: "μpub".to_string(),
				version: Some(upub::VERSION.into()),
				repository: Some("https://git.alemi.dev/upub.git/".into()),
				homepage: None,
			},
			"2.1".to_string()
		),
		_ => return Err(StatusCode::NOT_IMPLEMENTED),
	};
	Ok(Json(
		nodeinfo::NodeInfoOwned {
			version,
			software,
			open_registrations: false,
			protocols: vec!["activitypub".into()],
			services: nodeinfo::types::Services {
				inbound: vec![],
				outbound: vec![],
			},
			usage: nodeinfo::types::Usage {
				local_posts: total_posts,
				local_comments: total_comments,
				users: Some(nodeinfo::types::Users {
					active_month: None,
					active_halfyear: None,
					total: total_users.map(|x| x as i64),
				}),
			},
			metadata: serde_json::Map::default(),
		}
	))
}


#[derive(Debug, serde::Deserialize)]
pub struct WebfingerQuery {
	pub resource: String,
}

pub struct JsonRD<T>(pub T);
impl<T: serde::Serialize> IntoResponse for JsonRD<T> {
	fn into_response(self) -> Response {
		([("Content-Type", "application/jrd+json")], Json(self.0)).into_response()
	}
}

pub async fn webfinger(
	State(ctx): State<Context>,
	Query(query): Query<WebfingerQuery>
) -> crate::ApiResult<JsonRD<JsonResourceDescriptor>> {
	let user =
		if query.resource.starts_with("acct:") {
			if let Some((user, domain)) = query
				.resource
				.replace("acct:", "")
				.split_once('@')
			{
				model::actor::Entity::find()
					.filter(model::actor::Column::PreferredUsername.eq(user))
					.filter(model::actor::Column::Domain.eq(domain))
					.one(ctx.db())
					.await?
					.ok_or_else(crate::ApiError::not_found)?

			} else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
			}
		} else if query.resource.starts_with("http") {
			match model::actor::Entity::find_by_ap_id(&query.resource)
				.one(ctx.db())
				.await?
			{
				Some(usr) => usr,
				None => return Err(ApiError::not_found()),
			}
		} else {
			return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
		};

	let expires = if user.domain == ctx.domain() {
		// TODO configurable webfinger TTL, also 30 days may be too much???
		Some(chrono::Utc::now() + chrono::Duration::days(30))
	} else {
		// we are no authority on local users, this info should be considered already outdated,
		// but can still be relevant, for example for our frontend
		Some(chrono::Utc::now())
	};
	
	Ok(JsonRD(JsonResourceDescriptor {
		subject: format!("acct:{}@{}", user.preferred_username, user.domain),
		aliases: vec![user.id.clone()],
		links: vec![
			JsonResourceDescriptorLink {
				rel: "self".to_string(),
				link_type: Some("application/ld+json".to_string()),
				href: Some(user.id),
				properties: jrd::Map::default(),
				titles: jrd::Map::default(),
			},
		],
		properties: jrd::Map::default(),
		expires,
	}))
}

pub async fn manifest(State(ctx): State<Context>) -> Json<ManifestResponse> {
	axum::Json(ManifestResponse {
		id: ctx.cfg().instance.domain.clone(),
		name: ctx.cfg().instance.name.clone(),
		short_name: ctx.cfg().instance.name.clone(),
		description: ctx.cfg().instance.description.clone(),
		start_url: "/web".to_string(),
		scope: format!("https://{}/web", ctx.cfg().instance.domain),
		display: "standalone".to_string(),
		background_color: "#201f29".to_string(),
		theme_color: "#bf616a".to_string(),
		orientation: "portrait-primary".to_string(),
		icons: vec![],
		shortcuts: vec![],
		categories: vec!["social".to_string()]
	})
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ManifestResponse {
		background_color: String,
		categories: Vec<String>,
		description: String,
		display: String, // "fullscreen", "standalone", "minima-ui", "browser"
		icons: Vec<String>, // TODO Vec of objects: {stc: string, sizes: string, type: string? }
		id: String,
		name: String,
		orientation: String, // "any", "natural", "landscape", "landscape-primary", "landscape-secondary", "portrait", "portrait-primary", "portrait-secondary"
		scope: String,
		short_name: String,
		shortcuts: Vec<String>, // TODO Vec of objects: {name: string, url: string, description: string?}
		start_url: String,
		theme_color: String,
}

// i don't even want to bother with XML, im just returning a formatted xml string
pub async fn host_meta(State(ctx): State<Context>) -> Response {
	(
		[("Content-Type", "application/xrd+xml")],
		format!(r#"<?xml version="1.0" encoding="UTF-8"?>
			<XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
				<Link type="application/xrd+xml" template="{}{}/.well-known/webfinger?resource={{uri}}" rel="lrdd" />
			</XRD>"#,
			ctx.protocol(), ctx.domain())
	).into_response()
}

#[derive(Debug, serde::Serialize)]
pub struct OauthAuthorizationServerResponse {
	issuer: String,
	authorization_endpoint: String,
	token_endpoint: String,
	scopes_supported: Vec<String>,
	response_types_supported: Vec<String>,
	grant_types_supported: Vec<String>,
	service_documentation: String,
	code_challenge_methods_supported: Vec<String>,
	authorization_response_iss_parameter_supported: bool,
}

pub async fn oauth_authorization_server(State(ctx): State<Context>) -> crate::ApiResult<Json<OauthAuthorizationServerResponse>> {
	Ok(Json(OauthAuthorizationServerResponse {
		issuer: upub::url!(ctx, ""),
		authorization_endpoint: upub::url!(ctx, "/auth"),
		token_endpoint: "".to_string(),
		scopes_supported: vec![
			"read:account".to_string(),
			"write:account".to_string(),
			"read:favorites".to_string(),
			"write:favorites".to_string(),
			"read:following".to_string(),
			"write:following".to_string(),
			"write:notes".to_string(),
		],
		response_types_supported: vec!["code".to_string()],
		grant_types_supported: vec!["authorization_code".to_string()],
		service_documentation: "".to_string(),
		code_challenge_methods_supported: vec![],
		authorization_response_iss_parameter_supported: false,
	}))
}
