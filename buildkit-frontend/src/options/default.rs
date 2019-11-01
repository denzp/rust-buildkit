use std::collections::BTreeMap;
use std::iter::once;

use either::Either;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct Options {
    inner: BTreeMap<String, OptionValue>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum OptionValue {
    Flag(bool),
    Single(String),
    Multiple(Vec<String>),
}

impl Options {
    pub fn has<S>(&self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        match self.inner.get(name.as_ref()) {
            Some(container) => match container {
                OptionValue::Flag(exists) => *exists,
                OptionValue::Single(_) => true,
                OptionValue::Multiple(_) => true,
            },

            None => false,
        }
    }

    pub fn is_flag_set<S>(&self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        match self.inner.get(name.as_ref()) {
            Some(container) => match container {
                OptionValue::Flag(flag) => *flag,
                OptionValue::Single(_) => false,
                OptionValue::Multiple(_) => false,
            },

            None => false,
        }
    }

    pub fn has_value<S1, S2>(&self, name: S1, value: S2) -> bool
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        match self.inner.get(name.as_ref()) {
            Some(container) => match container {
                OptionValue::Flag(_) => false,
                OptionValue::Single(single) => single == value.as_ref(),
                OptionValue::Multiple(values) => values.iter().any(|item| item == value.as_ref()),
            },

            None => false,
        }
    }

    pub fn get<S>(&self, name: S) -> Option<&str>
    where
        S: AsRef<str>,
    {
        match self.inner.get(name.as_ref()) {
            Some(container) => match container {
                OptionValue::Flag(_) => None,
                OptionValue::Single(value) => Some(value.as_str()),
                OptionValue::Multiple(values) => values.iter().map(String::as_str).next(),
            },

            None => None,
        }
    }

    pub fn iter<S>(&self, name: S) -> Option<impl Iterator<Item = &str>>
    where
        S: AsRef<str>,
    {
        match self.inner.get(name.as_ref()) {
            Some(container) => match container {
                OptionValue::Flag(_) => None,
                OptionValue::Single(value) => Some(Either::Left(once(value.as_str()))),
                OptionValue::Multiple(values) => {
                    Some(Either::Right(values.iter().map(String::as_str)))
                }
            },

            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::from_env;
    use super::*;

    #[test]
    fn options_parsing() {
        let options = from_env::<Options, _>(into_env(vec![
            "name1",
            "name2=true",
            "name3=false",
            "name4=",
            "name5=value",
            "name6=de=limiter",
            "name7=false,true",
            "name8=value1,value2,value3",
            "name9=value1,val=ue2,value3",
            "build-arg:name10",
            "build-arg:name11=value",
        ]))
        .unwrap();

        assert_eq!(options.inner["name1"], OptionValue::Flag(true));
        assert_eq!(options.inner["name2"], OptionValue::Flag(true));
        assert_eq!(options.inner["name3"], OptionValue::Flag(false));
        assert_eq!(options.inner["name4"], OptionValue::Flag(true));

        assert_eq!(options.inner["name5"], OptionValue::Single("value".into()));
        assert_eq!(
            options.inner["name6"],
            OptionValue::Single("de=limiter".into())
        );
        assert_eq!(
            options.inner["name7"],
            OptionValue::Multiple(vec!["false".into(), "true".into()])
        );
        assert_eq!(
            options.inner["name8"],
            OptionValue::Multiple(vec!["value1".into(), "value2".into(), "value3".into()])
        );
        assert_eq!(
            options.inner["name9"],
            OptionValue::Multiple(vec!["value1".into(), "val=ue2".into(), "value3".into()])
        );

        assert_eq!(options.inner["name10"], OptionValue::Flag(true));
        assert_eq!(options.inner["name11"], OptionValue::Single("value".into()));
    }

    #[test]
    fn has_method() {
        let options = from_env::<Options, _>(into_env(vec![
            "option1",
            "option2=true",
            "option3=false",
            "option4=true,false",
        ]))
        .unwrap();

        assert_eq!(options.has("option1"), true);
        assert_eq!(options.has("option2"), true);
        assert_eq!(options.has("option3"), false);
        assert_eq!(options.has("option4"), true);
    }

    #[test]
    fn has_value_method() {
        let options = from_env::<Options, _>(into_env(vec![
            "option1",
            "option2=true",
            "option3=true,false,any_other",
        ]))
        .unwrap();

        assert_eq!(options.has_value("option1", ""), false);
        assert_eq!(options.has_value("option1", "any_other"), false);
        assert_eq!(options.has_value("option2", ""), false);
        assert_eq!(options.has_value("option2", "any_other"), false);
        assert_eq!(options.has_value("option3", "true"), true);
        assert_eq!(options.has_value("option3", "false"), true);
        assert_eq!(options.has_value("option3", "any_other"), true);
        assert_eq!(options.has_value("option3", "missing"), false);
    }

    #[test]
    fn iter_method() {
        let options = from_env::<Options, _>(into_env(vec![
            "option1",
            "option2=true",
            "option3=true,false,any_other",
        ]))
        .unwrap();

        assert!(options.iter("option1").is_none());
        assert!(options.iter("option2").is_none());
        assert!(options.iter("option4").is_none());

        assert!(options.iter("option3").is_some());
        assert_eq!(
            options.iter("option3").unwrap().collect::<Vec<_>>(),
            vec!["true", "false", "any_other"]
        );
    }

    fn into_env(args: Vec<&'static str>) -> Vec<(String, String)> {
        args.into_iter()
            .enumerate()
            .map(|(index, option)| {
                (
                    format!("BUILDKIT_FRONTEND_OPT_{}", index),
                    String::from(option),
                )
            })
            .collect()
    }
}
