// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

const COMMENT: char = '#';
const SCISSORS: &str = "# ------------------------ >8 ------------------------";
const TITLE_END: &str = " (";

// A commit message given the library's scores: a paragraph of its own above
// the trailers, or last when there are none, in place of the one an earlier
// scoring left -- an amend scores again. Git's comments, and everything from
// the scissors line `--verbose` writes, stay below it, as git hands them over.
// A message with no words is left as it is, so git still aborts its commit.
pub struct MessageScoring {
    block: String,
}

// A message cut where git's own lines begin: its text, and the comments or the
// scissors' cut below it.
struct MessageParts<'a> {
    pub text: &'a str,
    pub tail: &'a str,
}

impl MessageScoring {
    pub fn new(block: String) -> Self {
        Self { block }
    }

    pub fn applied_to(&self, message: &str) -> String {
        let MessageParts { text, tail } = Self::split_at_the_comments(message);
        let mut paragraphs: Vec<String> = Self::paragraphs(text)
            .into_iter()
            .filter(|paragraph| !self.is_scores(paragraph))
            .collect();
        if paragraphs.is_empty() {
            return String::from(message);
        }
        let at = match paragraphs.last() {
            Some(last) if paragraphs.len() > 1 && Self::is_trailers(last) => paragraphs.len() - 1,
            _ => paragraphs.len(),
        };
        paragraphs.insert(at, String::from(self.block.trim_end()));
        let mut scored = paragraphs.join("\n\n");
        scored.push('\n');
        if !tail.is_empty() {
            let gap = text.len() - text.trim_end_matches('\n').len();
            scored.push_str(&"\n".repeat(gap.saturating_sub(1)));
            scored.push_str(tail);
        }
        scored
    }

    // Where the comment lines that end the message begin; its end if none do.
    fn comments_start(message: &str) -> usize {
        let mut at = message.len();
        for line in message.split_inclusive('\n').rev() {
            if !line.starts_with(COMMENT) {
                break;
            }
            at -= line.len();
        }
        at
    }

    fn is_scores(&self, paragraph: &str) -> bool {
        let title = self.block.split(TITLE_END).next().unwrap_or_default();
        paragraph.starts_with(title)
    }

    // A last paragraph every line of which is a key and a value -- the
    // trailers git and its tools read.
    fn is_trailers(paragraph: &str) -> bool {
        paragraph.lines().all(|line| {
            line.split_once(": ").is_some_and(|(key, _)| {
                !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
        })
    }

    fn paragraphs(text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|chunk| chunk.trim_matches('\n'))
            .filter(|chunk| !chunk.trim().is_empty())
            .map(String::from)
            .collect()
    }

    // The message's text, and what follows it: from the scissors line if there
    // is one, else the comment lines that end it.
    fn split_at_the_comments(message: &str) -> MessageParts<'_> {
        let at = message
            .find(SCISSORS)
            .unwrap_or_else(|| Self::comments_start(message));
        let (text, tail) = message.split_at(at);
        MessageParts { text, tail }
    }
}
