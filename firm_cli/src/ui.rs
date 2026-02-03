use clap::ValueEnum;
use console::Style;
use firm_core::{Entity, EntitySchema};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fmt, time::Duration};

use super::logging;

/// Helpers to create consistent console UI styles.
pub struct UiStyle;

impl UiStyle {
    /// A regular message.
    pub fn normal() -> Style {
        Style::new()
    }

    /// A highlighted message.
    pub fn highlight() -> Style {
        Style::new().bold()
    }

    /// A dim message.
    pub fn dim() -> Style {
        Style::new().dim()
    }

    /// A warning message.
    pub fn warning() -> Style {
        Style::new().yellow().bold()
    }

    /// A success message.
    pub fn success() -> Style {
        Style::new().green().bold()
    }

    /// An error message
    pub fn error() -> Style {
        Style::new().red().bold()
    }
}

/// Prints a header message.
pub fn header(msg: &str) {
    eprintln!("{}", UiStyle::highlight().apply_to(msg));
}

/// Prints a debug message.
pub fn debug(msg: &str) {
    eprintln!("{}", UiStyle::dim().apply_to(msg));
}

/// Prints an info message.
pub fn info(msg: &str) {
    eprintln!("{}", UiStyle::normal().apply_to(msg));
}

/// Prints a warning message.
pub fn warning(msg: &str) {
    eprintln!("{}", UiStyle::warning().apply_to(msg));
}

/// Prints a success message.
pub fn success(msg: &str) {
    eprintln!("{}", UiStyle::success().apply_to(msg));
}

/// Prints an error message.
pub fn error(msg: &str) {
    eprintln!("{}", UiStyle::error().apply_to(msg));
}

/// Prints an error message with added details.
pub fn error_with_details(main_msg: &str, details: &str) {
    eprintln!("{}", UiStyle::error().apply_to(main_msg));
    eprintln!("   {}", UiStyle::dim().apply_to(details));
}

/// Selects the output format used by the CLI.
#[derive(Clone, Debug, ValueEnum, PartialEq)]
#[derive(Default)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Json,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Pretty => write!(f, "pretty"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}


/// Outputs a single entity in pretty format.
pub fn pretty_output_entity_single(entity: &Entity) {
    println!("\n{}", entity);
}

/// Outputs a list of entities in pretty format.
pub fn pretty_output_entity_list(entities: &Vec<&Entity>) {
    for (i, entity) in entities.iter().enumerate() {
        pretty_output_entity_single(entity);

        // Add a separator after each entity, except for the last one.
        if i < entities.len() - 1 {
            println!("---------------------------------------");
        }
    }
}

/// Outputs a single entity schema in pretty format.
pub fn pretty_output_schema_single(schema: &EntitySchema) {
    println!("\n{}", schema);
}

/// Outputs a list of entity schemas in pretty format.
pub fn pretty_output_schema_list(schemas: &Vec<&EntitySchema>) {
    for (i, schema) in schemas.iter().enumerate() {
        pretty_output_schema_single(schema);

        // Add a separator after each entity, except for the last one.
        if i < schemas.len() - 1 {
            println!("---------------------------------------");
        }
    }
}

/// Outputs a serde-serializable object in json format.
pub fn json_output<T: serde::Serialize>(data: &T) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        println!("{}", json);
    }
}

/// Outputs raw text (e.g., file paths, simple values).
pub fn raw_output(text: &str) {
    println!("{}", text);
}

/// Creates a spinner progress indicator.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .expect("Invalid template"),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(msg.to_string());

    let tracker = logging::get_multi_progress();
    tracker.add(pb.clone());

    pb
}

/// Creates a progress bar indicator.
pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} {msg} [{bar}] {pos}/{len}")
            .expect("Invalid template")
            .progress_chars("# "),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    let tracker = logging::get_multi_progress();
    tracker.add(pb.clone());

    pb
}
