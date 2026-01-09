use std::fmt;

/// Errors that can occur during parser initialization.
#[derive(Debug, Clone, PartialEq)]
pub enum LanguageError {
    IncompatibleLanguageVersion,
    LanguageNotInitialized,
}

impl fmt::Display for LanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageError::IncompatibleLanguageVersion => write!(
                f,
                "Tree-sitter language version is incompatible with parser"
            ),
            LanguageError::LanguageNotInitialized => {
                write!(f, "Parser was not initialized with the language")
            }
        }
    }
}

/// Errors that can occur when parsing values from DSL source.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueParseError {
    UnknownValueKind,
    MissingValue,
    MissingParseMethod,
    InvalidBoolean(String),
    InvalidInteger(String),
    InvalidFloat(String),
    InvalidCurrencyFormat {
        source: String,
        parts_count: usize,
    },
    InvalidCurrencyAmount(String),
    InvalidCurrencyCode(String),
    InvalidReferenceFormat {
        source: String,
        parts_count: usize,
    },
    InvalidDate(String),
    InvalidDateTime(String),
    InvalidTimezone(String),
    HeterogeneousList {
        expected_type: String,
        found_type: String,
        index: usize,
    },
}

impl fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueParseError::UnknownValueKind => {
                write!(
                    f,
                    "Value type could not be determined from tree-sitter node"
                )
            }
            ValueParseError::MissingValue => {
                write!(f, "Field is missing its value assignment")
            }
            ValueParseError::MissingParseMethod => {
                write!(f, "No parsing method implemented for this value type")
            }
            ValueParseError::InvalidBoolean(value) => {
                write!(f, "Boolean value could not be parsed: '{}'", value)
            }
            ValueParseError::InvalidInteger(value) => {
                write!(f, "Integer value could not be parsed: '{}'", value)
            }
            ValueParseError::InvalidFloat(value) => {
                write!(f, "Float value could not be parsed: '{}'", value)
            }
            ValueParseError::InvalidCurrencyFormat {
                source,
                parts_count,
            } => {
                write!(
                    f,
                    "Currency format is invalid (expected 'amount currency'): '{}' has {} part(s)",
                    source, parts_count
                )
            }
            ValueParseError::InvalidCurrencyAmount(amount) => {
                write!(
                    f,
                    "Currency amount could not be parsed as decimal: '{}'",
                    amount
                )
            }
            ValueParseError::InvalidCurrencyCode(code) => {
                write!(f, "Currency code is not recognized: '{}'", code)
            }
            ValueParseError::InvalidReferenceFormat {
                source,
                parts_count,
            } => {
                write!(
                    f,
                    "Reference format is invalid (expected 2 or 3 dot-separated parts): '{}' has {} part(s)",
                    source, parts_count
                )
            }
            ValueParseError::InvalidDate(date) => {
                write!(f, "Date value could not be parsed: '{}'", date)
            }
            ValueParseError::InvalidDateTime(datetime) => {
                write!(f, "DateTime value could not be parsed: '{}'", datetime)
            }
            ValueParseError::InvalidTimezone(timezone) => {
                write!(f, "Timezone offset could not be parsed: '{}'", timezone)
            }
            ValueParseError::HeterogeneousList {
                expected_type,
                found_type,
                index,
            } => {
                write!(
                    f,
                    "List contains values of different types (but must be homogeneous): expected {}, found {} at index {}",
                    expected_type, found_type, index
                )
            }
        }
    }
}
