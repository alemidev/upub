use html5ever::{tendril::SliceExt, tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer}};

// TODO this drives me so mad!!! The #[non_exhaustive] attr on the underlying ___Options structs
//      makes it impossible to construct them with struct syntax! I need to use this BULLSHIT
//      builder pattern. And it's NOT CONST!!! I have to keep this shit heap allocated somewhere
//      because this thing is #[non_exhaustive]... oh i hate this so much
fn options() -> &'static comrak::Options {
	static OPTIONS: std::sync::OnceLock<comrak::Options> = std::sync::OnceLock::new();
	OPTIONS.get_or_init(||
		comrak::Options {
			extension: comrak::ExtensionOptionsBuilder::default()
				.autolink(true)
				.strikethrough(true)
				.description_lists(false)
				.tagfilter(true)
				.table(true)
				.tasklist(false)
				.superscript(true)
				.header_ids(None)
				.footnotes(false)
				.front_matter_delimiter(None)
				.multiline_block_quotes(true)
				.math_dollars(true)
				.math_code(true)
				.build()
				.expect("error creating default markdown extension options"),
		
			parse: comrak::ParseOptionsBuilder::default()
				.smart(false)
				.default_info_string(None)
				.relaxed_tasklist_matching(true)
				.relaxed_autolinks(false)
				.build()
				.expect("erropr creating default markdown parse options"),
		
			render: comrak::RenderOptionsBuilder::default()
				.hardbreaks(true)
				.github_pre_lang(true)
				.full_info_string(false)
				.width(120)
				.unsafe_(false)
				.escape(true)
				.list_style(comrak::ListStyleType::Dash)
				.sourcepos(false)
				.escaped_char_spans(true)
				.build()
				.expect("error creating default markdown render options"),
		}
	)
}

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
		self.html(&comrak::markdown_to_html(text, options()))
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
					"h1" | "h2" | "h3"               // allow titles, up to 3 depth
					| "sup" | "sub"                  // allow superscript/subscript
					| "hr" | "br"                    // allow horizontal rules and linebreaks
					| "p" | "span"                   // allow placing paragraphs and spans
					| "b" | "i" | "s" | "del"        // allow simple formatting: bold, italic and strikethrough, but not underlined as it can look like a link!
					| "blockquote" | "pre" | "code"  // allow code blocks
					| "ul" | "ol" | "li"             // allow lists
					| "img" | "a"                    // allow images and links, but will get sanitized later
				) {
					return TokenSinkResult::Continue; // skip this tag
				}

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
