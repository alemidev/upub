use html5ever::{tendril::SliceExt, tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer}};

const OPTIONS: comrak::Options<'static> = comrak::Options {
	extension: comrak::options::Extension {
		strikethrough: true,
		tagfilter: true,
		table: true,
		autolink: true,
		tasklist: false,
		superscript: true,
		header_ids: None,
		footnotes: false,
		description_lists: false,
		front_matter_delimiter: None,
		multiline_block_quotes: true,
		math_dollars: true,
		math_code: true,
		wikilinks_title_after_pipe: false,
		wikilinks_title_before_pipe: false,
		underline: true,
		subscript: true,
		spoiler: true,
		greentext: true,
		alerts: true,
		// TODO use these two for cloaking?
		image_url_rewriter: None,
		link_url_rewriter: None,
		cjk_friendly_emphasis: true,
    inline_footnotes: false,
    shortcodes: false,
    subtext: true,
    highlight: true,
    phoenix_heex: false,
	},

	parse: comrak::options::Parse {
		smart: false,
		default_info_string: None,
		relaxed_tasklist_matching: true,
		relaxed_autolinks: false,
		broken_link_callback: None,
    tasklist_in_table: false,
    ignore_setext: true,
    leave_footnote_definitions: true,
    escaped_char_spans: true,
	},

	render: comrak::options::Render {
		hardbreaks: true,
		github_pre_lang: true,
		full_info_string: false,
		width: 120,
		escape: true,
		list_style: comrak::options::ListStyleType::Dash,
		sourcepos: false,
		escaped_char_spans: true,
		experimental_minimize_commonmark: false,
		ignore_empty_links: false,
		gfm_quirks: false,
		prefer_fenced: true,
		figure_with_caption: false,
		tasklist_classes: false,
		ol_width: 3,
    r#unsafe: false,
	},
};

pub type Cloaker = Box<dyn Fn(&str) -> String>;

#[derive(Default)]
pub struct Sanitizer {
	pub cloaker: Option<Cloaker>,
	pub buffer: String,
}

pub fn safe_html(text: String) -> String {
	Sanitizer::default().html(text)
}

pub fn safe_markdown(text: String) -> String {
	Sanitizer::default().markdown(text)
}

pub fn strip_html(text: String) -> String {
	Stripper::default().strip_html(text)
}

impl Sanitizer {
	pub fn new(cloak: Cloaker) -> Self {
		Self {
			buffer: String::default(),
			cloaker: Some(cloak),
		}
	}

	pub fn markdown(self, text: String) -> String {
		self.html(comrak::markdown_to_html(&text, &OPTIONS))
	}
	
	pub fn html(self, text: String) -> String {
		let mut input = BufferQueue::default();
		let tendril = text.to_tendril();
		input.push_back(tendril);
	
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
					| "b" | "i" | "s"                // allow simple formatting: bold, italic and strikethrough, but not underlined as it can look like a link!
					| "strong" | "em" | "del"        // alternative ways to do bold, italig and strikethrough
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

#[derive(Default)]
pub struct Stripper(pub String);

impl Stripper {
	pub fn strip_html(self, text: String) -> String {
		let mut input = BufferQueue::default();
		let tendril = text.to_tendril();
		input.push_back(tendril);
	
		let mut tok = Tokenizer::new(self, Default::default());
		let _ = tok.feed(&mut input);
	
		if !input.is_empty() {
			tracing::warn!("buffer input not empty after processing html");
		}
		tok.end();
	
		tok.sink.0
	}
}

impl TokenSink for Stripper {
	type Handle = ();

	/// Each processed token will be handled by this method
	fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
		match token {
			Token::TagToken(_) => {},
			Token::CharacterTokens(txt) => self.0.push_str(txt.as_ref()),
			Token::CommentToken(_) => {},
			Token::DoctypeToken(_) => {},
			Token::NullCharacterToken => {},
			Token::EOFToken => {},
			Token::ParseError(e) => tracing::error!("error parsing html: {e}"),
		}
		TokenSinkResult::Continue
	}
}
