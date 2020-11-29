use serde_derive::{Deserialize, Serialize};

use crate::threadupdater::ParserType;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Parser {
    Basic,
    #[serde(rename = "4chan")]
    FourChan,
    Tinyboard,
    AspNetChan,
    Kusabax,
    FoolFuuka,
}

impl From<Parser> for ParserType {
    fn from(parser: Parser) -> Self {
        match parser {
            Parser::Basic => ParserType::Basic,
            Parser::FourChan => ParserType::FourChan,
            Parser::Tinyboard => ParserType::Tinyboard,
            Parser::AspNetChan => ParserType::AspNetChan,
            Parser::Kusabax => ParserType::Kusabax,
            Parser::FoolFuuka => ParserType::FoolFuuka,
        }
    }
}

impl From<ParserType> for Parser {
    fn from(parser: ParserType) -> Self {
        match parser {
            ParserType::Basic => Parser::Basic,
            ParserType::FourChan => Parser::FourChan,
            ParserType::Tinyboard => Parser::Tinyboard,
            ParserType::AspNetChan => Parser::AspNetChan,
            ParserType::Kusabax => Parser::Kusabax,
            ParserType::FoolFuuka => Parser::FoolFuuka,
        }
    }
}
