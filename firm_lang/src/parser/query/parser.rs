//! Parser for query language using pest

use pest::Parser;
use pest_derive::Parser;

use super::parsed_query::*;

#[derive(Parser)]
#[grammar = "parser/query/grammar.pest"]
pub struct QueryParser;

/// Error type for query parsing
#[derive(Debug, Clone, PartialEq)]
pub enum QueryParseError {
    SyntaxError(String),
    InvalidNumber(String),
}

impl std::fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryParseError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            QueryParseError::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
        }
    }
}

impl std::error::Error for QueryParseError {}

/// Parse a query string into a ParsedQuery
pub fn parse_query(input: &str) -> Result<ParsedQuery, QueryParseError> {
    let pairs = QueryParser::parse(Rule::query, input)
        .map_err(|e| QueryParseError::SyntaxError(e.to_string()))?;

    let mut from_clause = None;
    let mut operations = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::query => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::from_clause => {
                            from_clause = Some(parse_from_clause(inner_pair)?);
                        }
                        Rule::operation => {
                            operations.push(parse_operation(inner_pair)?);
                        }
                        Rule::EOI => {}
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let from = from_clause.ok_or_else(|| {
        QueryParseError::SyntaxError("Query must start with 'from' clause".to_string())
    })?;

    Ok(ParsedQuery { from, operations })
}

fn parse_from_clause(pair: pest::iterators::Pair<Rule>) -> Result<ParsedFromClause, QueryParseError> {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::entity_selector {
            let selector = parse_entity_selector(inner_pair)?;
            return Ok(ParsedFromClause { selector });
        }
    }
    Err(QueryParseError::SyntaxError(
        "Invalid from clause".to_string(),
    ))
}

fn parse_entity_selector(
    pair: pest::iterators::Pair<Rule>,
) -> Result<ParsedEntitySelector, QueryParseError> {
    let text = pair.as_str();
    if text == "*" {
        Ok(ParsedEntitySelector::Wildcard)
    } else {
        Ok(ParsedEntitySelector::Type(text.to_string()))
    }
}

fn parse_operation(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperation, QueryParseError> {
    let inner_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| QueryParseError::SyntaxError("Empty operation".to_string()))?;

    match inner_pair.as_rule() {
        Rule::where_clause => parse_where_clause(inner_pair),
        Rule::related_clause => parse_related_clause(inner_pair),
        Rule::order_clause => parse_order_clause(inner_pair),
        Rule::limit_clause => parse_limit_clause(inner_pair),
        _ => Err(QueryParseError::SyntaxError(format!(
            "Unknown operation: {:?}",
            inner_pair.as_rule()
        ))),
    }
}

