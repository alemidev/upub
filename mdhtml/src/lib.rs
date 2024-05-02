use html5ever::tendril::*;
use html5ever::tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer};
use comrak::{markdown_to_html, Options};

/// In our case, our sink only contains a tokens vector
#[derive(Debug, Clone, Default)]
struct Sink(String);

impl TokenSink for Sink {
	type Handle = ();

	/// Each processed token will be handled by this method
	fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
		match token {
			Token::TagToken(tag) => {
				if !matches!(
					tag.name.as_ref(),
					"h1" | "h2" | "h3" 
					| "hr" | "br" | "p" | "b" | "i"
					| "blockquote" | "pre" | "code"
					| "ul" | "ol" | "li"
					| "img" | "a"
				) { return TokenSinkResult::Continue } // skip this tag

				self.0.push('<');
				if !tag.self_closing && matches!(tag.kind, TagKind::EndTag) {
					self.0.push('/');
				}

				self.0.push_str(tag.name.as_ref());

				match tag.name.as_ref() {
					"img" => for attr in tag.attrs {
						match attr.name.local.as_ref() {
							"src" => self.0.push_str(&format!(" src=\"{}\"", attr.value.as_ref())),
							"title" => self.0.push_str(&format!(" title=\"{}\"", attr.value.as_ref())),
							"alt" => self.0.push_str(&format!(" alt=\"{}\"", attr.value.as_ref())),
							_ => {},
						}
					},
					"a" => {
						for attr in tag.attrs {
							match attr.name.local.as_ref() {
								"href" => self.0.push_str(&format!(" href=\"{}\"", attr.value.as_ref())),
								"title" => self.0.push_str(&format!(" title=\"{}\"", attr.value.as_ref())),
								_ => {},
							}
						}
						self.0.push_str(" rel=\"nofollow noreferrer\" target=\"_blank\"");
					},
					_ => {},
				}

				if tag.self_closing {
					self.0.push('/');
				}
				self.0.push('>');
			},
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

pub fn safe_markdown(text: &str) -> String {
	safe_html(&markdown_to_html(text, &Options::default()))
}

pub fn safe_html(text: &str) -> String {
	let mut input = BufferQueue::default();
	input.push_back(text.to_tendril().try_reinterpret().unwrap());

	let sink = Sink::default();

	let mut tok = Tokenizer::new(sink, Default::default());
	let _ = tok.feed(&mut input);

	if !input.is_empty() {
		tracing::warn!("buffer input not empty after processing html");
	}
	tok.end();

	tok.sink.0
}
