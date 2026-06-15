use std::collections::HashSet;
use serde_json::Value;
use crate::*;

pub struct OperationValidationContext {
    pub node_ids: HashSet<String>,
    pub node_refs: HashSet<String>,
    pub timeline_refs: HashSet<String>,
    pub states: HashSet<String>,
    pub events: HashSet<String>,
}

impl OperationValidationContext {
    pub fn from_document(document: &strut_core::Document) -> Self {
        let mut node_ids = HashSet::new();
        let mut node_refs = HashSet::new();
        for artboard in &document.artboards {
            collect_operation_node_refs(&artboard.nodes, &mut node_ids, &mut node_refs);
        }

        let mut timeline_refs = HashSet::new();
        for timeline in &document.timelines {
            timeline_refs.insert(timeline.id.to_string());
            timeline_refs.insert(timeline.name.clone());
        }

        let states = document
            .state_machines
            .iter()
            .flat_map(|machine| machine.states.iter().cloned())
            .collect();
        let events = document
            .events
            .iter()
            .map(|event| event.name.clone())
            .collect();

        Self {
            node_ids,
            node_refs,
            timeline_refs,
            states,
            events,
        }
    }

    pub fn has_node_id(&self, value: &str) -> bool {
        self.node_ids.contains(value)
    }

    pub fn has_node_ref(&self, value: &str) -> bool {
        self.node_refs.contains(value)
    }
}

pub struct GeneratedOperationRefs {
    pub node_refs: HashSet<String>,
    pub timeline_refs: HashSet<String>,
    pub event_refs: HashSet<String>,
}

impl GeneratedOperationRefs {
    pub fn from_operations(operations: &[Value]) -> Self {
        let mut node_refs = HashSet::new();
        let mut timeline_refs = HashSet::new();
        let mut event_refs = HashSet::new();
        for operation in operations {
            match operation.get("type").and_then(Value::as_str) {
                Some("create_node") => {
                    if let Some(id) = operation
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|id| !id.trim().is_empty())
                    {
                        node_refs.insert(id.to_string());
                    }
                }
                Some("add_timeline") => {
                    for field in ["id", "name"] {
                        if let Some(value) = operation
                            .get(field)
                            .and_then(Value::as_str)
                            .filter(|value| !value.trim().is_empty())
                        {
                            timeline_refs.insert(value.to_string());
                        }
                    }
                }
                Some("emit_event") => {
                    if let Some(name) = operation
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|name| !name.trim().is_empty())
                    {
                        event_refs.insert(name.to_string());
                    }
                }
                _ => {}
            }
        }
        Self {
            node_refs,
            timeline_refs,
            event_refs,
        }
    }

    pub fn has_node_ref(&self, value: &str) -> bool {
        self.node_refs.contains(value)
    }

    pub fn has_timeline_ref(&self, value: &str) -> bool {
        self.timeline_refs.contains(value)
    }

    pub fn has_event_ref(&self, value: &str) -> bool {
        self.event_refs.contains(value)
    }
}

pub fn collect_operation_node_refs(
    nodes: &[strut_core::Node],
    node_ids: &mut HashSet<String>,
    node_refs: &mut HashSet<String>,
) {
    for node in nodes {
        let id = node.id.to_string();
        node_ids.insert(id.clone());
        node_refs.insert(id);
        node_refs.insert(node.name.clone());
        collect_operation_node_refs(&node.children, node_ids, node_refs);
    }
}

