use std::sync::atomic::AtomicI64;

use axum::{extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response}, Json};
use jrd::{JsonResourceDescriptor, JsonResourceDescriptorLink};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};
use upub::{model, Context};

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
pub async fn nodeinfo(State(ctx): State<Context>, Path(version): Path<String>) -> crate::ApiResult<Json<nodeinfo_upub::NodeInfoOwned>> {
	// keep these as statics so they get calculated once and then stay cached
	// TODO this will cache them just once per runtime, maybe re-calculate them after some time?
	static TOTAL_USERS: AtomicI64 = AtomicI64::new(i64::MIN);
	static TOTAL_POSTS: AtomicI64 = AtomicI64::new(i64::MIN);
	static TOTAL_COMMENTS: AtomicI64 = AtomicI64::new(i64::MIN);
	static TOTAL_ACTIVE_USERS_MONTH: AtomicI64 = AtomicI64::new(i64::MIN);
	static TOTAL_ACTIVE_USERS_HALFYEAR: AtomicI64 = AtomicI64::new(i64::MIN);

	// TODO because we need to get the actual numbers with async operations we can't use OnceLocks...
	//      can we make the following lines way more compact?? this is hell to maintain
	let mut total_users = TOTAL_USERS.load(std::sync::atomic::Ordering::Relaxed);
	if total_users == i64::MIN {
		let actual_total_users = model::actor::Entity::find()
			.filter(model::actor::Column::Domain.eq(ctx.domain()))
			.count(ctx.db())
			.await? as i64; // TODO safe cast
		TOTAL_USERS.store(actual_total_users, std::sync::atomic::Ordering::Relaxed);
		total_users = actual_total_users;
	}

	let mut total_posts = TOTAL_POSTS.load(std::sync::atomic::Ordering::Relaxed);
	if total_posts == i64::MIN {
		let actual_total_posts = model::object::Entity::find()
			.inner_join(model::actor::Entity)
			.filter(model::actor::Column::Domain.eq(ctx.domain()))
			.filter(model::object::Column::InReplyTo.is_null())
			.count(ctx.db())
			.await? as i64; // TODO safe cast
		TOTAL_POSTS.store(actual_total_posts, std::sync::atomic::Ordering::Relaxed);
		total_posts = actual_total_posts;
	}

	let mut total_comments = TOTAL_COMMENTS.load(std::sync::atomic::Ordering::Relaxed);
	if total_comments == i64::MIN {
		let actual_total_comments = model::object::Entity::find()
			.inner_join(model::actor::Entity)
			.filter(model::actor::Column::Domain.eq(ctx.domain()))
			.filter(model::object::Column::InReplyTo.is_not_null())
			.count(ctx.db())
			.await? as i64; // TODO safe cast
		TOTAL_COMMENTS.store(actual_total_comments, std::sync::atomic::Ordering::Relaxed);
		total_comments = actual_total_comments;
	}

	let mut total_active_users_month = TOTAL_ACTIVE_USERS_MONTH.load(std::sync::atomic::Ordering::Relaxed);
	if total_active_users_month == i64::MIN {
		let actual_total_active_users_month = model::actor::Entity::find()
			.distinct()
			.inner_join(model::object::Entity)
			.select_only()
			.select_column(model::actor::Column::Id)
			.filter(model::actor::Column::Domain.eq(ctx.domain()))
			.filter(model::object::Column::Published.gte(chrono::Utc::now() - std::time::Duration::from_secs(60 * 60 * 24 * 30)))
			.count(ctx.db())
			.await? as i64; // TODO safe cast
		TOTAL_ACTIVE_USERS_MONTH.store(actual_total_active_users_month, std::sync::atomic::Ordering::Relaxed);
		total_active_users_month = actual_total_active_users_month;
	}

	let mut total_active_users_halfyear = TOTAL_ACTIVE_USERS_HALFYEAR.load(std::sync::atomic::Ordering::Relaxed);
	if total_active_users_halfyear == i64::MIN {
		let actual_total_active_users_halfyear = model::actor::Entity::find()
			.distinct()
			.inner_join(model::object::Entity)
			.select_only()
			.select_column(model::actor::Column::Id)
			.filter(model::actor::Column::Domain.eq(ctx.domain()))
			.filter(model::object::Column::Published.gte(chrono::Utc::now() - std::time::Duration::from_secs(60 * 60 * 24 * 30 * 6)))
			.count(ctx.db())
			.await? as i64; // TODO safe cast
		TOTAL_ACTIVE_USERS_HALFYEAR.store(actual_total_active_users_halfyear, std::sync::atomic::Ordering::Relaxed);
		total_active_users_halfyear = actual_total_active_users_halfyear;
	}

	let (software, version) = match version.as_str() {
		"2.0.json" | "2.0" => (
			nodeinfo_upub::types::Software {
				name: "μpub".to_string(),
				version: Some(upub::VERSION.into()),
				repository: None,
				homepage: None,
			},
			"2.0".to_string()
		),
		"2.1.json" | "2.1" => (
			nodeinfo_upub::types::Software {
				name: "μpub".to_string(),
				version: Some(upub::VERSION.into()),
				repository: Some("https://github.com/alemidev/upub".into()),
				homepage: None,
			},
			"2.1".to_string()
		),
		_ => return Err(crate::ApiError::Status(StatusCode::NOT_IMPLEMENTED)),
	};
	Ok(Json(
		nodeinfo_upub::NodeInfoOwned {
			version,
			software,
			open_registrations: ctx.cfg().security.allow_registration,
			protocols: vec!["activitypub".into()],
			services: nodeinfo_upub::types::Services {
				inbound: vec![],
				outbound: vec![],
			},
			usage: nodeinfo_upub::types::Usage {
				local_posts: Some(total_posts),
				local_comments: Some(total_comments),
				users: Some(nodeinfo_upub::types::Users {
					active_month: Some(total_active_users_month),
					active_halfyear: Some(total_active_users_halfyear),
					total: Some(total_users),
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
				None => return Err(crate::ApiError::not_found()),
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
				link_type: Some(apb::jsonld::CONTENT_TYPE_LD_JSON_ACTIVITYPUB.to_string()),
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
