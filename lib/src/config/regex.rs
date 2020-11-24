use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::error::*;

/// One or more regexes
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Regexes {
    Single(String),
    Multiple(Vec<String>),
}

impl Regexes {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value.clone()],
            Self::Multiple(values) => values.clone(),
        }
    }

    pub fn build_regexes(&self) -> Result<Vec<regex::Regex>, ChandlerError> {
        let mut regexes: Vec<regex::Regex> = Vec::new();

        fn build_regex(pattern: &str) -> Result<regex::Regex, ChandlerError> {
            let mut builder = regex::RegexBuilder::new(pattern);
            builder.multi_line(true);

            builder.build().map_err(|err| {
                ChandlerError::Other(Cow::Owned(format!("Invalid regex '{}': {}", pattern, err.to_string())))
            })
        }

        let regex_strings = self.to_vec();

        for regex_str in regex_strings.iter() {
            regexes.push(build_regex(regex_str)?);
        }

        Ok(regexes)
    }
}
