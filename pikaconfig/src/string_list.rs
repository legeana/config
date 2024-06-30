#![allow(dead_code)]

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum StringList {
    Single(String),
    List(Vec<String>),
}

impl StringList {
    pub fn iter(&self) -> StringListIterator<'_> {
        match self {
            Self::Single(e) => StringListIterator::Single(std::iter::once(e)),
            Self::List(v) => StringListIterator::List(v.iter()),
        }
    }
    pub fn as_vec(&self) -> Vec<&String> {
        self.iter().collect()
    }
    pub fn to_vec(&self) -> Vec<String> {
        self.iter().cloned().collect()
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::List(v) => v.is_empty(),
        }
    }
}

pub enum StringListIterator<'a> {
    Single(std::iter::Once<&'a String>),
    List(std::slice::Iter<'a, String>),
}

impl<'a> Iterator for StringListIterator<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(i) => i.next(),
            Self::List(i) => i.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Single(i) => i.size_hint(),
            Self::List(i) => i.size_hint(),
        }
    }
}

impl Default for StringList {
    fn default() -> Self {
        StringList::List(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct StringListExample {
        pub field: StringList,
    }

    #[test]
    fn test_load_missing() {
        let r: Result<StringListExample, toml::de::Error> = toml::from_str("");
        let e = r.expect_err("toml::from_str");
        assert_eq!(e.message(), "missing field `field`");
    }

    #[test]
    fn test_load_single() {
        let s: StringListExample = toml::from_str(
            "
            field = 'hello'
            ",
        )
        .expect("toml::from_str");
        assert_eq!(
            s,
            StringListExample {
                field: StringList::Single("hello".to_owned()),
            }
        );
    }

    #[test]
    fn test_load_list() {
        let s: StringListExample = toml::from_str(
            "
            field = ['hello']
            ",
        )
        .expect("toml::from_str");
        assert_eq!(
            s,
            StringListExample {
                field: StringList::List(vec!["hello".to_owned()]),
            }
        );
    }

    #[test]
    fn test_load_list_many() {
        let s: StringListExample = toml::from_str(
            "
            field = ['hello', 'world']
            ",
        )
        .expect("toml::from_str");
        assert_eq!(
            s,
            StringListExample {
                field: StringList::List(vec!["hello".to_owned(), "world".to_owned()]),
            }
        );
    }

    #[test]
    fn test_single_iter() {
        let s = StringList::Single("test".to_owned());

        let v: Vec<_> = s.iter().collect();
        assert_eq!(v, vec!["test"]);
    }

    #[test]
    fn test_list_iter() {
        let s = StringList::List(vec!["hello".to_owned(), "world".to_owned()]);

        let v: Vec<_> = s.iter().collect();
        assert_eq!(v, vec!["hello", "world"]);
    }

    #[test]
    fn test_single_as_vec() {
        let s = StringList::Single("test".to_owned());
        assert_eq!(s.as_vec(), vec!["test"]);
    }

    #[test]
    fn test_list_as_vec() {
        let s = StringList::List(vec!["hello".to_owned(), "world".to_owned()]);
        assert_eq!(s.as_vec(), vec!["hello", "world"]);
    }

    #[test]
    fn test_single_to_vec() {
        let s = StringList::Single("test".to_owned());
        assert_eq!(s.to_vec(), vec!["test".to_owned()]);
    }

    #[test]
    fn test_list_to_vec() {
        let s = StringList::List(vec!["hello".to_owned(), "world".to_owned()]);
        assert_eq!(s.to_vec(), vec!["hello".to_owned(), "world".to_owned()]);
    }

    #[test]
    fn test_single_is_not_empty() {
        let s = StringList::Single("test".to_owned());
        assert!(!s.is_empty());
    }

    #[test]
    fn test_default_is_empty() {
        assert!(StringList::default().is_empty());
    }

    #[test]
    fn test_list_is_empty() {
        assert!(StringList::List(Vec::new()).is_empty());
    }

    #[test]
    fn test_list_is_not_empty() {
        let s = StringList::List(vec!["hello".to_owned(), "world".to_owned()]);
        assert!(!s.is_empty());
    }

    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct StringListDefaultExample {
        #[serde(default)]
        pub field: StringList,
    }

    #[test]
    fn test_default() {
        let s: StringListDefaultExample = toml::from_str("").expect("toml::from_str");
        assert_eq!(s, StringListDefaultExample::default());
    }
}
