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
	Array(Vec<Node<T>>),
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
	pub fn get(&self) -> Option<&T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(x),
			Node::Array(v) => v.first(),
		}
	}

	pub fn all(&self) -> Option<Vec<&T>> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(vec![x]),
			Node::Array(v) =>
				Some(v.iter().map(|x| &x).collect()),
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Node::Empty => 0,
			Node::Link(_) => 0,
			Node::Object(_) => 1,
			Node::Array(v) => v.len(),
		}
	}
}

impl From<serde_json::Value> for Node<serde_json::Value> {
	fn from(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::String(uri) => Node::Link(Box::new(uri)),
			serde_json::Value::Object(_) => match value.get("href") {
				Some(_) => Node::Object(value),
				None => Node::Link(Box::new(value)),
			},
			serde_json::Value::Array(arr) => Node::Array(
				arr
					.into_iter()
					.filter_map(|x| Self::new(x).ok())
					.collect()
			),
			_ => Node::Empty,
		}
	}
}

impl Node<serde_json::Value>{
	pub async fn fetch(&mut self) -> reqwest::Result<()> {
		if let Node::Link(link) = self {
			*self = reqwest::get(link.href())
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}
		Ok(())
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
