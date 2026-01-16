# Examples

Here are some practical examples of what you might build using the Firm Rust crates.

## Custom reporting tool

Build a tool that generates weekly reports from your workspace:

```rust,no_run
use firm_lang::workspace::Workspace;
use firm_core::{
    EntityGraph, EntityType, FieldId,
    Query, EntitySelector, QueryOperation,
    FilterCondition, FilterOperator, FilterValue, FieldRef
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load workspace
    let mut workspace = Workspace::new();
    workspace.load_directory("./my_workspace")?;
    let build = workspace.build()?;
    
    // Build graph
    let mut graph = EntityGraph::new();
    graph.add_entities(build.entities)?;
    graph.build();
    
    // Find completed tasks
    let query = Query::new(EntitySelector::Type(EntityType::new("task")))
        .with_operation(QueryOperation::Where(
            FilterCondition::new(
                FieldRef::Regular(FieldId::new("is_completed")),
                FilterOperator::Equal,
                FilterValue::Boolean(true),
            )
        ));
    
    let completed_tasks = query.execute(&graph);
    
    // Generate report
    println!("Weekly Report");
    println!("Completed {} tasks", completed_tasks.len());
    
    Ok(())
}
```

## Custom validation

Add business-specific validation rules:

```rust,no_run
use firm_core::{EntityGraph, EntityType, FieldId};

fn validate_business_rules(graph: &EntityGraph) -> Result<(), String> {
    // Ensure every project has an owner
    let projects = graph.list_by_type(&EntityType::new("project"));
    
    for project in projects {
        if project.get_field(&FieldId::new("owner_ref")).is_none() {
            return Err(format!("Project {} missing owner", project.id));
        }
    }
    
    // Ensure all opportunities have a value
    let opportunities = graph.list_by_type(&EntityType::new("opportunity"));
    
    for opp in opportunities {
        if opp.get_field(&FieldId::new("value")).is_none() {
            return Err(format!("Opportunity {} missing value", opp.id));
        }
    }
    
    Ok(())
}
```

## Data synchronization

Sync data between Firm and external systems:

```rust,no_run
use firm_lang::workspace::Workspace;
use firm_core::{Entity, EntityType, FieldId, FieldValue};

fn sync_from_crm() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch from external CRM
    let crm_contacts = fetch_from_crm()?;
    
    // Load existing workspace
    let mut workspace = Workspace::new();
    workspace.load_directory("./workspace")?;
    
    // Create or update contacts
    for crm_contact in crm_contacts {
        let entity = Entity::new(
            EntityId::new(&crm_contact.id),
            EntityType::new("contact")
        )
        .with_field(FieldId::new("name"), FieldValue::String(crm_contact.name))
        .with_field(FieldId::new("email"), FieldValue::String(crm_contact.email));
        
        // Generate DSL and write to file
        let dsl = generate_dsl(&entity)?;
        std::fs::write(
            format!("./workspace/contacts/{}.firm", crm_contact.id),
            dsl
        )?;
    }
    
    Ok(())
}
```

## Custom query tool

Build a specialized query interface using Firm's query API:

```rust,no_run
use firm_core::{
    EntityGraph, EntityType, FieldId,
    Query, EntitySelector, QueryOperation,
    FilterCondition, FilterOperator, FilterValue, FieldRef
};
use rust_decimal::Decimal;

fn find_high_value_opportunities(
    graph: &EntityGraph,
    min_value: Decimal
) -> Vec<&Entity> {
    let query = Query::new(EntitySelector::Type(EntityType::new("opportunity")))
        .with_operation(QueryOperation::Where(
            FilterCondition::new(
                FieldRef::Regular(FieldId::new("value")),
                FilterOperator::GreaterThanOrEqual,
                FilterValue::Decimal(min_value),
            )
        ))
        .with_operation(QueryOperation::Order {
            field: FieldRef::Regular(FieldId::new("value")),
            direction: SortDirection::Descending,
        });
    
    query.execute(graph)
}
```

## Automated task creation

Automatically generate tasks based on events:

```rust,no_run
use firm_core::{Entity, EntityGraph, EntityType, EntityId, FieldId, FieldValue, ReferenceValue};
use firm_lang::generator::generate_dsl;

fn create_followup_tasks(graph: &EntityGraph) -> Result<(), Box<dyn std::error::Error>> {
    // Find interactions from this week without follow-up tasks
    let interactions = graph.list_by_type(&EntityType::new("interaction"));
    
    for interaction in interactions {
        // Check if follow-up task exists
        let related = graph.get_related(&interaction.id, None);
        let has_followup = related
            .map(|entities| {
                entities.iter().any(|e| e.entity_type == EntityType::new("task"))
            })
            .unwrap_or(false);
        
        if !has_followup {
            // Create follow-up task
            let task = Entity::new(
                EntityId::new(&format!("followup_{}", interaction.id.as_str())),
                EntityType::new("task")
            )
            .with_field(
                FieldId::new("name"),
                FieldValue::String(format!("Follow up on {}", interaction.id.as_str()))
            )
            .with_field(
                FieldId::new("source_ref"),
                FieldValue::Reference(ReferenceValue::Entity(interaction.id.clone()))
            );
            
            // Write to file
            let dsl = generate_dsl(&task)?;
            std::fs::write(
                format!("./workspace/tasks/followup_{}.firm", interaction.id.as_str()),
                dsl
            )?;
        }
    }
    
    Ok(())
}
```
