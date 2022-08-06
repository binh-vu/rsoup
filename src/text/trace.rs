use hashbrown::HashMap;

use crate::helper::ITree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTrace {
    pub text: String,
    // html elements are stored in order
    pub trace: Vec<TextHTMLElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextHTMLElement {
    pub tag: String,
    pub start: usize,
    pub end: usize,
    pub attrs: HashMap<String, String>,
    pub children: Vec<TextHTMLElement>,
}

impl TextTrace {
    pub fn new() -> TextTrace {
        TextTrace {
            text: String::new(),
            trace: Vec::new(),
        }
    }

    pub fn merge(&mut self, text: TextTrace) {
        for mut el in text.trace {
            el.shift(self.text.len());
            self.trace.push(el);
        }
        self.text.push_str(&text.text);
    }

    pub fn preorder_traversal<'s>(&'s self) -> TreesPreorderTraversal<'s> {
        TreesPreorderTraversal::new(&self.trace)
    }

    pub fn to_bare_html(&self) -> String {
        let mut tokens = Vec::<&str>::with_capacity(2 + self.trace.len());
        let mut closing_tags = Vec::<&TextHTMLElement>::new();
        let mut pointer = 0;

        for token in self.preorder_traversal() {
            // println!(
            //     "------before\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tags
            // );

            while let Some(closing_tag) = closing_tags.last() {
                if closing_tag.end <= token.start {
                    // this tag is closed
                    if token.start == token.end {
                        // this token is empty, but is it part of this closing tag?
                        // if not, we can continue to closing tag
                        // if yes, we break here.
                        // and it only happens when its a direct children
                        if closing_tag
                            .children
                            .iter()
                            .any(|child| std::ptr::eq(child, token))
                        {
                            break;
                        }
                    }
                    tokens.push(&self.text[pointer..closing_tag.end]);
                    tokens.push("</");
                    tokens.push(&closing_tag.tag);
                    tokens.push(">");
                    pointer = token.end;
                    closing_tags.pop();
                } else {
                    break;
                }
            }

            tokens.push(&self.text[pointer..token.start]);
            tokens.push("<");
            tokens.push(&token.tag);
            tokens.push(">");

            pointer = token.start;
            closing_tags.push(token);

            // println!(
            //     "------after\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tags
            // );
        }

        for closing_tag in closing_tags.iter().rev() {
            tokens.push(&self.text[pointer..closing_tag.end]);
            tokens.push("</");
            tokens.push(&closing_tag.tag);
            tokens.push(">");
            pointer = closing_tag.end;
        }
        tokens.push(&self.text[pointer..]);
        tokens.join("")
    }
}

impl TextHTMLElement {
    pub fn shift(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
        for child in self.children.iter_mut() {
            child.shift(offset);
        }
    }
}

// impl ITree<&TextHTMLElement> for TextHTMLElement {
//     fn get_root(&self) -> &TextHTMLElement {
//         self
//     }

//     fn get_children(&self, node: &TextHTMLElement) -> &[&TextHTMLElement] {
//         return &node.children;
//     }
// }

pub struct TreesPreorderTraversal<'s> {
    trees: &'s [TextHTMLElement],
    tree_index: usize,
    stack: Vec<(&'s TextHTMLElement, usize)>,
}

impl<'s> TreesPreorderTraversal<'s> {
    pub fn new(trees: &'s [TextHTMLElement]) -> Self {
        Self {
            trees,
            tree_index: 0,
            stack: Vec::new(),
        }
    }
}

impl<'s> Iterator for TreesPreorderTraversal<'s> {
    type Item = &'s TextHTMLElement;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.len() == 0 {
                if self.tree_index == self.trees.len() {
                    // no more tree
                    return None;
                }
                // pop one tree and add it to the stack
                self.stack.push((&self.trees[self.tree_index], 0));
                self.tree_index += 1;
                // return this node
                return Some(&self.stack[self.stack.len() - 1].0);
            }

            // current element, has been returned previously,
            // so we will try to return its child
            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            if child_index < node.children.len() {
                // add this child to stack
                self.stack.push((&node.children[child_index], 0));
                // mark this child has been returned
                self.stack[n1].1 += 1;
                // return this child
                return Some(&node.children[child_index]);
            }

            // no child can be returned, we done at this level, move up
            self.stack.pop();
        }
    }
}
