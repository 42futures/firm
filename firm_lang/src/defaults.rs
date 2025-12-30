use firm_core::{EntitySchema, EntityType, FieldId, FieldType};

/// Instantiates all default schemas.
pub fn all_default_schemas() -> Vec<EntitySchema> {
    vec![
        // Core entities
        person(),
        organization(),
        industry(),
        // Customer relations
        account(),
        channel(),
        lead(),
        contact(),
        interaction(),
        opportunity(),
        // Work management
        strategy(),
        objective(),
        key_result(),
        project(),
        task(),
        review(),
        // Resources
        file_asset(),
    ]
}

/// An individual person (an Agent in the REA model).
///
/// This is a fundamental entity models people, whether they are employees, customers, or partners.
pub fn person() -> EntitySchema {
    EntitySchema::new(EntityType::new("person"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("email"), FieldType::String)
        .with_optional_field(FieldId::new("phone"), FieldType::String)
        .with_optional_field(FieldId::new("urls"), FieldType::List)
}

/// An organization, company, or group (an Agent in the REA model).
///
/// A fundamental entity for modeling businesses, institutions, or collections of people.
pub fn organization() -> EntitySchema {
    EntitySchema::new(EntityType::new("organization"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("address"), FieldType::String)
        .with_optional_field(FieldId::new("email"), FieldType::String)
        .with_optional_field(FieldId::new("phone"), FieldType::String)
        .with_optional_field(FieldId::new("urls"), FieldType::List)
        .with_optional_field(FieldId::new("vat_id"), FieldType::String)
        .with_optional_field(FieldId::new("industry_ref"), FieldType::Reference)
}

/// Represents an industry or business sector.
///
/// This entity is used to classify organizations, helping to categorize and query
/// businesses by their area of operation.
pub fn industry() -> EntitySchema {
    EntitySchema::new(EntityType::new("industry"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("sector"), FieldType::String)
        .with_optional_field(FieldId::new("classification_code"), FieldType::String)
        .with_optional_field(FieldId::new("classification_system"), FieldType::String)
}

/// Represents a business relationship with an organization, typically a customer.
///
/// This is a contextual entity that links to an organization and tracks the state
/// of your relationship with them.
pub fn account() -> EntitySchema {
    EntitySchema::new(EntityType::new("account"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("organization_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("status"), FieldType::String)
}

/// Represents a communication or marketing channel.
///
/// Used to track where interactions, leads, and opportunities originate from,
/// such as "Email", "Website", or "Conference".
pub fn channel() -> EntitySchema {
    EntitySchema::new(EntityType::new("channel"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("type"), FieldType::String)
        .with_optional_field(FieldId::new("description"), FieldType::String)
}

/// Represents a potential business lead.
///
/// A contextual entity that captures an initial expression of interest. It typically
/// references a person, contact or account and tracks its qualification status.
pub fn lead() -> EntitySchema {
    EntitySchema::new(EntityType::new("lead"))
        .with_metadata()
        .with_required_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_required_field(FieldId::new("status"), FieldType::String)
        .with_optional_field(FieldId::new("person_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("account_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("score"), FieldType::Integer)
}

/// Represents a person in the context of a business relationship.
///
/// This contextual entity links a fundamental person to an account or other business
/// context, defining their role and status.
pub fn contact() -> EntitySchema {
    EntitySchema::new(EntityType::new("contact"))
        .with_metadata()
        .with_optional_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("person_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("account_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("role"), FieldType::String)
        .with_optional_field(FieldId::new("status"), FieldType::String)
}

/// Represents a specific interaction or communication (an Event in the REA model).
///
/// Used to log meetings, calls, emails, chats, or any other touchpoint with contacts
/// or accounts.
pub fn interaction() -> EntitySchema {
    EntitySchema::new(EntityType::new("interaction"))
        .with_metadata()
        .with_required_field(FieldId::new("type"), FieldType::String)
        .with_required_field(FieldId::new("subject"), FieldType::String)
        .with_required_field(FieldId::new("initiator_ref"), FieldType::Reference)
        .with_required_field(FieldId::new("primary_contact_ref"), FieldType::Reference)
        .with_required_field(FieldId::new("interaction_date"), FieldType::DateTime)
        .with_optional_field(FieldId::new("outcome"), FieldType::String)
        .with_optional_field(FieldId::new("secondary_contacts_ref"), FieldType::List)
        .with_optional_field(FieldId::new("channel_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("opportunity_ref"), FieldType::Reference)
}

/// Represents a potential sale or business deal.
///
/// This entity tracks a qualified lead through the sales pipeline, capturing its value,
/// status, and probability of success.
pub fn opportunity() -> EntitySchema {
    EntitySchema::new(EntityType::new("opportunity"))
        .with_metadata()
        .with_required_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("status"), FieldType::String)
        .with_optional_field(FieldId::new("value"), FieldType::Currency)
        .with_optional_field(FieldId::new("probability"), FieldType::Integer)
}

/// Represents a high-level, long-term plan or goal.
///
/// A foundational element for work management, strategies provide direction and can be
/// linked to by more low-level entities.
pub fn strategy() -> EntitySchema {
    EntitySchema::new(EntityType::new("strategy"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("description"), FieldType::String)
        .with_optional_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("status"), FieldType::String)
        .with_optional_field(FieldId::new("start_date"), FieldType::DateTime)
        .with_optional_field(FieldId::new("end_date"), FieldType::DateTime)
}

/// Represents a specific, measurable goal that contributes to a strategy.
///
/// Objectives break down high-level strategies into actionable targets that are
/// further defined by key result entities.
pub fn objective() -> EntitySchema {
    EntitySchema::new(EntityType::new("objective"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("description"), FieldType::String)
        .with_optional_field(FieldId::new("strategy_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("status"), FieldType::String)
        .with_optional_field(FieldId::new("start_date"), FieldType::DateTime)
        .with_optional_field(FieldId::new("end_date"), FieldType::DateTime)
}

/// Represents a measurable outcome used to track an objective.
///
/// Key results make objectives concrete with quantified success metrics.
pub fn key_result() -> EntitySchema {
    EntitySchema::new(EntityType::new("key_result"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("objective_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("start_value"), FieldType::Float)
        .with_optional_field(FieldId::new("target_value"), FieldType::Float)
        .with_optional_field(FieldId::new("current_value"), FieldType::Float)
        .with_optional_field(FieldId::new("unit"), FieldType::String)
}

/// Represents a planned initiative to achieve specific objectives.
///
/// A project may a contain tasks and link strategic goals to day-to-day execution.
pub fn project() -> EntitySchema {
    EntitySchema::new(EntityType::new("project"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("status"), FieldType::String)
        .with_optional_field(FieldId::new("description"), FieldType::String)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("objective_refs"), FieldType::List)
        .with_optional_field(FieldId::new("due_date"), FieldType::DateTime)
}

/// Represents a single, actionable unit of work.
///
/// Tasks are the most granular items in work management and are typically associated
/// with a project or another source entity.
pub fn task() -> EntitySchema {
    EntitySchema::new(EntityType::new("task"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_optional_field(FieldId::new("description"), FieldType::String)
        .with_optional_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("assignee_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("due_date"), FieldType::DateTime)
        .with_optional_field(FieldId::new("is_completed"), FieldType::Boolean)
        .with_optional_field(FieldId::new("completed_at"), FieldType::DateTime)
}

/// Represents a periodic review or meeting (an Event in the REA model).
///
/// Used to track progress on projects, objectives, or strategies, linking together
/// relevant people and resources for a specific point in time.
pub fn review() -> EntitySchema {
    EntitySchema::new(EntityType::new("review"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("date"), FieldType::DateTime)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("source_refs"), FieldType::List)
        .with_optional_field(FieldId::new("attendee_refs"), FieldType::List)
}

/// Represents a digital file or document (a Resource in the REA model).
///
/// This entity links to a file path and can be associated with any other entity,
/// serving as a way to track project assets, contracts, or other documents.
pub fn file_asset() -> EntitySchema {
    EntitySchema::new(EntityType::new("file_asset"))
        .with_metadata()
        .with_required_field(FieldId::new("name"), FieldType::String)
        .with_required_field(FieldId::new("path"), FieldType::Path)
        .with_optional_field(FieldId::new("description"), FieldType::String)
        .with_optional_field(FieldId::new("source_ref"), FieldType::Reference)
        .with_optional_field(FieldId::new("owner_ref"), FieldType::Reference)
}
