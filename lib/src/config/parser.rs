use serde_derive::{Deserialize, Serialize};

use crate::threadupdater::ParserType;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Parser {
    #[serde(rename = "4chan")]
    FourChan,
}

impl From<Parser> for ParserType {
    fn from(parser: Parser) -> Self {
        match parser {
            Parser::FourChan => ParserType::FourChan,
        }
    }
}

impl From<ParserType> for Parser {
    fn from(parser: ParserType) -> Self {
        match parser {
            ParserType::FourChan => Parser::FourChan,
        }
    }
}
