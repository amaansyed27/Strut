struct TimelinePlan {
    id: String,
    name: String,
    state: Option<String>,
    #[serde(alias = "duration_ms")]
    duration_ms: u32,
    #[serde(default)]
    tracks: Vec<TimelineTrackPlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineTrackPlan {
    target: String,
    property: String,
    #[serde(default)]
    keyframes: Vec<KeyframePlan>,
}