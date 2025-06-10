use std::str::FromStr;

use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameAll {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl FromStr for RenameAll {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lowercase" => Ok(RenameAll::Lowercase),
            "UPPERCASE" => Ok(RenameAll::Uppercase),
            "PascalCase" => Ok(RenameAll::PascalCase),
            "camelCase" => Ok(RenameAll::CamelCase),
            "snake_case" => Ok(RenameAll::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(RenameAll::ScreamingSnakeCase),
            "kebab-case" => Ok(RenameAll::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(RenameAll::ScreamingKebabCase),
            _ => Err(format!("Invalid rename_all value: {s}")),
        }
    }
}

impl std::fmt::Display for RenameAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenameAll::Lowercase => write!(f, "lowercase"),
            RenameAll::Uppercase => write!(f, "UPPERCASE"),
            RenameAll::PascalCase => write!(f, "PascalCase"),
            RenameAll::CamelCase => write!(f, "camelCase"),
            RenameAll::SnakeCase => write!(f, "snake_case"),
            RenameAll::ScreamingSnakeCase => write!(f, "SCREAMING_SNAKE_CASE"),
            RenameAll::KebabCase => write!(f, "kebab-case"),
            RenameAll::ScreamingKebabCase => write!(f, "SCREAMING-KEBAB-CASE"),
        }
    }
}

impl RenameAll {
    pub fn apply(&self, input: String) -> String {
        match self {
            RenameAll::Lowercase => input.to_lowercase(),
            RenameAll::Uppercase => input.to_uppercase(),
            RenameAll::PascalCase => input.to_pascal_case(),
            RenameAll::CamelCase => input.to_lower_camel_case(),
            RenameAll::SnakeCase => input.to_snake_case(),
            RenameAll::ScreamingSnakeCase => input.to_shouty_snake_case(),
            RenameAll::KebabCase => input.to_kebab_case(),
            RenameAll::ScreamingKebabCase => input.to_shouty_kebab_case(),
        }
    }
}
