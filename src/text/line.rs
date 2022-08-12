/// Represent a line of text that whitespace in text of inline elements are
/// processed according to the document: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model/Whitespace
#[derive(Debug, Clone)]
pub(super) struct Line<'s> {
    pub tokens: Vec<&'s str>,
    len_before_last_sentence: usize,
    len_last_sentence: usize,
    has_trailing_space: bool,
}

impl<'s> Line<'s> {
    pub fn with_capacity(size: usize) -> Self {
        Line {
            tokens: Vec::with_capacity(size),
            len_before_last_sentence: 0,
            len_last_sentence: 0,
            has_trailing_space: false,
        }
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.len_before_last_sentence = 0;
        self.len_last_sentence = 0;
        self.has_trailing_space = false;
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn len(&self) -> usize {
        self.len_before_last_sentence + self.len_last_sentence
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        self.tokens.join("")
    }

    /// Append a sentence to the line following the HTML whitespace rules.
    ///
    /// 1. Always remove leading spaces
    /// 2. Consecutive spaces are replaced by a single space
    /// 3. A trailing space is delayed and only applied when a new non-empty sentence is added.
    /// 4. If there is no trailing space, and the new sentence has leading spaces (before removing), add a beginning space.
    pub fn append(&mut self, sentence: &'s str) {
        // remove leading spaces
        let trimed_start_sentence = sentence.trim_start();

        // don't add an empty string
        if trimed_start_sentence.len() == 0 {
            if self.tokens.len() > 0 {
                self.has_trailing_space = true;
            }
            return;
        }

        self.len_before_last_sentence += self.len_last_sentence;

        // apply a trailing space from the previous sentence, or add a space if the beginning of the new sentence has leading spaces
        if self.has_trailing_space
            || (self.tokens.len() > 0 && sentence.starts_with(char::is_whitespace))
        {
            self.tokens.push(" ");
            self.len_before_last_sentence += 1;
        }

        self.len_last_sentence = 0;
        // split sentence by space to merge consecutive spaces
        for token in trimed_start_sentence.split(char::is_whitespace) {
            if token.len() == 0 {
                continue;
            }
            self.tokens.push(token);
            self.tokens.push(" ");
            self.len_last_sentence += token.len() + 1;
        }

        // remove trailing space
        self.tokens.pop();
        self.len_last_sentence -= 1;
        self.has_trailing_space = sentence.ends_with(char::is_whitespace);
    }
}

#[derive(Debug, Clone)]
pub(super) struct Paragraph<'s> {
    pub tokens: Vec<&'s str>,
    len: usize,
}

impl<'s> Paragraph<'s> {
    pub fn with_capacity(size: usize) -> Self {
        Paragraph {
            tokens: Vec::with_capacity(size),
            len: 0,
        }
    }

    pub fn append(&mut self, line: &Line<'s>) {
        if line.is_empty() {
            return;
        }
        if self.tokens.len() > 0 {
            self.tokens.push("\n");
            self.len += 1;
        }
        self.tokens.extend(line.tokens.iter());
        self.len += line.len();
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn to_string(&self) -> String {
        self.tokens.join("")
    }
}