fn parse_where_clause(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperation, QueryParseError> {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::condition {
            let condition = parse_condition(inner_pair)?;
            return Ok(ParsedOperation::Where(condition));
        }
    }
    Err(QueryParseError::SyntaxError(
        "Invalid where clause".to_string(),
    ))
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<ParsedCondition, QueryParseError> {
    let mut inner = pair.into_inner();

    let field_pair = inner
        .next()
        .ok_or_else(|| QueryParseError::SyntaxError("Missing field in condition".to_string()))?;

    let field = match field_pair.as_rule() {
        Rule::metadata_field => {
            let metadata_name = field_pair
                .into_inner()
                .next()
                .ok_or_else(|| {
                    QueryParseError::SyntaxError("Invalid metadata field".to_string())
                })?
                .as_str()
                .to_string();
            ParsedField::Metadata(metadata_name)
        }
        Rule::field_name => ParsedField::Regular(field_pair.as_str().to_string()),
        _ => {
            return Err(QueryParseError::SyntaxError(
                "Invalid field in condition".to_string(),
            ))
        }
    };

    let operator_pair = inner.next().ok_or_else(|| {
        QueryParseError::SyntaxError("Missing operator in condition".to_string())
    })?;
    let operator = parse_operator(operator_pair)?;

    let value_pair = inner
        .next()
        .ok_or_else(|| QueryParseError::SyntaxError("Missing value in condition".to_string()))?;
    let value = parse_value(value_pair)?;

    Ok(ParsedCondition {
        field,
        operator,
        value,
    })
}

fn parse_operator(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperator, QueryParseError> {
    match pair.as_str() {
        "==" => Ok(ParsedOperator::Equal),
        "!=" => Ok(ParsedOperator::NotEqual),
        ">" => Ok(ParsedOperator::GreaterThan),
        "<" => Ok(ParsedOperator::LessThan),
        ">=" => Ok(ParsedOperator::GreaterOrEqual),
        "<=" => Ok(ParsedOperator::LessOrEqual),
        "contains" => Ok(ParsedOperator::Contains),
        "startswith" => Ok(ParsedOperator::StartsWith),
        "endswith" => Ok(ParsedOperator::EndsWith),
        "in" => Ok(ParsedOperator::In),
        _ => Err(QueryParseError::SyntaxError(format!(
            "Unknown operator: {}",
            pair.as_str()
        ))),
    }
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<ParsedQueryValue, QueryParseError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| QueryParseError::SyntaxError("Empty value".to_string()))?;

    match inner.as_rule() {
        Rule::string => {
            let string_content = inner
                .into_inner()
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Empty string".to_string()))?
                .as_str()
                .to_string();
            Ok(ParsedQueryValue::String(string_content))
        }
        Rule::number => {
            let num_str = inner.as_str();
            let num = num_str.parse::<f64>().map_err(|_| {
                QueryParseError::InvalidNumber(format!("Cannot parse number: {}", num_str))
            })?;
            Ok(ParsedQueryValue::Number(num))
        }
        Rule::boolean => {
            let bool_val = inner.as_str() == "true";
            Ok(ParsedQueryValue::Boolean(bool_val))
        }
        Rule::currency => {
            let mut inner_pairs = inner.into_inner();
            let num_str = inner_pairs
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Missing amount in currency".to_string()))?
                .as_str();
            let amount = num_str.parse::<f64>().map_err(|_| {
                QueryParseError::InvalidNumber(format!("Cannot parse currency amount: {}", num_str))
            })?;
            let code = inner_pairs
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Missing currency code".to_string()))?
                .as_str()
                .to_string();
            Ok(ParsedQueryValue::Currency { amount, code })
        }
        Rule::datetime => {
            Ok(ParsedQueryValue::DateTime(inner.as_str().to_string()))
        }
        Rule::reference => {
            Ok(ParsedQueryValue::Reference(inner.as_str().to_string()))
        }
        Rule::path => {
            let string_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Missing path string".to_string()))?;
            let path_content = string_pair
                .into_inner()
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Empty path".to_string()))?
                .as_str()
                .to_string();
            Ok(ParsedQueryValue::Path(path_content))
        }
        Rule::enum_value => {
            let string_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Missing enum string".to_string()))?;
            let enum_content = string_pair
                .into_inner()
                .next()
                .ok_or_else(|| QueryParseError::SyntaxError("Empty enum".to_string()))?
                .as_str()
                .to_string();
            Ok(ParsedQueryValue::Enum(enum_content))
        }
        Rule::list => {
            let mut values = Vec::new();
            for list_item in inner.into_inner() {
                if list_item.as_rule() == Rule::value {
                    values.push(parse_value(list_item)?);
                }
            }
            Ok(ParsedQueryValue::List(values))
        }
        _ => Err(QueryParseError::SyntaxError(format!(
            "Unknown value type: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_related_clause(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperation, QueryParseError> {
    let mut degree = None;
    let mut selector = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::degree => {
                let degree_pair = inner_pair.into_inner().next().ok_or_else(|| {
                    QueryParseError::SyntaxError("Invalid degree".to_string())
                })?;
                let degree_num = degree_pair.as_str().parse::<usize>().map_err(|_| {
                    QueryParseError::InvalidNumber(format!(
                        "Invalid degree number: {}",
                        degree_pair.as_str()
                    ))
                })?;
                degree = Some(degree_num);
            }
            Rule::entity_selector => {
                selector = Some(parse_entity_selector(inner_pair)?);
            }
            _ => {}
        }
    }

    Ok(ParsedOperation::Related { degree, selector })
}

fn parse_order_clause(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperation, QueryParseError> {
    let mut field = None;
    let mut direction = ParsedDirection::default();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::order_field => {
                // order_field can be either metadata_field or field_name
                let field_pair = inner_pair.into_inner().next().ok_or_else(|| {
                    QueryParseError::SyntaxError("Invalid order field".to_string())
                })?;

                match field_pair.as_rule() {
                    Rule::metadata_field => {
                        let metadata_name = field_pair
                            .into_inner()
                            .next()
                            .ok_or_else(|| {
                                QueryParseError::SyntaxError("Invalid metadata field".to_string())
                            })?
                            .as_str()
                            .to_string();
                        field = Some(ParsedField::Metadata(metadata_name));
                    }
                    Rule::field_name => {
                        field = Some(ParsedField::Regular(field_pair.as_str().to_string()));
                    }
                    _ => {
                        return Err(QueryParseError::SyntaxError(
                            "Invalid field in order clause".to_string(),
                        ))
                    }
                }
            }
            Rule::direction => {
                direction = match inner_pair.as_str() {
                    "asc" => ParsedDirection::Ascending,
                    "desc" => ParsedDirection::Descending,
                    _ => ParsedDirection::default(),
                };
            }
            _ => {}
        }
    }

    let field = field.ok_or_else(|| {
        QueryParseError::SyntaxError("Missing field in order clause".to_string())
    })?;

    Ok(ParsedOperation::Order { field, direction })
}

fn parse_limit_clause(pair: pest::iterators::Pair<Rule>) -> Result<ParsedOperation, QueryParseError> {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::number {
            let limit = inner_pair.as_str().parse::<usize>().map_err(|_| {
                QueryParseError::InvalidNumber(format!(
                    "Invalid limit number: {}",
                    inner_pair.as_str()
                ))
            })?;
            return Ok(ParsedOperation::Limit(limit));
        }
    }
    Err(QueryParseError::SyntaxError(
        "Invalid limit clause".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query_str = "from task | where is_completed == false | limit 5";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        assert_eq!(query.from.selector, ParsedEntitySelector::Type("task".to_string()));
        assert_eq!(query.operations.len(), 2);
    }

    #[test]
    fn test_parse_wildcard() {
        let query_str = "from * | where @type == \"task\"";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        assert_eq!(query.from.selector, ParsedEntitySelector::Wildcard);
    }

    #[test]
    fn test_parse_related_with_degree() {
        let query_str = "from project | related(2) task";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Related { degree, selector }) = query.operations.first() {
            assert_eq!(*degree, Some(2));
            assert_eq!(*selector, Some(ParsedEntitySelector::Type("task".to_string())));
        } else {
            panic!("Expected Related operation");
        }
    }

    #[test]
    fn test_parse_order_with_direction() {
        let query_str = "from task | order due_date desc";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Order { field, direction }) = query.operations.first() {
            assert_eq!(*field, ParsedField::Regular("due_date".to_string()));
            assert_eq!(*direction, ParsedDirection::Descending);
        } else {
            panic!("Expected Order operation");
        }
    }

    #[test]
    fn test_parse_currency_value() {
        let query_str = "from project | where budget == 5000.50 USD";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Where(condition)) = query.operations.first() {
            if let ParsedQueryValue::Currency { amount, code } = &condition.value {
                assert_eq!(*amount, 5000.50);
                assert_eq!(code, "USD");
            } else {
                panic!("Expected Currency value");
            }
        }
    }

    #[test]
    fn test_parse_datetime_value() {
        let query_str = "from task | where created > 2025-01-15";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Where(condition)) = query.operations.first() {
            assert!(matches!(condition.value, ParsedQueryValue::DateTime(_)));
        }
    }

    #[test]
    fn test_parse_reference_value() {
        let query_str = "from task | where assignee == person.john_doe";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Where(condition)) = query.operations.first() {
            if let ParsedQueryValue::Reference(ref_str) = &condition.value {
                assert_eq!(ref_str, "person.john_doe");
            } else {
                panic!("Expected Reference value");
            }
        }
    }

    #[test]
    fn test_parse_enum_value() {
        let query_str = "from task | where status == enum\"completed\"";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Where(condition)) = query.operations.first() {
            if let ParsedQueryValue::Enum(enum_val) = &condition.value {
                assert_eq!(enum_val, "completed");
            } else {
                panic!("Expected Enum value");
            }
        }
    }

    #[test]
    fn test_parse_path_value() {
        let query_str = "from document | where file == path\"./file.pdf\"";
        let result = parse_query(query_str);
        assert!(result.is_ok());

        let query = result.unwrap();
        if let Some(ParsedOperation::Where(condition)) = query.operations.first() {
            if let ParsedQueryValue::Path(path_str) = &condition.value {
                assert_eq!(path_str, "./file.pdf");
            } else {
                panic!("Expected Path value");
            }
        }
    }
}
