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

impl RenameAll {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(RenameAll::Lowercase),
            "UPPERCASE" => Some(RenameAll::Uppercase),
            "PascalCase" => Some(RenameAll::PascalCase),
            "camelCase" => Some(RenameAll::CamelCase),
            "snake_case" => Some(RenameAll::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameAll::ScreamingSnakeCase),
            "kebab-case" => Some(RenameAll::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameAll::ScreamingKebabCase),
            _ => None,
        }
    }

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
