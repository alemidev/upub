pub enum Node<T : super::Base> {
	Array(Vec<T>), // TODO would be cool to make it Box<[T]> so that Node is just a ptr
	Object(Box<T>),
	Link(Box<dyn super::Link>),
	Empty,
}

impl<T : super::Base> From<Option<T>> for Node<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(x) => Node::Object(Box::new(x)),
			None => Node::Empty,
		}
	}
}

impl<T : super::Base + Clone> Iterator for Node<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let x = match self {
			Self::Empty => return None,
			Self::Link(_) => return None,
			Self::Array(arr) => return arr.pop(), // TODO weird that we iter in reverse
			Self::Object(x) => *x.clone(), // TODO needed because next() on object can't get value without owning
		};
		*self = Self::Empty;
		Some(x)
	}
}

impl<T : super::Base> Node<T> {
	pub fn get(&self) -> Option<&T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(x),
			Node::Array(v) => v.last(), // TODO so it's coherent with next(), still weird tho!
		}
	}

	pub fn extract(self) -> Option<T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(*x),
			Node::Array(mut v) => v.pop(), // TODO so it's coherent with next(), still weird tho!
		}
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, Node::Empty)
	}

	pub fn is_link(&self) -> bool {
		matches!(self, Node::Link(_))
	}

	pub fn is_object(&self) -> bool {
		matches!(self, Node::Object(_))
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Node::Array(_))
	}

	pub fn len(&self) -> usize {
		match self {
			Node::Empty => 0,
			Node::Link(_) => 1,
			Node::Object(_) => 1,
			Node::Array(v) => v.len(),
		}
	}

	pub fn id(&self) -> Option<String> {
		match self {
			Node::Empty | Node::Array(_) => None,
			Node::Link(uri) => Some(uri.href().to_string()),
			Node::Object(obj) => obj.id().map(|x| x.to_string()),
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
				.map(serde_json::Value::String)
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
		Node::Array(values)
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
			serde_json::Value::Array(arr) => Node::Array(arr),
			serde_json::Value::Object(_) => match value.get("href") {
				None => Node::Object(Box::new(value)),
				Some(_) => Node::Link(Box::new(value)),
			},
			_ => Node::Empty,
		}
	}
}

