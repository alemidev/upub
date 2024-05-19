/// ActivityPub object node, representing either nothing, something, a link to something or
/// multiple things
pub enum Node<T : super::Base> {
	Array(std::collections::VecDeque<Node<T>>), // TODO would be cool to make it Box<[T]> so that Node is just a ptr
	Object(Box<T>),
	Link(Box<dyn crate::Link + Sync + Send>), // TODO feature flag to toggle these maybe?
	Empty,
}

// TODO convert in a from_residual (iirc?) so that in rust nightly we can do ?
impl<T : super::Base> From<Option<T>> for Node<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(x) => Node::Object(Box::new(x)),
			None => Node::Empty,
		}
	}
}

impl<T : super::Base> Iterator for Node<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match std::mem::replace(self, Self::Empty) {
			Self::Empty => None,
			Self::Object(res) => Some(*res),
			Self::Link(lnk) => {
				*self = Self::Link(lnk);
				None
			},
			Self::Array(mut arr) => {
				let mut out = None;
				while let Some(res) = arr.pop_front() {
					if let Some(inner) = res.extract() {
						out = Some(inner);
						break;
					}
				}
				*self = Self::Array(arr);
				out
			}
		}
	}
}

impl<T : super::Base> Node<T> {
	/// return reference to embedded object (or first if many are present)
	pub fn get(&self) -> Option<&T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(x),
			Node::Array(v) => v.iter().filter_map(|x| x.get()).next(),
		}
	}

	/// return reference to embedded object (or first if many are present)
	pub fn extract(self) -> Option<T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(*x),
			Node::Array(mut v) => v.pop_front()?.extract(),
		}
	}

	/// true only if Node is empty
	pub fn is_nothing(&self) -> bool {
		matches!(self, Node::Empty)
	}

	/// true only if Node is link
	pub fn is_link(&self) -> bool {
		matches!(self, Node::Link(_))
	}

	/// true only if Node contains one embedded object
	pub fn is_object(&self) -> bool {
		matches!(self, Node::Object(_))
	}

	/// true only if Node contains many embedded objects
	pub fn is_array(&self) -> bool {
		matches!(self, Node::Array(_))
	}

	/// true only if Node is empty
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}


	/// returns number of contained items (links count as items for len)
	pub fn len(&self) -> usize {
		match self {
			Node::Empty => 0,
			Node::Link(_) => 1,
			Node::Object(_) => 1,
			Node::Array(v) => v.len(),
		}
	}

	/// returns id of object: url for link, id for object, None if empty or array
	pub fn id(&self) -> Option<String> {
		match self {
			Node::Empty => None,
			Node::Link(uri) => Some(uri.href().to_string()),
			Node::Object(obj) => Some(obj.id()?.to_string()),
			Node::Array(arr) => Some(arr.front()?.id()?.to_string()),
		}
	}

	pub fn ids(&self) -> Vec<String> {
		match self {
			Node::Empty => vec![],
			Node::Link(uri) => vec![uri.href().to_string()],
			Node::Object(x) => x.id().map_or(vec![], |x| vec![x.to_string()]),
			Node::Array(x) => x.iter().filter_map(Self::id).collect()
		}
	}

	pub fn flat(self) -> Vec<Node<T>> {
		match self {
			Node::Empty => vec![],
			Node::Link(_) | Node::Object(_) => vec![self],
			// i think AP disallows array of arrays so no need to make this recursive
			Node::Array(arr) => arr.into()
		}
	}
}

#[cfg(feature = "unstructured")]
impl Node<serde_json::Value> {
	pub fn link(uri: String) -> Self {
		Node::Link(Box::new(uri))
	}

	pub fn links(uris: Vec<String>) -> Self {
		Node::Array(
			uris
				.into_iter()
				.map(Node::link)
				.collect()
		)
	}

	pub fn maybe_link(uri: Option<String>) -> Self {
		match uri {
			Some(uri) => Node::Link(Box::new(uri)),
			None => Node::Empty,
		}
	}

	pub fn object(x: serde_json::Value) -> Self {
		Node::Object(Box::new(x))
	}

	pub fn maybe_object(x: Option<serde_json::Value>) -> Self {
		match x {
			Some(x) => Node::Object(Box::new(x)),
			None => Node::Empty,
		}
	}

	pub fn array(values: Vec<serde_json::Value>) -> Self {
		Node::Array(
			std::collections::VecDeque::from_iter(
				values.into_iter()
					.map(Node::object)
			)
		)
	}

	#[cfg(feature = "fetch")]
	pub async fn fetch(&mut self) -> reqwest::Result<&mut Self> {
		if let Node::Link(link) = self {
			*self = reqwest::Client::new()
				.get(link.href())
				.header("Accept", "application/json")
				.send()
				.await?
				.json::<serde_json::Value>()
				.await?
				.into();
		}
		Ok(self)
	}
}

#[cfg(feature = "unstructured")]
impl From<Option<&str>> for Node<serde_json::Value> {
	fn from(value: Option<&str>) -> Self {
		match value {
			Some(x) => Node::Link(Box::new(x.to_string())),
			None => Node::Empty,
		}
	}
}

#[cfg(feature = "unstructured")]
impl From<&str> for Node<serde_json::Value> {
	fn from(value: &str) -> Self {
		Node::Link(Box::new(value.to_string()))
	}
}

#[cfg(feature = "unstructured")]
impl From<serde_json::Value> for Node<serde_json::Value> {
	fn from(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::String(uri) => Node::Link(Box::new(uri)),
			serde_json::Value::Array(arr) => Node::Array(
				std::collections::VecDeque::from_iter(
					arr.into_iter()
						.map(Node::from)
				)
			),
			serde_json::Value::Object(_) => match value.get("href") {
				None => Node::Object(Box::new(value)),
				Some(_) => Node::Link(Box::new(value)),
			},
			_ => Node::Empty,
		}
	}
}

