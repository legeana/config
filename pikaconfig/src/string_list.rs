use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum StringList {
    Single(String),
    List(Vec<String>),
}

impl StringList {
    pub fn as_slice(&self) -> &[String] {
        match self {
            Self::Single(e) => std::slice::from_ref(e),
            Self::List(v) => v,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.as_slice().iter()
    }
    pub fn to_vec(&self) -> Vec<String> {
        self.as_slice().to_vec()
    }
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
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
        assert_eq!(s.as_slice(), &["test"]);
    }

    #[test]
    fn test_list_as_vec() {
        let s = StringList::List(vec!["hello".to_owned(), "world".to_owned()]);
        assert_eq!(s.as_slice(), &["hello", "world"]);
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
