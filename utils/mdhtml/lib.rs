use html5ever::{tendril::SliceExt, tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer}};
use comrak::{markdown_to_html, Options};

pub type Cloaker = Box<dyn Fn(&str) -> String>;

#[derive(Default)]
pub struct Sanitizer {
	pub cloaker: Option<Cloaker>,
	pub buffer: String,
}

pub fn safe_html(text: &str) -> String {
	Sanitizer::default().html(text)
}

pub fn safe_markdown(text: &str) -> String {
	Sanitizer::default().markdown(text)
}

impl Sanitizer {
	pub fn new(cloak: Cloaker) -> Self {
		Self {
			buffer: String::default(),
			cloaker: Some(cloak),
		}
	}

	pub fn markdown(self, text: &str) -> String {
		self.html(&markdown_to_html(text, &Options::default()))
	}
	
	pub fn html(self, text: &str) -> String {
		let mut input = BufferQueue::default();
		input.push_back(text.to_tendril().try_reinterpret().unwrap());
	
		let mut tok = Tokenizer::new(self, Default::default());
		let _ = tok.feed(&mut input);
	
		if !input.is_empty() {
			tracing::warn!("buffer input not empty after processing html");
		}
		tok.end();
	
		tok.sink.buffer
	}
}

impl TokenSink for Sanitizer {
	type Handle = ();

	/// Each processed token will be handled by this method
	fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
		match token {
			Token::TagToken(tag) => {
				if !matches!(
					tag.name.as_ref(),
					"h1" | "h2" | "h3" 
					| "hr" | "br" | "p" | "b" | "i" | "s"
					| "blockquote" | "pre" | "code"
					| "ul" | "ol" | "li"
					| "img" | "a"
				) { return TokenSinkResult::Continue } // skip this tag

				self.buffer.push('<');

				if !tag.self_closing && matches!(tag.kind, TagKind::EndTag) {
					self.buffer.push('/');
				}

				self.buffer.push_str(tag.name.as_ref());

				if !matches!(tag.kind, TagKind::EndTag) {
					match tag.name.as_ref() {
						"img" => for attr in tag.attrs {
							match attr.name.local.as_ref() {
								"src" => {
									let src = if let Some(ref cloak) = self.cloaker {
										cloak(attr.value.as_ref())
									} else {
										attr.value.to_string()
									};
									self.buffer.push_str(&format!(" src=\"{src}\""))
								},
								"title" => self.buffer.push_str(&format!(" title=\"{}\"", attr.value.as_ref())),
								"alt" => self.buffer.push_str(&format!(" alt=\"{}\"", attr.value.as_ref())),
								_ => {},
							}
						},
						"a" => {
							let any_attr = !tag.attrs.is_empty();
							for attr in tag.attrs {
								match attr.name.local.as_ref() {
									"href" => self.buffer.push_str(&format!(" href=\"{}\"", attr.value.as_ref())),
									"title" => self.buffer.push_str(&format!(" title=\"{}\"", attr.value.as_ref())),
									"class" => if attr.value.as_ref() == "u-url mention" {
										self.buffer.push_str(" class=\"u-url mention\"")
									},
									_ => {},
								}
							}
							if any_attr {
								self.buffer.push_str(" rel=\"nofollow noreferrer\" target=\"_blank\"");
							}
						},
						_ => {},
					}
				}

				if tag.self_closing {
					self.buffer.push('/');
				}

				self.buffer.push('>');
			},
			Token::CharacterTokens(txt) => self.buffer.push_str(txt.as_ref()),
			Token::CommentToken(_) => {},
			Token::DoctypeToken(_) => {},
			Token::NullCharacterToken => {},
			Token::EOFToken => {},
			Token::ParseError(e) => tracing::error!("error parsing html: {e}"),
		}
		TokenSinkResult::Continue
	}
}
