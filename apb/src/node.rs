/// ActivityPub object node, representing either nothing, something, a link to something or
/// multiple things
pub enum Node<T : super::Base> {
	/// this document node holds multiple objects
	Array(std::collections::VecDeque<Node<T>>), // TODO would be cool to make it Box<[T]> so that Node is just a ptr
	/// this document node holds one object
	Object(Box<T>),
	/// this document node holds a reference to an object
	Link(Box<dyn crate::Link + Sync + Send>), // TODO feature flag to toggle these maybe?
	/// this document node is not present
	Empty,
}

impl<T: super::Base + std::fmt::Debug> std::fmt::Debug for Node<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if f.alternate() {
			match self {
				Self::Array(x) => write!(f, "Node(array: {x:#?})"),
				Self::Link(x) => write!(f, "Node(link: {})", x.href().unwrap_or_default()),
				Self::Object(x) => write!(f, "Node(object: {x:#?})"),
				Self::Empty => write!(f, "Node(empty)"),
			}
		} else {
			match self {
				Self::Array(x) => write!(f, "Node(array: {x:?})"),
				Self::Link(x) => write!(f, "Node(link: {})", x.href().unwrap_or_default()),
				Self::Object(x) => write!(f, "Node(object: {x:?})"),
				Self::Empty => write!(f, "Node(empty)"),
			}
		}
	}
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

impl<T : super::Base> Node<T> {
	/// return reference to embedded object (or first if many are present)
	#[deprecated = "use .inner() instead"]
	pub fn get(&self) -> Option<&T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(x),
			Node::Array(v) => v.iter().filter_map(|x| x.get()).next(),
		}
	}

	/// return embedded object (or first if many are present)
	#[deprecated = "use .into_inner() instead"]
	pub fn extract(self) -> Option<T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(*x),
			Node::Array(mut v) => v.pop_front()?.extract(),
		}
	}

	/// return reference to embedded object (or first if many are present)
	pub fn inner(&self) -> crate::Field<&T> {
		match self {
			Node::Empty => Err(crate::FieldErr("node is empty")),
			Node::Link(_) => Err(crate::FieldErr("node has not been dereferenced")),
			Node::Object(x) => Ok(x),
			Node::Array(v) => v.iter().next().ok_or(crate::FieldErr("node contains no items"))?.inner(),
		}
	}

	/// return embedded object (or first if many are present)
	pub fn into_inner(self) -> crate::Field<T> {
		match self {
			Node::Empty => Err(crate::FieldErr("node is empty")),
			Node::Link(_) => Err(crate::FieldErr("node has not been dereferenced")),
			Node::Object(x) => Ok(*x),
			Node::Array(v) => v.into_iter().next().ok_or(crate::FieldErr("node contains no items"))?.into_inner(),
		}
	}

	/// true only if Node is empty
	pub fn is_empty(&self) -> bool {
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
	pub fn id(&self) -> crate::Field<String> {
		match self {
			Node::Empty => Err(crate::FieldErr("id")),
			Node::Link(uri) => uri.href(),
			Node::Object(obj) => obj.id(),
			Node::Array(arr) => arr.front().map(|x| x.id()).ok_or(crate::FieldErr("id"))?,
		}
	}

	pub fn all_ids(&self) -> Vec<String> {
		match self {
			Node::Empty => vec![],
			Node::Link(uri) => uri.href().map(|x| vec![x.to_string()]).unwrap_or_default(),
			Node::Object(x) => x.id().map_or(vec![], |x| vec![x.to_string()]),
			Node::Array(x) => x.iter().filter_map(|x| Some(x.id().ok()?.to_string())).collect()
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
	pub fn into_value(self) -> serde_json::Value {
		match self {
			Self::Object(x) => *x,
			Self::Link(l) => serde_json::Value::String(l.href().unwrap_or_default().to_string()),
			Self::Empty => serde_json::Value::Null,
			Self::Array(arr) => serde_json::Value::Array(
				arr
					.into_iter()
					.map(|x| x.into_value())
					.collect()
			),
		}
	}

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

	pub fn maybe_array(values: Vec<serde_json::Value>) -> Self {
		if values.is_empty() {
			Node::Empty
		} else {
			Node::array(values)
		}
	}

	#[cfg(feature = "fetch")]
	pub async fn fetch(&mut self) -> reqwest::Result<&mut Self> {
		if let Node::Link(link) = self {
			if let Ok(url) = link.href() {
				*self = reqwest::Client::new()
					.get(url)
					.header("Accept", "application/json")
					.send()
					.await?
					.json::<serde_json::Value>()
					.await?
					.into();
			}
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
		use crate::Link;
		match value {
			serde_json::Value::String(uri) => Node::Link(Box::new(uri)),
			serde_json::Value::Array(arr) => Node::Array(
				std::collections::VecDeque::from_iter(
					arr.into_iter()
						.map(Node::from)
				)
			),
			serde_json::Value::Object(_) => match value.link_type() {
				Ok(_) => Node::Link(Box::new(value)),
				Err(_) => Node::Object(Box::new(value)),
			},
			_ => Node::Empty,
		}
	}
}