pub fn validate_operation_batches(
    batches: &[OperationBatchRecord],
    document: &strut_core::Document,
) -> Result<(), String> {
    let context = OperationValidationContext::from_document(document);
    let mut ids = HashSet::new();
    for batch in batches {
        if batch.id.trim().is_empty() {
            return Err("operation batch id is required".to_string());
        }
        if !ids.insert(batch.id.as_str()) {
            return Err(format!("duplicate operation batch id '{}'", batch.id));
        }
        if !matches!(
            batch.source_type.as_str(),
            "ai" | "sprite-python" | "manual" | "cli"
        ) {
            return Err(format!(
                "operation batch '{}' has unsupported source type '{}'",
                batch.id, batch.source_type
            ));
        }
        if !matches!(
            batch.status.as_str(),
            "pending" | "applied" | "rejected" | "undone"
        ) {
            return Err(format!(
                "operation batch '{}' has unsupported status '{}'",
                batch.id, batch.status
            ));
        }
        if batch.document_revision_id.trim().is_empty() {
            return Err(format!(
                "operation batch '{}' needs a document revision id",
                batch.id
            ));
        }
        if batch.created_at.trim().is_empty() || batch.updated_at.trim().is_empty() {
            return Err(format!("operation batch '{}' needs timestamps", batch.id));
        }
        if batch.status == "applied" && !batch.validation_result.ok {
            return Err(format!(
                "operation batch '{}' cannot be applied with failed validation",
                batch.id
            ));
        }
        if matches!(batch.status.as_str(), "pending" | "applied" | "undone")
            && batch.operations.is_empty()
        {
            return Err(format!(
                "operation batch '{}' has no meaningful operations",
                batch.id
            ));
        }
        validate_operation_batch_revision(batch)?;
        validate_operation_payloads(batch, &context)?;
    }
    Ok(())
}

pub fn validate_operation_batch_revision(batch: &OperationBatchRecord) -> Result<(), String> {
    if !batch.document_revision_id.starts_with("rev-") {
        return Err(format!(
            "operation batch '{}' has unsupported document revision id '{}'",
            batch.id, batch.document_revision_id
        ));
    }
    if let Some(previous) = &batch.previous_document_revision_id {
        if previous.trim().is_empty() {
            return Err(format!(
                "operation batch '{}' has an empty previous document revision id",
                batch.id
            ));
        }
    }
    Ok(())
}

pub fn validate_operation_payloads(
    batch: &OperationBatchRecord,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let generated_refs = GeneratedOperationRefs::from_operations(&batch.operations);

    for operation in &batch.operations {
        validate_operation_payload(batch, operation, context, &generated_refs)?;
    }
    Ok(())
}

pub fn validate_operation_payload(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    let operation_type = operation
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "operation batch '{}' contains a malformed operation",
                batch.id
            )
        })?;

    match operation_type {
        "set_property" => validate_set_property_operation(batch, operation, context),
        "replace_document" => validate_replace_document_operation(batch, operation),
        "create_node" => validate_create_node_operation(batch, operation),
        "group_nodes" => validate_group_nodes_operation(batch, operation, context, generated_refs),
        "add_state" => validate_add_state_operation(batch, operation, context),
        "add_timeline" => validate_add_timeline_operation(batch, operation, context),
        "add_keyframe" => {
            validate_add_keyframe_operation(batch, operation, context, generated_refs)
        }
        "bind_property" => {
            validate_bind_property_operation(batch, operation, context, generated_refs)
        }
        "emit_event" => validate_emit_event_operation(batch, operation, context, generated_refs),
        other => Err(format!(
            "operation batch '{}' contains unsupported operation type '{}'",
            batch.id, other
        )),
    }
}

pub fn validate_set_property_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let target_id = required_string_field(batch, operation, "targetId")?;
    if !context.has_node_id(target_id) {
        return Err(format!(
            "operation batch '{}' targets unknown node id '{}'",
            batch.id, target_id
        ));
    }

    let property = required_string_field(batch, operation, "property")?;
    let value = operation.get("value").ok_or_else(|| {
        format!(
            "operation batch '{}' set_property operation needs a value",
            batch.id
        )
    })?;
    validate_set_property_value(batch, property, value)?;
    if let Some(previous_value) = operation.get("previousValue") {
        validate_set_property_value(batch, property, previous_value)?;
    }
    Ok(())
}

pub fn validate_set_property_value(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
) -> Result<(), String> {
    match property {
        "style.fill" | "style.stroke" => {
            if value.is_null() || value.as_str().is_some() {
                Ok(())
            } else {
                Err(format!(
                    "operation batch '{}' has invalid value for property '{}'",
                    batch.id, property
                ))
            }
        }
        "style.opacity" => validate_finite_number_range(batch, property, value, 0.0, 1.0),
        "style.stroke_width" => validate_finite_number_range(batch, property, value, 0.0, f64::MAX),
        "transform.translate_x" | "transform.translate_y" | "transform.rotate" => {
            validate_finite_number(batch, property, value)
        }
        "transform.scale_x" | "transform.scale_y" => {
            validate_finite_number_range(batch, property, value, f64::MIN_POSITIVE, f64::MAX)
        }
        _ => Err(format!(
            "operation batch '{}' uses unsupported set_property path '{}'",
            batch.id, property
        )),
    }
}

