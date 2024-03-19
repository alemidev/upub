#[derive(Debug, thiserror::Error)]
pub enum NodeResolutionError {
	#[error("error fetching object: {0}")]
	FetchError(#[from] reqwest::Error),

	#[error("empty array")]
	EmptyArray,

	#[error("field not present")]
	Empty,
}

pub enum Node<T> {
	Array(Vec<T>),
	Object(T),
	Link(Box<dyn super::Link>),
	Empty,
}

impl<T> From<Option<T>> for Node<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(x) => Node::Object(x),
			None => Node::Empty,
		}
	}
}

impl<T> Node<T> {
	pub fn first(&self) -> Option<&T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(x),
			Node::Array(v) => v.first(),
		}
	}

	pub fn items(&self) -> Option<Vec<&T>> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(vec![x]),
			Node::Array(v) =>
				Some(v.iter().map(|x| &x).collect()),
		}
	}
}

impl<T> Node<T>
where
	T : super::Base
{
	pub fn id(&self) -> Option<&str> {
		match self {
			Node::Array(v) => v.first()?.id(),
			Node::Link(x) => Some(x.href()),
			Node::Object(x) => x.id(),
			Node::Empty => None,
		}
	}
}

impl<T> Node<T>
where
	T : Clone + for<'de> serde::Deserialize<'de>,
{
	pub async fn resolve(self) -> Result<T, NodeResolutionError> {
		match self {
			Node::Empty => Err(NodeResolutionError::Empty),
			Node::Object(object) => Ok(object),
			Node::Array(array) => Ok(
				array
					.first()
					.ok_or(NodeResolutionError::EmptyArray)?
					.clone()
			),
			Node::Link(link) => Ok(
				reqwest::get(link.href())
					.await?
					.json::<T>()
					.await?
			),
		}
	}
}

pub trait NodeExtractor {
	fn node(&self, id: &str) -> Node<serde_json::Value>;
	fn node_vec(&self, id: &str) -> Node<serde_json::Value>;
}

impl NodeExtractor for serde_json::Value {
	fn node(&self, id: &str) -> Node<serde_json::Value> {
		match self.get(id) {
			None => Node::Empty,
			Some(x) => match Node::new(x.clone()) {
				Err(e) => Node::Empty,
				Ok(x) => x,
			}
		}
	}

	fn node_vec(&self, id: &str) -> Node<serde_json::Value> {
		match self.get(id) {
			None => Node::Empty,
			Some(x) => match Node::many(x.clone()) {
				Err(e) => Node::Empty,
				Ok(x) => x,
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("json object is wrongly structured")]
pub struct JsonStructureError;

impl Node<serde_json::Value> {
	pub fn new(value: serde_json::Value) -> Result<Self, JsonStructureError> {
		if !(value.is_string() || value.is_object()) {
			return Err(JsonStructureError);
		}
		if value.is_string() || value.get("href").is_some() {
			Ok(Self::Link(Box::new(value)))
		} else {
			Ok(Self::Object(value))
		}
	}

	pub fn many(value: serde_json::Value) -> Result<Vec<Self>, JsonStructureError> {
		if let serde_json::Value::Array(arr) = value {
			Ok(
				arr
					.into_iter()
					.filter_map(|x| Self::new(x.clone()).ok())
					.collect()
			)
		} else {
			Ok(vec![Self::new(value)?])
		}
	}
}


pub(crate) trait InsertStr {
	fn insert_str(&mut self, k: &str, v: Option<&str>);
	fn insert_timestr(&mut self, k: &str, t: Option<chrono::DateTime<chrono::Utc>>);
}
impl InsertStr for serde_json::Map<String, serde_json::Value> {
	fn insert_str(&mut self, k: &str, v: Option<&str>) {
		if let Some(v) = v {
			self.insert(
				k.to_string(),
				serde_json::Value::String(v.to_string()),
			);
		}
	}

	fn insert_timestr(&mut self, k: &str, t: Option<chrono::DateTime<chrono::Utc>>) {
		if let Some(published) = t {
			self.insert(
				"published".to_string(),
				serde_json::Value::String(published.to_rfc3339()),
			);
		}
	}
}
