struct GenerationPlan {
    id: Option<String>,
    name: String,
    subject: SubjectPlan,
    #[serde(default)]
    parts: Vec<SemanticPartPlan>,
    #[serde(default, alias = "motion_roles")]
    motion_roles: Vec<MotionRolePlan>,
    #[serde(default)]
    states: Vec<String>,
    #[serde(default)]
    timelines: Vec<TimelinePlan>,
    #[serde(default)]
    editability: EditabilityPlan,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubjectPlan {
    classification: String,
    label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPartPlan {
    id: String,
    name: String,
    role: String,
    geometry: PlanGeometry,
    #[serde(default)]
    style: PlanStyle,
    #[serde(default, alias = "motion_roles")]
    motion_roles: Vec<String>,
    #[serde(default)]
    constraints: EditabilityConstraint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanGeometry {
    kind: String,
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
    rx: Option<f64>,
    ry: Option<f64>,
    cx: Option<f64>,
    cy: Option<f64>,
    d: Option<String>,
    value: Option<String>,
    size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanStyle {
    fill: Option<String>,
    stroke: Option<String>,
    #[serde(alias = "stroke_width")]
    stroke_width: Option<f64>,
    opacity: Option<f64>,
}

impl Default for PlanStyle {
    fn default() -> Self {
        Self {
            fill: Some("#f6f0df".to_string()),
            stroke: Some("#25221d".to_string()),
            stroke_width: Some(5.0),
            opacity: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditabilityConstraint {
    #[serde(default = "default_editable")]
    editable: bool,
    #[serde(default, alias = "allowed_properties")]
    allowed_properties: Vec<String>,
}

impl Default for EditabilityConstraint {
    fn default() -> Self {
        Self {
            editable: true,
            allowed_properties: Vec::new(),
        }
    }
}

fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditabilityPlan {
    #[serde(default, alias = "editable_parts")]
    editable_parts: Vec<String>,
    #[serde(default, alias = "locked_parts")]
    locked_parts: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    notes: Vec<String>,
}