pub fn validate_replace_document_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
) -> Result<(), String> {
    let next_document = operation.get("nextDocument").ok_or_else(|| {
        format!(
            "operation batch '{}' replace_document operation needs nextDocument",
            batch.id
        )
    })?;
    validate_document_value(batch, next_document, "nextDocument")?;

    if let Some(previous_document) = operation.get("previousDocument") {
        if !previous_document.is_null() {
            validate_document_value(batch, previous_document, "previousDocument")?;
        }
    }
    Ok(())
}

pub fn validate_document_value(
    batch: &OperationBatchRecord,
    value: &Value,
    field: &str,
) -> Result<(), String> {
    let document =
        serde_json::from_value::<strut_core::Document>(value.clone()).map_err(|error| {
            format!(
                "operation batch '{}' has invalid replacement document in {field}: {error}",
                batch.id
            )
        })?;
    strut_format::validate_document(&document).map_err(|error| {
        format!(
            "operation batch '{}' replacement document in {field} failed validation: {error}",
            batch.id
        )
    })
}

pub fn validate_create_node_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
) -> Result<(), String> {
    let id = required_string_field(batch, operation, "id")?;
    let name = required_string_field(batch, operation, "name")?;
    let kind = required_string_field(batch, operation, "kind")?;
    if id.trim().is_empty() || name.trim().is_empty() {
        return Err(format!(
            "operation batch '{}' create_node operation needs stable id and name",
            batch.id
        ));
    }
    if !matches!(
        kind,
        "group" | "rect" | "rectangle" | "ellipse" | "path" | "text"
    ) {
        return Err(format!(
            "operation batch '{}' create_node operation has unsupported kind '{}'",
            batch.id, kind
        ));
    }
    let geometry = operation.get("geometry").ok_or_else(|| {
        format!(
            "operation batch '{}' create_node operation needs geometry",
            batch.id
        )
    })?;
    let geometry = serde_json::from_value::<PlanGeometry>(geometry.clone()).map_err(|error| {
        format!(
            "operation batch '{}' create_node operation has malformed geometry: {error}",
            batch.id
        )
    })?;
    validate_plan_geometry(id, &geometry)
}

pub fn validate_group_nodes_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    required_string_field(batch, operation, "id")?;
    required_string_field(batch, operation, "name")?;
    let children = operation
        .get("children")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("operation batch '{}' group_nodes needs children", batch.id))?;
    if children.is_empty() {
        return Err(format!(
            "operation batch '{}' group_nodes has no children",
            batch.id
        ));
    }
    for child in children {
        let Some(child) = child.as_str() else {
            return Err(format!(
                "operation batch '{}' group_nodes contains a malformed child id",
                batch.id
            ));
        };
        if !context.has_node_ref(child) && !generated_refs.has_node_ref(child) {
            return Err(format!(
                "operation batch '{}' group_nodes references unknown child '{}'",
                batch.id, child
            ));
        }
    }
    Ok(())
}

pub fn validate_add_state_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let state = required_string_field(batch, operation, "state")?;
    if !context.states.contains(state) {
        return Err(format!(
            "operation batch '{}' add_state references unknown state '{}'",
            batch.id, state
        ));
    }
    Ok(())
}

pub fn validate_add_timeline_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    required_string_field(batch, operation, "id")?;
    required_string_field(batch, operation, "name")?;
    let duration = operation
        .get("duration_ms")
        .and_then(Value::as_u64)
        .or_else(|| operation.get("durationMs").and_then(Value::as_u64))
        .ok_or_else(|| {
            format!(
                "operation batch '{}' add_timeline needs a positive duration",
                batch.id
            )
        })?;
    if duration == 0 {
        return Err(format!(
            "operation batch '{}' add_timeline needs a positive duration",
            batch.id
        ));
    }
    if let Some(state) = operation.get("state").and_then(Value::as_str) {
        if !context.states.contains(state) {
            return Err(format!(
                "operation batch '{}' add_timeline references unknown state '{}'",
                batch.id, state
            ));
        }
    }
    Ok(())
}

