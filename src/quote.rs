const ESCAPE: char = '\\';

struct Escaper {
    escape: char,
    special: Vec<char>,
}

impl Escaper {
    fn escape(&self, iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
        let special = self.special.clone();
        let escape = self.escape;
        iter.flat_map(move |c| {
            if special.contains(&c) {
                [Some(escape), Some(c)]
            } else {
                [Some(c), None]
            }
        })
        .flatten()
    }
    fn enquote(&self, quote: char, iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
        std::iter::once(quote)
            .chain(self.escape(iter))
            .chain(std::iter::once(quote))
    }
}

pub fn enquote_with(quote: char, text: &str) -> String {
    let esc = Escaper {
        escape: ESCAPE,
        special: vec![ESCAPE, quote],
    };
    esc.enquote(quote, text.chars()).collect()
}

#[allow(dead_code)]
pub fn enquote(text: &str) -> String {
    enquote_with('"', text)
}

#[allow(dead_code)]
pub fn unquote(_text: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enquote_with() {
        assert_eq!(enquote_with('\'', "hello world"), "'hello world'");
        assert_eq!(enquote_with('\'', "'hello'"), r#"'\'hello\''"#);
        assert_eq!(enquote_with('\'', r#"hello\ world"#), r#"'hello\\ world'"#);
    }

    #[test]
    fn test_enquote() {
        assert_eq!(enquote("hello"), r#""hello""#);
        assert_eq!(
            enquote(r#"test "hello" world"#),
            r#""test \"hello\" world""#
        );
        assert_eq!(enquote(r#"escaped\ symbol"#), r#""escaped\\ symbol""#);
    }
}
