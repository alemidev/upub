pub enum Node<T : super::Base> {
	Array(Vec<Node<T>>), // TODO would be cool to make it Box<[Node<T>]> so that Node is just a ptr
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

impl<T : super::Base> Node<T> {
	pub fn get(self) -> Option<T> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(*x),
			Node::Array(v) => v.into_iter().find_map(|x| match x {
				Node::Object(x) => Some(*x),
				_ => None,
			}),
		}
	}

	// TODO extremely unforgiving, is this even useful?
	pub fn get_items(&self) -> Option<Vec<&T>> {
		match self {
			Node::Empty | Node::Link(_) => None,
			Node::Object(x) => Some(vec![x]),
			Node::Array(v) =>
				Some(v.iter().filter_map(|x| match x {
					Node::Object(x) => Some(&**x),
					_ => None,
				}).collect()),
		}
	}

	pub fn get_links(&self) -> Vec<String> {
		match self {
			Node::Empty => vec![],
			Node::Link(x) => vec![x.href().to_string()],
			Node::Object(x) => match x.id() {
				Some(x) => vec![x.to_string()],
				None => vec![],
			},
			Node::Array(v) =>
				v.iter().filter_map(|x| match x {
					Node::Link(x) => Some(x.href().to_string()),
					Node::Object(x) => x.id().map(|x| x.to_string()),
					// TODO handle array of arrays maybe?
					_ => None,
				}).collect(),
		}
	}
	
	pub fn is_empty(&self) -> bool {
		match self {
			Node::Empty | Node::Link(_) => true,
			Node::Object(_) | Node::Array(_) => false,
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

	pub fn flat(self) -> Vec<serde_json::Value> {
		match self {
			Node::Empty => vec![],
			Node::Link(l) => vec![serde_json::Value::String(l.href().to_string())],
			Node::Object(x) => vec![x.underlying_json_object()],
			Node::Array(arr) => {
				arr
					.into_iter()
					.filter_map(|node| match node {
						Node::Empty => None,
						Node::Link(l) => Some(serde_json::Value::String(l.href().to_string())),
						Node::Object(o) => Some(o.underlying_json_object()),
						Node::Array(_) => Some(serde_json::Value::Array(node.flat())),
					}).collect()
			}
		}
	}

	pub fn id(&self) -> Option<String> {
		match self {
			Node::Empty => None,
			Node::Link(uri) => Some(uri.href().to_string()),
			Node::Object(obj) => obj.id().map(|x| x.to_string()),
			Node::Array(arr) => arr.first()?.id().map(|x| x.to_string()),
		}
	}
}

impl Node<serde_json::Value>{
	pub fn empty() -> Self {
		Node::Empty
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

	pub fn object(x: impl super::Base) -> Self {
		Node::Object(Box::new(x.underlying_json_object()))
	}

	pub fn maybe_object(x: Option<impl super::Base>) -> Self {
		match x {
			Some(x) => Node::Object(Box::new(x.underlying_json_object())),
			None => Node::Empty,
		}
	}

	pub fn array(x: Vec<impl super::Base>) -> Self {
		Node::Array(x.into_iter().map(|x| Node::object(x.underlying_json_object())).collect())
	}

	pub async fn fetch(&mut self) -> reqwest::Result<()> {
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
		Ok(())
	}
}

impl From<Option<&str>> for Node<serde_json::Value> {
	fn from(value: Option<&str>) -> Self {
		match value {
			Some(x) => Node::Link(Box::new(x.to_string())),
			None => Node::Empty,
		}
	}
}

impl From<&str> for Node<serde_json::Value> {
	fn from(value: &str) -> Self {
		Node::Link(Box::new(value.to_string()))
	}
}

impl From<serde_json::Value> for Node<serde_json::Value> {
	fn from(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::String(uri) => Node::Link(Box::new(uri)),
			serde_json::Value::Object(_) => match value.get("href") {
				None => Node::Object(Box::new(value)),
				Some(_) => Node::Link(Box::new(value)),
			},
			serde_json::Value::Array(arr) => Node::Array(
				arr
					.into_iter()
					.map(Self::from)
					.collect()
			),
			_ => Node::Empty,
		}
	}
}