pub fn validate_add_keyframe_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    let timeline = required_string_field(batch, operation, "timeline")?;
    if !context.timeline_refs.contains(timeline) && !generated_refs.has_timeline_ref(timeline) {
        return Err(format!(
            "operation batch '{}' add_keyframe references unknown timeline '{}'",
            batch.id, timeline
        ));
    }
    let target = required_string_field(batch, operation, "target")?;
    if !context.has_node_ref(target) && !generated_refs.has_node_ref(target) {
        return Err(format!(
            "operation batch '{}' add_keyframe targets unknown node '{}'",
            batch.id, target
        ));
    }
    let property = required_string_field(batch, operation, "property")?;
    if !allowed_timeline_property(property) {
        return Err(format!(
            "operation batch '{}' add_keyframe uses unsupported property '{}'",
            batch.id, property
        ));
    }
    if operation.get("time_ms").and_then(Value::as_u64).is_none()
        && operation.get("timeMs").and_then(Value::as_u64).is_none()
    {
        return Err(format!(
            "operation batch '{}' add_keyframe needs time_ms",
            batch.id
        ));
    }
    let value = operation.get("value").ok_or_else(|| {
        format!(
            "operation batch '{}' add_keyframe needs a numeric value",
            batch.id
        )
    })?;
    validate_finite_number(batch, property, value)
}

pub fn validate_bind_property_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    required_string_field(batch, operation, "name")?;
    let target = required_string_field(batch, operation, "target")?;
    if !context.has_node_ref(target) && !generated_refs.has_node_ref(target) {
        return Err(format!(
            "operation batch '{}' bind_property targets unknown node '{}'",
            batch.id, target
        ));
    }
    let property = required_string_field(batch, operation, "property")?;
    if !allowed_edit_property(property) {
        return Err(format!(
            "operation batch '{}' bind_property uses unsupported property '{}'",
            batch.id, property
        ));
    }
    Ok(())
}

pub fn validate_emit_event_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    let name = required_string_field(batch, operation, "name")?;
    if !context.events.contains(name) && !generated_refs.has_event_ref(name) {
        return Err(format!(
            "operation batch '{}' emit_event references unknown event '{}'",
            batch.id, name
        ));
    }
    Ok(())
}

pub fn required_string_field<'a>(
    batch: &OperationBatchRecord,
    operation: &'a Value,
    field: &str,
) -> Result<&'a str, String> {
    operation
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "operation batch '{}' operation needs string field '{}'",
                batch.id, field
            )
        })
}

pub fn validate_finite_number(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
) -> Result<(), String> {
    let Some(number) = value.as_f64() else {
        return Err(format!(
            "operation batch '{}' has non-numeric value for property '{}'",
            batch.id, property
        ));
    };
    if number.is_finite() {
        Ok(())
    } else {
        Err(format!(
            "operation batch '{}' has non-finite value for property '{}'",
            batch.id, property
        ))
    }
}

pub fn validate_finite_number_range(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
    minimum: f64,
    maximum: f64,
) -> Result<(), String> {
    validate_finite_number(batch, property, value)?;
    let number = value.as_f64().unwrap_or_default();
    if number >= minimum && number <= maximum {
        Ok(())
    } else {
        Err(format!(
            "operation batch '{}' has out-of-range value for property '{}'",
            batch.id, property
        ))
    }
}

pub fn validation_result(result: Result<(), String>) -> OperationValidationResult {
    let timestamp = timestamp_label();
    match result {
        Ok(()) => OperationValidationResult {
            ok: true,
            message: "document validated by Rust format rules".to_string(),
            validator: "strut-studio-rust".to_string(),
            validated_at: timestamp,
        },
        Err(error) => OperationValidationResult {
            ok: false,
            message: error,
            validator: "strut-studio-rust".to_string(),
            validated_at: timestamp,
        },
    }
}



pub fn validate_plan_geometry(_id: &str, _geometry: &models::PlanGeometry) -> Result<(), String> { Ok(()) }
pub fn allowed_timeline_property(_prop: &str) -> bool { true }
pub fn allowed_edit_property(_prop: &str) -> bool { true }
