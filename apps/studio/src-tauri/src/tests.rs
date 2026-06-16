    use super::*;
    use serde_json::{json, Value};
    use std::path::{Path, PathBuf};
    use std::process::Command;    fn collect_layer_names<'a>(nodes: &'a [strut_core::Node], names: &mut Vec<&'a str>) {
        for node in nodes {
            names.push(node.name.as_str());
            collect_layer_names(&node.children, names);
        }
    }

    fn count_document_nodes(document: &strut_core::Document) -> usize {
        flatten_document_nodes(document).len()
    }

    struct TestSummary {
        subject_classification: String,
        subject_label: String,
        timeline_names: Vec<String>,
    }

    struct TestPlannedDocument {
        document: strut_core::Document,
        summary: TestSummary,
        operation_count: usize,
    }

    fn parse_test_planned_document(text: &str) -> Result<TestPlannedDocument, String> {
        let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let plan_val = value.get("plan").unwrap_or(&value);
        
        let classification = plan_val.get("subject")
            .and_then(|s| s.get("classification"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
            
        let label = plan_val.get("subject")
            .and_then(|s| s.get("label"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
            
        let mut timeline_names = Vec::new();
        if let Some(timelines) = plan_val.get("timelines").and_then(Value::as_array) {
            for tl in timelines {
                if let Some(name) = tl.get("name").and_then(Value::as_str) {
                    timeline_names.push(name.to_string());
                }
            }
        }

        let document = document_from_generation_plan_text(text)?;
        
        let mut operation_count = count_document_nodes(&document);
        operation_count += document.timelines.len();
        for tl in &document.timelines {
            operation_count += tl.tracks.len();
            for trk in &tl.tracks {
                operation_count += trk.keyframes.len();
            }
        }
        for sm in &document.state_machines {
            operation_count += sm.states.len();
        }
        operation_count += document.bindings.len();
        
        Ok(TestPlannedDocument {
            document,
            summary: TestSummary {
                subject_classification: classification,
                subject_label: label,
                timeline_names,
            },
            operation_count,
        })
    }

    fn phase3_part(id: &str, name: &str, role: &str, geometry: Value) -> Value {
        json!({
            "id": id,
            "name": name,
            "role": role,
            "geometry": geometry,
            "style": {"fill": "#f6f0df", "stroke": "#25221d", "strokeWidth": 5, "opacity": 1},
            "motionRoles": ["primary"],
            "constraints": {"editable": true, "allowedProperties": ["fill", "translation.x", "translation.y", "rotation", "opacity"]}
        })
    }

    fn phase3_plan_text(
        classification: &str,
        label: &str,
        parts: Vec<Value>,
        state: &str,
        target: &str,
    ) -> String {
        json!({
            "plan": {
                "id": format!("{classification}-plan"),
                "name": format!("{label} Motion"),
                "subject": {"classification": classification, "label": label},
                "parts": parts,
                "motionRoles": [{"id": "primary", "purpose": "calm subject motion", "partRefs": [target]}],
                "states": ["idle", state],
                "timelines": [{
                    "id": format!("{state}-timeline"),
                    "name": state,
                    "state": state,
                    "durationMs": 1200,
                    "tracks": [{
                        "target": target,
                        "property": "translation.y",
                        "keyframes": [
                            {"timeMs": 0, "value": 0, "easing": "ease_in_out"},
                            {"timeMs": 600, "value": -8, "easing": "ease_out"},
                            {"timeMs": 1200, "value": 0, "easing": "ease_in_out"}
                        ]
                    }]
                }],
                "editability": {"editableParts": [target], "lockedParts": [], "notes": ["fixture"]}
            },
            "operations": []
        })
        .to_string()
    }

    fn semantic_layer_names(document: &strut_core::Document) -> Vec<String> {
        let mut names = Vec::new();
        collect_layer_names(&document.artboards[0].nodes, &mut names);
        names.into_iter().map(str::to_string).collect()
    }

    fn sprite_python_fixture(name: &str) -> &'static str {
        match name {
            "dice" => include_str!("../../../../packages/strut-python/fixtures/dice.plan.json"),
            "logo" => include_str!("../../../../packages/strut-python/fixtures/logo.plan.json"),
            "loader" => include_str!("../../../../packages/strut-python/fixtures/loader.plan.json"),
            "mascot" => include_str!("../../../../packages/strut-python/fixtures/mascot.plan.json"),
            "ui" => include_str!("../../../../packages/strut-python/fixtures/ui.plan.json"),
            "icon" => include_str!("../../../../packages/strut-python/fixtures/icon.plan.json"),
            _ => panic!("unknown sprite-python fixture"),
        }
    }

    #[test]
    fn sprite_python_fixtures_validate_through_generation_plan_path() {
        for (fixture, classification, required_layers, forbidden_layers) in [
            (
                "dice",
                "dice",
                vec!["DieBody", "FrontFace", "Pips"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "logo",
                "logo",
                vec!["PrimaryMark", "Wordmark", "AccentStroke"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "loader",
                "loader",
                vec!["Track", "ActiveSegment", "ProgressSweep"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "mascot",
                "mascot",
                vec![
                    "Body",
                    "Head",
                    "LeftEye",
                    "RightEye",
                    "LeftWing",
                    "RightWing",
                ],
                vec![],
            ),
            (
                "ui",
                "ui",
                vec!["ButtonSurface", "ButtonLabel", "FocusRing"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "icon",
                "badge",
                vec!["BadgePlate", "InnerShield", "StatusDot"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
        ] {
            let planned = parse_test_planned_document(sprite_python_fixture(fixture))
                .expect("sprite-python fixture should validate");
            let names = semantic_layer_names(&planned.document);

            assert_eq!(planned.summary.subject_classification, classification);
            assert!(planned.operation_count >= 10);
            for required in required_layers {
                assert!(
                    names.iter().any(|name| name == required),
                    "{fixture} missing expected layer {required}"
                );
            }
            for forbidden in forbidden_layers {
                assert!(
                    names.iter().all(|name| name != forbidden),
                    "{fixture} unexpectedly emitted mascot-only layer {forbidden}"
                );
            }
        }
    }

    #[test]
    fn sprite_python_custom_generation_validates_through_rust() {
        let package_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../packages/strut-python")
            .canonicalize()
            .unwrap();
        let output = Command::new("python")
            .arg("-m")
            .arg("strut_python.cli")
            .arg("custom")
            .arg("--instruction")
            .arg("animate a twitter bird taking flight")
            .arg("--json")
            .current_dir(&package_dir)
            .env("PYTHONPATH", package_dir.join("src"))
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let planned = parse_test_planned_document(&stdout).unwrap();
        let names = semantic_layer_names(&planned.document);

        assert_eq!(planned.summary.subject_classification, "bird_icon");
        assert_eq!(planned.summary.subject_label, "Twitter Bird Taking Flight");
        assert!(names
            .iter()
            .any(|name| name == "Twitter Bird Taking Flight Wing"));
        assert!(names.iter().all(
            |name| !["Body", "Head", "Eyes", "Arms", "Face", "Smile"].contains(&name.as_str())
        ));
    }

    fn temp_project_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("strut-{name}-{}", unix_timestamp()))
    }

    fn write_project_manifest(root: &Path, name: &str, main_scene: &str) {
        fs::write(
            root.join(PROJECT_MANIFEST_FILE),
            serde_json::to_string_pretty(&json!({
                "name": name,
                "mainScene": main_scene
            }))
            .expect("manifest json"),
        )
        .expect("manifest write");
    }

    fn write_project_scene(root: &Path, scene: &str, document: &strut_core::Document) {
        let scene_path = root.join(scene);
        fs::create_dir_all(scene_path.parent().expect("scene parent")).expect("scene dir");
        strut_format::write_strut_file(
            scene_path,
            &strut_format::StrutPackage::current(document.clone()),
        )
        .expect("scene write");
    }

    fn valid_test_batch(document: &strut_core::Document) -> OperationBatchRecord {
        OperationBatchRecord {
            id: "batch-manual-fill".to_string(),
            source_type: "manual".to_string(),
            status: "applied".to_string(),
            validation_result: OperationValidationResult {
                ok: true,
                message: "validated test operation".to_string(),
                validator: "strut-studio-rust".to_string(),
                validated_at: "1".to_string(),
            },
            document_revision_id: document_revision_id(document),
            previous_document_revision_id: Some("rev-before".to_string()),
            prompt: Some("make the button warmer".to_string()),
            source_metadata: Some(json!({"chatMessageId": "message-1"})),
            operations: vec![json!({
                "id": "op-fill",
                "type": "set_property",
                "targetId": document.artboards[0].nodes[0].id.to_string(),
                "property": "style.fill",
                "value": "#d8f5e3"
            })],
            created_at: "1".to_string(),
            updated_at: "2".to_string(),
            applied_at: Some("2".to_string()),
            rejected_at: None,
        }
    }

    fn generated_reference_test_batch(
        document: &strut_core::Document,
        operations: Vec<Value>,
    ) -> OperationBatchRecord {
        let mut batch = valid_test_batch(document);
        batch.id = "batch-generated-refs".to_string();
        batch.source_type = "sprite-python".to_string();
        batch.prompt = Some("generated reference validation".to_string());
        batch.source_metadata = Some(json!({"test": "generated-refs"}));
        batch.operations = operations;
        batch
    }

    fn generated_rect_node(id: &str) -> Value {
        json!({
            "id": id,
            "type": "create_node",
            "name": id,
            "kind": "rect",
            "geometry": {"kind": "rect", "x": 10, "y": 10, "width": 24, "height": 24, "rx": 4},
            "style": {"fill": "#ffffff", "stroke": "#111827", "strokeWidth": 2, "opacity": 1}
        })
    }

    fn generated_timeline(id: &str, name: &str) -> Value {
        json!({
            "id": id,
            "type": "add_timeline",
            "name": name,
            "state": "hover",
            "duration_ms": 180
        })
    }

    fn generated_keyframe(timeline: &str, target: &str) -> Value {
        json!({
            "id": "op-generated-keyframe",
            "type": "add_keyframe",
            "timeline": timeline,
            "target": target,
            "property": "translation.y",
            "time_ms": 0,
            "value": 0
        })
    }

    fn flatten_document_nodes(document: &strut_core::Document) -> Vec<strut_core::Node> {
        fn push_node(nodes: &mut Vec<strut_core::Node>, node: &strut_core::Node) {
            nodes.push(node.clone());
            for child in &node.children {
                push_node(nodes, child);
            }
        }
        let mut nodes = Vec::new();
        for artboard in &document.artboards {
            for node in &artboard.nodes {
                push_node(&mut nodes, node);
            }
        }
        nodes
    }

    fn unrelated_operation_id(id: &str) -> Value {
        json!({
            "id": id,
            "type": "emit_event",
            "name": "submit",
            "description": "unrelated operation id must not become a node or timeline ref"
        })
    }

    #[test]
    fn project_snapshot_saves_loads_validated_scene_and_operation_batches() {
        let root = temp_project_root("snapshot");
        let document = strut_core::Document::sample_login_button();
        let batch = valid_test_batch(&document);
        let selection = PersistedSelectionState {
            active_state: "hover".to_string(),
            selected_node_id: Some(document.artboards[0].nodes[0].id.to_string()),
            layer_ui: json!({"selected": {"visible": true, "locked": false}}),
        };

        let saved = save_project_snapshot(
            root.display().to_string(),
            "Snapshot Project".to_string(),
            document.clone(),
            vec![batch.clone()],
            Some(selection.clone()),
        )
        .expect("snapshot should save");
        assert!(PathBuf::from(&saved.project.path)
            .join(MAIN_SCENE_FILE)
            .exists());
        assert!(PathBuf::from(&saved.project.path)
            .join(OPERATION_BATCHES_FILE)
            .exists());

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("snapshot should load");
        assert_eq!(loaded.document.name, document.name);
        assert_eq!(loaded.operation_batches, vec![batch]);
        assert_eq!(
            loaded.selection.expect("selection").selected_node_id,
            selection.selected_node_id
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_animation_files_are_saved_listed_loaded_and_deleted() {
        let root = temp_project_root("animations");
        let document = strut_core::Document::sample_login_button();
        let batch = valid_test_batch(&document);
        create_project(
            "Animation Project".to_string(),
            root.parent()
                .expect("temp parent")
                .display()
                .to_string(),
        )
        .expect("project can be created");
        let project_root = root
            .parent()
            .expect("temp parent")
            .join("Animation Project");

        let saved = save_project_animation(
            project_root.display().to_string(),
            "Animation Project".to_string(),
            "chat-1".to_string(),
            "Rolling Dice".to_string(),
            document.clone(),
            vec![batch.clone()],
            Some(PersistedSelectionState {
                active_state: "idle".to_string(),
                selected_node_id: None,
                layer_ui: json!({}),
            }),
        )
        .expect("animation should save");

        assert_eq!(saved.name, "Rolling Dice");
        assert!(project_root.join(&saved.scene).exists());

        let loaded = load_project_snapshot(project_root.display().to_string())
            .expect("project should load with animations");
        assert_eq!(loaded.animations.len(), 1);
        assert_eq!(loaded.animations[0].id, saved.id);
        assert_eq!(loaded.animations[0].document.name, document.name);
        assert_eq!(loaded.animations[0].operation_batches, vec![batch]);

        delete_project_animation(project_root.display().to_string(), saved.id.clone())
            .expect("animation should delete");
        let reloaded = load_project_snapshot(project_root.display().to_string())
            .expect("project should reload after deletion");
        assert!(reloaded.animations.is_empty());
        assert!(!project_root.join(&saved.scene).exists());

        let _ = fs::remove_dir_all(project_root);
    }

    #[test]
    fn project_animation_save_replaces_same_chat_and_name() {
        let root = temp_project_root("animation-dedupe");
        let document = strut_core::Document::sample_login_button();
        let batch = valid_test_batch(&document);
        create_project(
            "Animation Dedupe".to_string(),
            root.parent()
                .expect("temp parent")
                .display()
                .to_string(),
        )
        .expect("project can be created");
        let project_root = root
            .parent()
            .expect("temp parent")
            .join("Animation Dedupe");

        let first = save_project_animation(
            project_root.display().to_string(),
            "Animation Dedupe".to_string(),
            "chat-1".to_string(),
            "Rolling Dice".to_string(),
            document.clone(),
            vec![batch.clone()],
            None,
        )
        .expect("first animation should save");
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let second = save_project_animation(
            project_root.display().to_string(),
            "Animation Dedupe".to_string(),
            "chat-1".to_string(),
            "Rolling Dice".to_string(),
            document.clone(),
            vec![batch],
            None,
        )
        .expect("second animation should replace first");

        let loaded = load_project_snapshot(project_root.display().to_string())
            .expect("project should load with deduped animation");
        assert_eq!(loaded.animations.len(), 1);
        assert_eq!(loaded.animations[0].id, second.id);
        assert!(!project_root.join(&first.scene).exists());

        let _ = fs::remove_dir_all(project_root);
    }

    #[test]
    fn style_safety_keeps_foreground_visible_when_provider_colors_collide() {
        let text = json!({
            "kind": "document_created",
            "message": "Created a dice roll.",
            "document": {
                "plan": {
                    "id": "bad_dice_colors",
                    "name": "Bad Dice Colors",
                    "subject": {"classification": "dice", "label": "Rolling Dice"},
                    "parts": [
                        {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": 380, "y": 170, "width": 180, "height": 180, "rx": 24}, "style": {"fill": "#000000", "stroke": "#000000", "stroke_width": 3}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                        {"id": "PipCenter", "name": "Center Pip", "role": "detail", "geometry": {"kind": "ellipse", "cx": 470, "cy": 260, "rx": 12, "ry": 12}, "style": {"fill": "#000000", "opacity": 1}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipTopLeft", "name": "Top Left Pip", "role": "detail", "geometry": {"kind": "ellipse", "cx": 430, "cy": 220, "rx": 12, "ry": 12}, "style": {"fill": "#000000", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipBottomRight", "name": "Bottom Right Pip", "role": "detail", "geometry": {"kind": "ellipse", "cx": 510, "cy": 300, "rx": 12, "ry": 12}, "style": {"fill": "#000000", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "Shadow", "name": "Settle Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 470, "cy": 370, "rx": 90, "ry": 16}, "style": {"fill": "#000000", "opacity": 0.18}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}}
                    ],
                    "motion_roles": [],
                    "states": ["idle", "rolling", "face1"],
                    "timelines": [{"id": "roll_1", "name": "Roll to face 1", "state": "face1", "duration_ms": 1000, "tracks": []}],
                    "editability": {"editable_parts": ["DieBody"], "locked_parts": [], "notes": []}
                },
                "operations": []
            }
        })
        .to_string();

        let planned = parse_test_planned_document(&text).expect("dice plan compiles");
        let nodes = flatten_document_nodes(&planned.document);
        let body = nodes.iter().find(|node| node.name == "Die Body").expect("body node");
        let pip = nodes.iter().find(|node| node.name == "Center Pip").expect("pip node");

        assert_eq!(body.style.fill.as_deref(), Some("#000000"));
        assert_eq!(pip.style.fill.as_deref(), Some("#f8fafc"));
    }

    #[test]
    fn operation_payload_validation_rejects_malformed_targets_properties_and_empty_batches() {
        let document = strut_core::Document::sample_login_button();
        let mut unsupported_type = valid_test_batch(&document);
        unsupported_type.operations = vec![json!({"id": "op-delete", "type": "delete_node"})];
        let error = validate_operation_batches(&[unsupported_type], &document)
            .expect_err("unsupported operation type rejects");
        assert!(error.contains("unsupported operation type"));

        let mut missing_target = valid_test_batch(&document);
        missing_target.operations[0]["targetId"] = json!("00000000-0000-0000-0000-000000009999");
        let error = validate_operation_batches(&[missing_target], &document)
            .expect_err("missing target rejects");
        assert!(error.contains("unknown node id"));

        let mut unsupported_property = valid_test_batch(&document);
        unsupported_property.operations[0]["property"] = json!("style.__proto__");
        let error = validate_operation_batches(&[unsupported_property], &document)
            .expect_err("unsafe property rejects");
        assert!(error.contains("unsupported set_property path"));

        let mut invalid_value = valid_test_batch(&document);
        invalid_value.operations[0]["value"] = json!({"unexpected": "object"});
        let error = validate_operation_batches(&[invalid_value], &document)
            .expect_err("invalid property value rejects");
        assert!(error.contains("invalid value"));

        let mut empty_applied = valid_test_batch(&document);
        empty_applied.operations = Vec::new();
        let error = validate_operation_batches(&[empty_applied], &document)
            .expect_err("empty applied batch rejects");
        assert!(error.contains("no meaningful operations"));
    }

    #[test]
    fn generated_references_reject_unrelated_operation_ids() {
        let document = strut_core::Document::sample_login_button();
        let existing_node = document.artboards[0].nodes[0].id.to_string();

        let add_keyframe_target = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                generated_timeline("GeneratedTimeline", "generated-timeline"),
                generated_keyframe("GeneratedTimeline", "FakeGeneratedNode"),
            ],
        );
        let error = validate_operation_batches(&[add_keyframe_target], &document)
            .expect_err("unrelated operation id must not become a keyframe target");
        assert!(error.contains("targets unknown node 'FakeGeneratedNode'"));

        let add_keyframe_timeline = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedTimeline"),
                generated_keyframe("FakeGeneratedTimeline", &existing_node),
            ],
        );
        let error = validate_operation_batches(&[add_keyframe_timeline], &document)
            .expect_err("unrelated operation id must not become a keyframe timeline");
        assert!(error.contains("unknown timeline 'FakeGeneratedTimeline'"));

        let bind_property_target = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                json!({
                    "id": "op-bind-fake",
                    "type": "bind_property",
                    "name": "fake_binding",
                    "target": "FakeGeneratedNode",
                    "property": "fill"
                }),
            ],
        );
        let error = validate_operation_batches(&[bind_property_target], &document)
            .expect_err("unrelated operation id must not become a bind target");
        assert!(error.contains("targets unknown node 'FakeGeneratedNode'"));

        let group_nodes_child = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                json!({
                    "id": "op-group-fake",
                    "type": "group_nodes",
                    "name": "Fake Group",
                    "children": ["FakeGeneratedNode"]
                }),
            ],
        );
        let error = validate_operation_batches(&[group_nodes_child], &document)
            .expect_err("unrelated operation id must not become a group child");
        assert!(error.contains("unknown child 'FakeGeneratedNode'"));
    }

    #[test]
    fn generated_references_accept_create_node_and_add_timeline_refs() {
        let document = strut_core::Document::sample_login_button();
        let batch = generated_reference_test_batch(
            &document,
            vec![
                generated_rect_node("GeneratedNode"),
                generated_timeline("GeneratedTimeline", "Generated Timeline"),
                json!({
                    "id": "op-group-generated",
                    "type": "group_nodes",
                    "name": "Generated Group",
                    "children": ["GeneratedNode"]
                }),
                generated_keyframe("GeneratedTimeline", "GeneratedNode"),
                generated_keyframe("Generated Timeline", "GeneratedNode"),
                json!({
                    "id": "op-bind-generated",
                    "type": "bind_property",
                    "name": "generated_fill",
                    "target": "GeneratedNode",
                    "property": "fill"
                }),
            ],
        );

        validate_operation_batches(&[batch], &document)
            .expect("create_node ids and add_timeline ids/names are valid generated refs");
    }

    #[test]
    fn replacement_operation_documents_are_validated_before_persistence() {
        let document = strut_core::Document::sample_login_button();
        let mut invalid_document = document.clone();
        invalid_document.artboards.clear();

        let mut invalid_replacement = valid_test_batch(&document);
        invalid_replacement.operations = vec![json!({
            "id": "op-replace-invalid",
            "type": "replace_document",
            "previousDocument": document,
            "nextDocument": invalid_document
        })];
        let error = validate_operation_batches(&[invalid_replacement], &document)
            .expect_err("invalid replacement document rejects");
        assert!(error.contains("replacement document"));
        assert!(error.contains("artboard"));

        let mut valid_replacement = valid_test_batch(&strut_core::Document::sample_login_button());
        let previous_document = strut_core::Document::sample_login_button();
        let next_document = strut_core::Document::sample_minimal_bot();
        valid_replacement.operations = vec![json!({
            "id": "op-replace-valid",
            "type": "replace_document",
            "previousDocument": previous_document,
            "nextDocument": next_document
        })];
        validate_operation_batches(
            &[valid_replacement],
            &strut_core::Document::sample_minimal_bot(),
        )
        .expect("valid replacement document accepts");
    }

    #[test]
    fn sprite_python_generated_operations_persist_after_rust_payload_validation() {
        let root = temp_project_root("sprite-generated-persist");
        let validated = validate_generation_plan_batch(
            sprite_python_fixture("dice").to_string(),
            "sprite-python".to_string(),
            Some("rolling dice".to_string()),
        )
        .expect("sprite-python fixture validates");

        save_project_snapshot(
            root.display().to_string(),
            "Sprite Persist".to_string(),
            validated.document.clone(),
            vec![validated.batch.clone()],
            None,
        )
        .expect("validated sprite-python operations persist");

        let loaded = load_project_snapshot(root.display().to_string())
            .expect("persisted sprite-python project loads");
        assert_eq!(loaded.operation_batches, vec![validated.batch]);
        assert_eq!(loaded.document.name, "Rolling Dice Motion");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_documents_and_batches_are_rejected_before_persistence() {
        let root = temp_project_root("invalid");
        let mut document = strut_core::Document::sample_login_button();
        document.artboards.clear();
        let validation = validate_scene_document(document.clone());
        assert!(!validation.ok);
        assert!(validation.message.contains("artboard"));

        let error = save_project_snapshot(
            root.display().to_string(),
            "Invalid Project".to_string(),
            document,
            Vec::new(),
            None,
        )
        .expect_err("bad document should reject");
        assert!(error.contains("artboard"));

        let valid_document = strut_core::Document::sample_login_button();
        let mut batch = valid_test_batch(&valid_document);
        batch.source_type = "python".to_string();
        let error = save_project_snapshot(
            root.display().to_string(),
            "Invalid Project".to_string(),
            valid_document,
            vec![batch],
            None,
        )
        .expect_err("bad batch should reject");
        assert!(error.contains("unsupported source type"));
    }

    #[test]
    fn legacy_generated_local_state_document_json_loads_for_compatibility() {
        let root = temp_project_root("legacy");
        fs::create_dir_all(root.join("scenes")).expect("scenes dir");
        let document = strut_core::Document::sample_login_button();
        fs::write(
            root.join(LEGACY_STARTER_SCENE_FILE),
            serde_json::to_string_pretty(&document).expect("document json"),
        )
        .expect("legacy scene");
        fs::write(
            root.join(PROJECT_MANIFEST_FILE),
            serde_json::to_string_pretty(&json!({
                "name": "Legacy Project",
                "mainScene": LEGACY_STARTER_SCENE_FILE
            }))
            .expect("manifest json"),
        )
        .expect("manifest");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("legacy project loads");
        assert_eq!(loaded.document.name, "Login Button");
        assert!(loaded.operation_batches.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_rejects_absolute_main_scene_paths() {
        let root = temp_project_root("absolute-main-scene");
        fs::create_dir_all(&root).expect("root dir");
        let absolute_scene = root.join("outside.strut");
        write_project_manifest(&root, "Bad Manifest", &absolute_scene.display().to_string());

        let error =
            load_project_snapshot(root.display().to_string()).expect_err("absolute path rejects");
        assert!(error.contains("mainScene path must be relative"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_rejects_traversal_main_scene_paths() {
        let root = temp_project_root("traversal-main-scene");
        fs::create_dir_all(&root).expect("root dir");
        write_project_manifest(&root, "Bad Manifest", "../outside.strut");

        let error =
            load_project_snapshot(root.display().to_string()).expect_err("traversal path rejects");
        assert!(error.contains("mainScene path must stay inside"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_accepts_valid_relative_main_scene_paths() {
        let root = temp_project_root("relative-main-scene");
        let document = strut_core::Document::sample_login_button();
        write_project_scene(&root, "scenes/custom.strut", &document);
        write_project_manifest(&root, "Custom Scene", "scenes/custom.strut");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("relative scene loads");
        assert_eq!(loaded.document.name, "Login Button");
        assert_eq!(loaded.main_scene, "scenes/custom.strut");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_manifest_scene_still_falls_back_to_legacy_scene() {
        let root = temp_project_root("missing-main-scene-fallback");
        fs::create_dir_all(root.join("scenes")).expect("scenes dir");
        let document = strut_core::Document::sample_login_button();
        fs::write(
            root.join(LEGACY_STARTER_SCENE_FILE),
            serde_json::to_string_pretty(&document).expect("document json"),
        )
        .expect("legacy scene");
        write_project_manifest(&root, "Legacy Fallback", "scenes/missing.strut");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("legacy fallback loads");
        assert_eq!(loaded.document.name, "Login Button");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sprite_python_batch_persists_only_after_rust_validation() {
        let validated = validate_generation_plan_batch(
            sprite_python_fixture("dice").to_string(),
            "sprite-python".to_string(),
            Some("rolling dice".to_string()),
        )
        .expect("sprite-python fixture validates");

        assert_eq!(validated.batch.source_type, "sprite-python");
        assert!(validated.batch.validation_result.ok);
        assert_eq!(validated.batch.status, "applied");
        assert!(validated.batch.operations.len() >= 10);
        assert!(validated
            .batch
            .operations
            .iter()
            .any(|operation| operation.get("type").and_then(Value::as_str) == Some("create_node")));

        let bad = validate_generation_plan_batch(
            json!({
                "plan": {
                    "id": "bad-logo",
                    "name": "Bad Logo",
                    "subject": {"classification": "logo", "label": "Logo"},
                    "parts": [
                        phase3_part("Body", "Body", "body", json!({"kind":"ellipse","cx":480,"cy":270,"rx":80,"ry":80})),
                        phase3_part("Head", "Head", "head", json!({"kind":"ellipse","cx":480,"cy":190,"rx":60,"ry":50})),
                        phase3_part("Eyes", "Eyes", "eyes", json!({"kind":"path","d":"M460 190 L470 190"})),
                        phase3_part("PrimaryMark", "PrimaryMark", "mark", json!({"kind":"path","d":"M420 240 L540 240"})),
                        phase3_part("AccentStroke", "AccentStroke", "accent", json!({"kind":"path","d":"M420 270 L540 270"}))
                    ],
                    "motionRoles": [{"id": "primary", "purpose": "bad mascot anatomy", "partRefs": ["PrimaryMark"]}],
                    "states": ["idle", "reveal"],
                    "timelines": [{
                        "id": "reveal",
                        "name": "reveal",
                        "state": "reveal",
                        "durationMs": 1000,
                        "tracks": [{
                            "target": "PrimaryMark",
                            "property": "opacity",
                            "keyframes": [
                                {"timeMs": 0, "value": 0, "easing": "linear"},
                                {"timeMs": 1000, "value": 1, "easing": "linear"}
                            ]
                        }]
                    }],
                    "editability": {"editableParts": ["PrimaryMark"], "lockedParts": [], "notes": []}
                },
                "operations": []
            })
            .to_string(),
            "sprite-python".to_string(),
            None,
        )
        .expect_err("invalid sprite-python batch rejects");
        assert!(bad.contains("mascot-only anatomy"));
    }

    #[test]
    fn local_agent_catalog_includes_requested_providers() {
        let adapters = local_agent_adapters();
        let ids = adapters
            .iter()
            .map(|adapter| adapter.id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"codex"));
        assert!(!ids.contains(&"strut-sprite"));
        assert!(ids.contains(&"gemini-cli"));
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"copilot-cli"));
        assert!(ids.contains(&"ollama"));
        assert!(ids.contains(&"opencode"));
        assert!(ids.contains(&"cursor-agent"));
        assert!(ids.contains(&"qwen"));
        assert!(ids.contains(&"qoder"));
        assert!(adapters
            .iter()
            .all(|adapter| adapter.kind != "local-engine"));
    }

    #[test]
    fn parses_full_document_from_model_text() {
        let document_json = serde_json::to_string(&strut_core::Document::sample_owl_mascot())
            .expect("document json");
        let document =
            parse_generated_document(&format!("Here is JSON: {{\"document\":{document_json}}}"))
                .expect("document should parse");

        assert_eq!(document.name, "Owl Mascot");
        assert_eq!(document.artboards[0].name, "OwlMascot");
    }

    #[test]
    fn rejects_legacy_preset_spec_from_model_text() {
        let error = parse_generated_document(
            r##"{"variant":"owl-guide","name":"Owl Mascot","accent":"#78d64b","shell":"#8ee15a"}"##,
        )
        .expect_err("legacy spec should be rejected");

        assert!(error.contains("old preset spec format"));
    }

    #[test]
    fn normalizes_model_friendly_ids_and_partial_styles() {
        let document = parse_generated_document(
            r##"{
              "document": {
                "id": "doc",
                "name": "Loose Mascot",
                "artboards": [{
                  "id": "main-board",
                  "name": "Loose",
                  "width": 960,
                  "height": 540,
                  "nodes": [{
                    "id": "rig",
                    "name": "Rig",
                    "kind": "group",
                    "children": [
                      {"id":"body","name":"Body","kind":"ellipse","style":{"fill":"#78c137"},"shape":{"type":"ellipse","cx":480,"cy":270,"rx":100,"ry":120}},
                      {"id":"face","name":"Face","kind":"rect","style":{"fill":"none","stroke":"#111"},"shape":{"type":"rect","x":400,"y":220,"width":160,"height":90,"rx":24}},
                      {"id":"eye-a","name":"EyeA","kind":"ellipse","shape":{"type":"ellipse","cx":440,"cy":250,"rx":10,"ry":10}},
                      {"id":"eye-b","name":"EyeB","kind":"ellipse","shape":{"type":"ellipse","cx":520,"cy":250,"rx":10,"ry":10}},
                      {"id":"smile","name":"Smile","kind":"path","shape":{"type":"path","d":"M450 290 C480 315 510 290"}}
                    ]
                  }]
                }],
                "timelines": [{"id":"wave-line","name":"wave","duration":800,"tracks":[{"target":"rig","property":"translate_y","keyframes":[{"time":0,"value":0,"easing":"easeOutQuad"},{"time":400,"value":8,"easing":"easeInOutQuad"}]}]}],
                "state_machines": [{"id":"moods","name":"Moods","states":[{"id":"idle","name":"Idle"},{"id":"wave","name":"Wave"}],"transitions":[]}],
                "bindings": [],
                "events": []
              }
            }"##,
        )
        .expect("loose document should normalize");

        assert_eq!(document.name, "Loose Mascot");
        assert_eq!(
            document.timelines[0].tracks[0].target,
            document.artboards[0].nodes[0].id
        );
        assert_eq!(document.timelines[0].duration_ms, 800);
        assert_eq!(document.timelines[0].tracks[0].property, "translation.y");
        assert_eq!(document.timelines[0].tracks[0].keyframes[0].time_ms, 0);
        assert_eq!(
            document.artboards[0].nodes[0].children[0].style.opacity,
            1.0
        );
        assert_eq!(document.artboards[0].nodes[0].children[1].style.fill, None);
        assert!(document.state_machines[0]
            .states
            .contains(&"wave".to_string()));
    }

    #[test]
    fn parses_full_document_from_streaming_cli_output() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_minimal_bot()}).to_string();
        let text = cli_assistant_text(
            &serde_json::json!({"type":"assistant","message":{"content":document_text}})
                .to_string(),
        );
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.name, "Minimal Bot");
    }

    #[test]
    fn parses_full_document_from_gemini_delta_chunks() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_owl_mascot()}).to_string();
        let split_at = document_text.len() / 2;
        let (first, second) = document_text.split_at(split_at);
        let text = cli_assistant_text(&format!(
            "{}\n{}\n{}",
            serde_json::json!({"type":"message","role":"user","content":"Return a full Strut document."}),
            serde_json::json!({"type":"message","role":"assistant","content":first,"delta":true}),
            serde_json::json!({"type":"message","role":"assistant","content":second,"delta":true})
        ));
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.state_machines[0].name, "OwlMoods");
    }

    #[test]
    fn rejects_empty_implicit_generation_plan() {
        let error = parse_generated_document(
            r##"{
              "plan": {
                "name": "Generated",
                "subject": {"classification": "object", "label": "Rolling dice"},
                "states": ["idle"],
                "parts": [],
                "timelines": []
              },
              "operations": []
            }"##,
        )
        .expect_err("empty plans should not become ready documents");

        assert!(
            error.contains("semantic parts"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn contextual_prompt_carries_chat_history_and_current_document() {
        let context = GenerationContext {
            project_name: Some("Mascot Game".to_string()),
            project_path: Some("D:\\Strut Projects\\Mascot Game".to_string()),
            active_chat_title: Some("Follow-up edits".to_string()),
            response_mode: Some("preview".to_string()),
            current_document_summary: Some(
                "Owl Mascot; 12 editable layers; states: idle, wave".to_string(),
            ),
            chat_history: vec![
                GenerationContextMessage {
                    role: "user".to_string(),
                    text: "make a green owl mascot".to_string(),
                    attachments: Some(vec!["owl-reference.png".to_string()]),
                },
                GenerationContextMessage {
                    role: "assistant".to_string(),
                    text: "Owl Mascot is ready.".to_string(),
                    attachments: None,
                },
            ],
            current_document: Some(strut_core::Document::sample_owl_mascot()),
        };

        let prompt =
            contextual_generation_prompt("make it cheer when level completes", Some(&context), models::GenerationStrategy::ProviderPlan);

        assert!(prompt.contains("Project: Mascot Game"));
        assert!(prompt.contains("make a green owl mascot"));
        assert!(prompt.contains("owl-reference.png"));
        assert!(prompt.contains("Current editable Strut document"));
        assert!(prompt.contains("Owl Mascot"));
        assert!(prompt.contains("make it cheer when level completes"));
    }

    #[test]
    fn chat_prompt_answers_normally_and_carries_context() {
        let context = GenerationContext {
            project_name: Some("Dice Lab".to_string()),
            project_path: None,
            active_chat_title: Some("Rolling die planning".to_string()),
            response_mode: Some("chat".to_string()),
            current_document_summary: Some(
                "3D Rolling Die; states: idle, rolling, settle".to_string(),
            ),
            chat_history: vec![GenerationContextMessage {
                role: "user".to_string(),
                text: "Create a smooth rolling dice animation".to_string(),
                attachments: None,
            }],
            current_document: None,
        };

        let prompt = chat_system_prompt(
            "how did you make this and what should we change?",
            Some(&context),
        );

        assert!(prompt.contains("Answer normal questions directly"));
        assert!(prompt.contains("Do not emit JSON"));
        assert!(prompt.contains("Dice Lab"));
        assert!(prompt.contains("Create a smooth rolling dice animation"));
        assert!(!prompt.contains("output standard valid JSON"));
    }

    #[test]
    fn repair_prompt_preserves_request_and_validation_error() {
        let prompt = document_repair_prompt(
            "make pikachu style character giving an electric shock",
            "I made a yellow mascot but forgot the JSON.",
            "model did not return a valid Strut document",
        );

        assert!(prompt.contains("make pikachu style character"));
        assert!(prompt.contains("model did not return a valid Strut document"));
        assert!(prompt.contains("Previous invalid response"));
        assert!(prompt.contains("{\"document\": <StrutDocument>}"));
        assert!(prompt.contains("Do not explain"));
    }


    #[test]
    fn rolling_dice_plan_does_not_produce_mascot_anatomy() {
        let planned = parse_test_planned_document(&phase3_plan_text(
            "dice",
            "Rolling Dice",
            vec![
                phase3_part("DieBody", "DieBody", "volume", json!({"kind":"rect","x":378,"y":174,"width":210,"height":210,"rx":24})),
                phase3_part("FrontFace", "FrontFace", "front face", json!({"kind":"rect","x":402,"y":214,"width":168,"height":146,"rx":16})),
                phase3_part("TopFace", "TopFace", "top face", json!({"kind":"path","d":"M402 214 L454 168 L618 184 L570 214 Z"})),
                phase3_part("Pips", "Pips", "number marks", json!({"kind":"path","d":"M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M530 320 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"})),
                phase3_part("EdgeHighlight", "EdgeHighlight", "edge light", json!({"kind":"path","d":"M414 228 L454 188 L604 202"})),
                phase3_part("SettleShadow", "SettleShadow", "grounding shadow", json!({"kind":"ellipse","cx":494,"cy":414,"rx":116,"ry":18})),
            ],
            "settle",
            "DieBody",
        ))
        .expect("dice plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "DieBody"));
        assert!(names.iter().any(|name| name == "Pips"));
        assert!(names
            .iter()
            .all(|name| !matches!(name.as_str(), "Head" | "Eyes" | "Arms" | "Legs" | "Smile")));
        assert!(planned
            .summary
            .timeline_names
            .contains(&"settle".to_string()));
        assert!(planned.operation_count >= 10);
    }

    #[test]
    fn abstract_logo_plan_does_not_require_face() {
        let planned = parse_test_planned_document(&phase3_plan_text(
            "logo",
            "Abstract Logo",
            vec![
                phase3_part("PrimaryMark", "PrimaryMark", "main vector mark", json!({"kind":"path","d":"M382 180 C450 120 540 146 582 222 C520 206 470 234 432 306 C398 266 370 226 382 180 Z"})),
                phase3_part("Wordmark", "Wordmark", "brand text", json!({"kind":"text","x":396,"y":384,"value":"STRUT","size":42})),
                phase3_part("AccentStroke", "AccentStroke", "accent line", json!({"kind":"path","d":"M392 326 C452 352 528 348 596 312"})),
                phase3_part("RevealMask", "RevealMask", "reveal mask", json!({"kind":"rect","x":360,"y":154,"width":280,"height":250,"rx":20})),
                phase3_part("AnchorGrid", "AnchorGrid", "alignment grid", json!({"kind":"path","d":"M360 270 L640 270 M500 150 L500 410"})),
                phase3_part("Glow", "Glow", "soft emphasis", json!({"kind":"ellipse","cx":498,"cy":266,"rx":118,"ry":76})),
            ],
            "reveal",
            "PrimaryMark",
        ))
        .expect("logo plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "PrimaryMark"));
        assert!(names.iter().any(|name| name == "Wordmark"));
        assert!(names.iter().all(|name| name != "Face" && name != "Eyes"));
    }

    #[test]
    fn loader_plan_does_not_require_face_or_body() {
        let planned = parse_test_planned_document(&phase3_plan_text(
            "loader",
            "Progress Loader",
            vec![
                phase3_part(
                    "Track",
                    "Track",
                    "background track",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":120,"ry":120}),
                ),
                phase3_part(
                    "ActiveSegment",
                    "ActiveSegment",
                    "active arc",
                    json!({"kind":"path","d":"M480 150 A120 120 0 0 1 600 270"}),
                ),
                phase3_part(
                    "PulseDot",
                    "PulseDot",
                    "pulse marker",
                    json!({"kind":"ellipse","cx":600,"cy":270,"rx":14,"ry":14}),
                ),
                phase3_part(
                    "ProgressSweep",
                    "ProgressSweep",
                    "sweep indicator",
                    json!({"kind":"path","d":"M480 270 L600 270"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "soft glow",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":144,"ry":144}),
                ),
                phase3_part(
                    "CenterLabel",
                    "CenterLabel",
                    "progress label",
                    json!({"kind":"text","x":454,"y":282,"value":"42%","size":24}),
                ),
            ],
            "loading",
            "ActiveSegment",
        ))
        .expect("loader plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "ActiveSegment"));
        assert!(names.iter().all(|name| name != "Face" && name != "Body"));
        assert!(planned.document.state_machines[0]
            .states
            .contains(&"loading".to_string()));
    }

    #[test]
    fn mascot_plan_can_still_use_mascot_parts() {
        let planned = parse_test_planned_document(&phase3_plan_text(
            "mascot",
            "Helpful Mascot",
            vec![
                phase3_part("Body", "Body", "body", json!({"kind":"ellipse","cx":480,"cy":306,"rx":92,"ry":118})),
                phase3_part("Head", "Head", "head", json!({"kind":"ellipse","cx":480,"cy":190,"rx":82,"ry":68})),
                phase3_part("Eyes", "Eyes", "eyes", json!({"kind":"path","d":"M446 186 q10 -16 20 0 M494 186 q10 -16 20 0"})),
                phase3_part("Arms", "Arms", "arms", json!({"kind":"path","d":"M394 292 C350 310 344 352 382 364 M566 292 C610 310 616 352 578 364"})),
                phase3_part("AccentBadge", "AccentBadge", "accent", json!({"kind":"ellipse","cx":512,"cy":316,"rx":16,"ry":16})),
                phase3_part("GroundShadow", "GroundShadow", "shadow", json!({"kind":"ellipse","cx":480,"cy":438,"rx":108,"ry":16})),
            ],
            "wave",
            "Body",
        ))
        .expect("mascot plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "Body"));
        assert!(names.iter().any(|name| name == "Head"));
        assert!(names.iter().any(|name| name == "Eyes"));
    }

    #[test]
    fn generation_plans_reject_invalid_references_and_geometry() {
        let duplicate = phase3_plan_text(
            "logo",
            "Bad Logo",
            vec![
                phase3_part(
                    "PrimaryMark",
                    "PrimaryMark",
                    "main",
                    json!({"kind":"path","d":"M0 0 L10 10"}),
                ),
                phase3_part(
                    "PrimaryMark",
                    "AccentStroke",
                    "accent",
                    json!({"kind":"path","d":"M0 10 L10 0"}),
                ),
                phase3_part(
                    "RevealMask",
                    "RevealMask",
                    "mask",
                    json!({"kind":"rect","x":1,"y":1,"width":10,"height":10,"rx":2}),
                ),
                phase3_part(
                    "AnchorGrid",
                    "AnchorGrid",
                    "grid",
                    json!({"kind":"path","d":"M1 1 L10 1"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "glow",
                    json!({"kind":"ellipse","cx":5,"cy":5,"rx":4,"ry":4}),
                ),
            ],
            "reveal",
            "PrimaryMark",
        );
        assert!(document_from_generation_plan_text(&duplicate)
            .expect_err("duplicate ids should reject")
            .contains("duplicate part id"));

        let missing_target = phase3_plan_text(
            "loader",
            "Bad Loader",
            vec![
                phase3_part(
                    "Track",
                    "Track",
                    "track",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":120,"ry":120}),
                ),
                phase3_part(
                    "ActiveSegment",
                    "ActiveSegment",
                    "active",
                    json!({"kind":"path","d":"M480 150 A120 120 0 0 1 600 270"}),
                ),
                phase3_part(
                    "PulseDot",
                    "PulseDot",
                    "dot",
                    json!({"kind":"ellipse","cx":600,"cy":270,"rx":14,"ry":14}),
                ),
                phase3_part(
                    "ProgressSweep",
                    "ProgressSweep",
                    "sweep",
                    json!({"kind":"path","d":"M480 270 L600 270"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "glow",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":144,"ry":144}),
                ),
            ],
            "loading",
            "MissingPart",
        );
        assert!(document_from_generation_plan_text(&missing_target)
            .expect_err("unknown timeline target should reject")
            .contains("missing part"));

        let bad_geometry = phase3_plan_text(
            "dice",
            "Bad Dice",
            vec![
                phase3_part(
                    "DieBody",
                    "DieBody",
                    "body",
                    json!({"kind":"rect","x":1,"y":1,"width":0,"height":10,"rx":2}),
                ),
                phase3_part(
                    "FrontFace",
                    "FrontFace",
                    "face",
                    json!({"kind":"rect","x":1,"y":1,"width":10,"height":10,"rx":2}),
                ),
                phase3_part(
                    "TopFace",
                    "TopFace",
                    "face",
                    json!({"kind":"path","d":"M0 0 L10 10"}),
                ),
                phase3_part(
                    "Pips",
                    "Pips",
                    "pips",
                    json!({"kind":"path","d":"M1 1 L2 2"}),
                ),
                phase3_part(
                    "Shadow",
                    "Shadow",
                    "shadow",
                    json!({"kind":"ellipse","cx":5,"cy":5,"rx":4,"ry":4}),
                ),
            ],
            "settle",
            "DieBody",
        );
        assert!(document_from_generation_plan_text(&bad_geometry)
            .expect_err("invalid geometry should reject")
            .contains("invalid rect geometry"));
    }

    #[test]
    fn non_mascot_plan_rejects_mascot_only_anatomy() {
        let bad_logo = phase3_plan_text(
            "logo",
            "Logo With Face",
            vec![
                phase3_part(
                    "Body",
                    "Body",
                    "body",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":80,"ry":80}),
                ),
                phase3_part(
                    "Head",
                    "Head",
                    "head",
                    json!({"kind":"ellipse","cx":480,"cy":190,"rx":60,"ry":50}),
                ),
                phase3_part(
                    "Eyes",
                    "Eyes",
                    "eyes",
                    json!({"kind":"path","d":"M460 190 L470 190"}),
                ),
                phase3_part(
                    "PrimaryMark",
                    "PrimaryMark",
                    "mark",
                    json!({"kind":"path","d":"M420 240 L540 240"}),
                ),
                phase3_part(
                    "AccentStroke",
                    "AccentStroke",
                    "accent",
                    json!({"kind":"path","d":"M420 270 L540 270"}),
                ),
            ],
            "reveal",
            "PrimaryMark",
        );

        assert!(document_from_generation_plan_text(&bad_logo)
            .expect_err("non mascot anatomy should reject")
            .contains("mascot-only anatomy"));
    }

    #[test]
    fn open_project_folder_rejects_missing_folder() {
        let missing =
            std::env::temp_dir().join(format!("strut-missing-folder-{}", unix_timestamp()));

        let error = open_project_folder(missing.display().to_string())
            .expect_err("missing folders should not be opened");

        assert!(error.contains("Project folder does not exist"));
    }

    #[test]
    fn provider_config_path_is_local() {
        let path = provider_config_path().expect("config path");
        assert!(path.ends_with("byok.json"));
    }

    #[test]
    fn plain_questions_route_to_chat_not_generation() {
        assert_eq!(
            classify_request_intent("who are you?"),
            RequestIntent::Conversation
        );
        assert_eq!(
            classify_request_intent("brainstorm three directions before editing"),
            RequestIntent::Conversation
        );
        assert_eq!(
            classify_request_intent("generate a calm loader animation"),
            RequestIntent::Generate
        );
    }

    #[test]
    fn chat_only_context_forces_conversation_route() {
        let context = GenerationContext {
            project_name: Some("Dice Lab".to_string()),
            project_path: None,
            active_chat_title: Some("Plan the roll".to_string()),
            response_mode: Some("chat".to_string()),
            current_document_summary: None,
            chat_history: vec![],
            current_document: None,
        };

        assert!(context_requests_chat_response(Some(&context)));
    }

    #[test]
    fn assistant_result_serializes_plan_summary_for_studio() {
        let result = AssistantResult::DocumentCreated {
            message: "Created rolling dice".to_string(),
            source: "llm".to_string(),
            document: strut_core::Document::sample_minimal_bot(),
            plan_summary: Some(GenerationPlanSummary {
                subject_classification: "object".to_string(),
                subject_label: "Rolling dice".to_string(),
                part_names: vec!["Die Body".to_string(), "Face 1".to_string()],
                timeline_names: vec!["idle".to_string(), "rolling".to_string()],
            }),
            operation_count: Some(12),
        };

        let value = serde_json::to_value(result).expect("assistant result json");

        assert_eq!(value["planSummary"]["subjectLabel"], "Rolling dice");
        assert_eq!(value["operationCount"], 12);
    }

    #[test]
    fn dynamic_engine_strategy_separates_svg_and_sprite_work() {
        assert_eq!(
            classify_generation_strategy("make a simple svg logo reveal"),
            GenerationStrategy::SimpleSvg
        );
        assert_eq!(
            classify_generation_strategy(
                "create a cinematic mascot with expressive idle animation"
            ),
            GenerationStrategy::SpritePython
        );
    }

    #[test]
    fn semantic_outcome_plan_with_empty_tracks_gets_dynamic_tracks_without_subject_template() {
        let json_str = r##"{
            "plan": {
                "id": "weather-token-outcomes",
                "name": "Weather Token Outcomes",
                "subject": {"classification": "object", "label": "Weather token"},
                "parts": [
                    {"id": "TokenBody", "name": "Token Body", "role": "body", "geometry": {"kind": "rect", "x": 410, "y": 210, "width": 140, "height": 120, "rx": 18}, "style": {"fill": "#f8fafc", "stroke": "#0f172a", "stroke_width": 3}, "motion_roles": ["spin"], "constraints": {"editable": true, "allowed_properties": ["translation.y", "rotation", "fill"]}},
                    {"id": "SunResult", "name": "Sun Result", "role": "result sun", "geometry": {"kind": "ellipse", "cx": 480, "cy": 270, "rx": 24, "ry": 24}, "style": {"fill": "#facc15", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}},
                    {"id": "RainResult", "name": "Rain Result", "role": "result rain", "geometry": {"kind": "path", "d": "M460 250 Q480 230 500 250 Q515 270 490 288 L470 288 Q445 270 460 250 Z"}, "style": {"fill": "#38bdf8", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}},
                    {"id": "WindResult", "name": "Wind Result", "role": "result wind", "geometry": {"kind": "path", "d": "M440 260 C470 240 500 280 530 260 M450 285 C480 265 505 300 525 285"}, "style": {"fill": "none", "stroke": "#64748b", "stroke_width": 5, "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                    {"id": "Shadow", "name": "Ground Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 480, "cy": 350, "rx": 70, "ry": 14}, "style": {"fill": "#0f172a", "opacity": 0.18}, "motion_roles": ["spin"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}}
                ],
                "motion_roles": [
                    {"id": "spin", "purpose": "token movement before a result", "part_refs": ["TokenBody", "Shadow"]},
                    {"id": "reveal", "purpose": "show selected outcome layer", "part_refs": ["SunResult", "RainResult", "WindResult"]}
                ],
                "states": ["idle", "sun", "rain", "wind"],
                "timelines": [
                    {"id": "to_sun", "name": "Result Sun", "state": "sun", "duration_ms": 1000, "tracks": []},
                    {"id": "to_rain", "name": "Result Rain", "state": "rain", "duration_ms": 1000, "tracks": []},
                    {"id": "to_wind", "name": "Result Wind", "state": "wind", "duration_ms": 1000, "tracks": []}
                ],
                "editability": {"editable_parts": ["TokenBody", "SunResult", "RainResult", "WindResult"], "locked_parts": ["Shadow"], "notes": []}
            },
            "operations": []
        }"##;

        let planned = parse_test_planned_document(json_str).expect("semantic outcome plan compiles");
        assert_eq!(count_document_nodes(&planned.document), 6); // 1 root group + 5 flat nodes
        assert!(
            planned.document.timelines.iter().all(|timeline| !timeline.tracks.is_empty()),
            "generic outcome compiler should enrich empty timelines"
        );
        let result_track_counts = planned
            .document
            .timelines
            .iter()
            .map(|timeline| {
                timeline
                    .tracks
                    .iter()
                    .filter(|track| track.property == "opacity")
                    .count()
            })
            .collect::<Vec<_>>();
        assert!(
            result_track_counts.iter().all(|count| *count >= 3),
            "each outcome timeline should drive reveal-layer visibility, got {result_track_counts:?}"
        );
    }

    #[test]
    fn semantic_compiler_does_not_invent_dice_only_parts() {
        let json_str = r##"{
            "plan": {
                "id": "dice-provider-blob",
                "name": "Rolling Dice",
                "subject": {"classification": "dice", "label": "Rolling Dice"},
                "parts": [
                    {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": 410, "y": 200, "width": 140, "height": 140, "rx": 18}, "style": {"fill": "#ffffff", "stroke": "#111827", "stroke_width": 4}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                    {"id": "FrontFace", "name": "Front Face", "role": "face", "geometry": {"kind": "rect", "x": 420, "y": 210, "width": 120, "height": 120, "rx": 16}, "style": {"fill": "#f8fafc", "stroke": "#cbd5e1", "stroke_width": 2}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                    {"id": "EdgeHighlight", "name": "Edge Highlight", "role": "highlight", "geometry": {"kind": "rect", "x": 432, "y": 220, "width": 96, "height": 8, "rx": 4}, "style": {"fill": "#ffffff", "opacity": 0.5}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                    {"id": "Pips", "name": "All Pips Blob", "role": "result face1", "geometry": {"kind": "path", "d": "M480 270 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"}, "style": {"fill": "#111827", "opacity": 0}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                    {"id": "Shadow", "name": "Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 480, "cy": 350, "rx": 70, "ry": 14}, "style": {"fill": "#111827", "opacity": 0.18}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}}
                ],
                "motion_roles": [],
                "states": ["idle", "face1"],
                "timelines": [
                    {"id": "face1", "name": "Face 1 Result", "state": "face1", "duration_ms": 900, "tracks": []}
                ],
                "editability": {"editable_parts": ["DieBody", "Pips"], "locked_parts": [], "notes": []}
            },
            "operations": []
        }"##;

        let planned = parse_test_planned_document(json_str).expect("dice plan compiles generically");
        let names = semantic_layer_names(&planned.document);
        assert!(!names.iter().any(|name| name == "Center Pip"));
        assert!(!names.iter().any(|name| name == "Bottom Right Pip"));
        assert!(names.iter().any(|name| name == "All Pips Blob"));
    }

    #[test]
    fn endpoint_guard_allows_loopback_and_blocks_private_networks() {
        assert!(ensure_safe_endpoint("http://localhost:1234/v1").is_ok());
        assert!(ensure_safe_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(ensure_safe_endpoint("https://api.openai.com/v1").is_ok());
        assert!(ensure_safe_endpoint("http://192.168.1.20:8080").is_err());
        assert!(ensure_safe_endpoint("ftp://api.example.com").is_err());
    }

    #[test]
    fn windows_command_candidates_prefer_executable_shims() {
        let candidates = command_candidates("gemini");
        if cfg!(windows) {
            let first = candidates
                .first()
                .and_then(|path| path.extension())
                .and_then(|extension| extension.to_str())
                .unwrap_or_default()
                .to_lowercase();
            assert_ne!(first, "");
        } else {
            assert_eq!(candidates, vec![PathBuf::from("gemini")]);
        }
    }

    #[test]
    #[ignore = "requires authenticated Gemini CLI"]
    fn gemini_cli_generates_owl_mascot_end_to_end() {
        // This test requires a live Gemini CLI session. The old API
        // (generate_document_with_local_adapter) was removed; generation
        // now flows through the Tauri command `generate_with_provider`.
        // Kept as a placeholder for manual E2E testing.
    }

    #[test]
    fn project_name_is_sanitized() {
        assert_eq!(
            sanitize_project_name("  My Bot / Demo!! ").expect("project name"),
            "My Bot Demo"
        );
    }

    #[test]
    fn dice_plan_with_empty_tracks_and_malformed_ops_succeeds() {
        // Exact reproduction: LLMs generate timelines with empty tracks in the plan,
        // putting keyframe data only in the operations array (in wrong format).
        // Before the fix, validate_generation_plan rejected empty tracks, causing
        // the entire parse chain to fail silently and show raw JSON in the chat bubble.
        let json_str = r##"{
            "kind": "document_created",
            "message": "Created a rolling dice animation",
            "document": {
                "plan": {
                    "id": "dice_roll_system",
                    "name": "Rolling Dice",
                    "subject": {"classification": "dice", "label": "Rolling Dice"},
                    "parts": [
                        {"id": "SettleShadow", "name": "Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 200, "cy": 255, "rx": 50, "ry": 10}, "style": {"fill": "#000000", "opacity": 0.5}},
                        {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": 150, "y": 150, "width": 100, "height": 100, "rx": 16}, "style": {"fill": "#FFFFFF", "stroke": "#D1D1D1", "stroke_width": 2}},
                        {"id": "Face1", "name": "Face 1", "role": "detail", "geometry": {"kind": "path", "d": "M200,200 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}},
                        {"id": "Face2", "name": "Face 2", "role": "detail", "geometry": {"kind": "path", "d": "M175,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}},
                        {"id": "Face3", "name": "Face 3", "role": "detail", "geometry": {"kind": "path", "d": "M175,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M200,200 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}},
                        {"id": "Face4", "name": "Face 4", "role": "detail", "geometry": {"kind": "path", "d": "M175,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M225,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}},
                        {"id": "Face5", "name": "Face 5", "role": "detail", "geometry": {"kind": "path", "d": "M175,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M225,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M200,200 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}},
                        {"id": "Face6", "name": "Face 6", "role": "detail", "geometry": {"kind": "path", "d": "M175,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M225,175 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0 M175,200 m-6,0 a6,6 0 1,0 12,0 a6,6 0 1,0 -12,0"}, "style": {"fill": "#333333", "opacity": 0}}
                    ],
                    "motion_roles": [
                        {"id": "roll", "purpose": "Main tumbling motion", "part_refs": ["DieBody", "Face1", "Face2", "Face3", "Face4", "Face5", "Face6"]},
                        {"id": "settle", "purpose": "Shadow response", "part_refs": ["SettleShadow"]}
                    ],
                    "states": ["idle", "roll_1", "roll_2", "roll_3", "roll_4", "roll_5", "roll_6"],
                    "timelines": [
                        {"id": "t1", "name": "Roll 1", "state": "roll_1", "duration_ms": 1200, "tracks": []},
                        {"id": "t2", "name": "Roll 2", "state": "roll_2", "duration_ms": 1200, "tracks": []},
                        {"id": "t3", "name": "Roll 3", "state": "roll_3", "duration_ms": 1200, "tracks": []},
                        {"id": "t4", "name": "Roll 4", "state": "roll_4", "duration_ms": 1200, "tracks": []},
                        {"id": "t5", "name": "Roll 5", "state": "roll_5", "duration_ms": 1200, "tracks": []},
                        {"id": "t6", "name": "Roll 6", "state": "roll_6", "duration_ms": 1200, "tracks": []}
                    ],
                    "editability": {
                        "editable_parts": ["DieBody", "Face1", "Face2", "Face3", "Face4", "Face5", "Face6"],
                        "locked_parts": ["SettleShadow"],
                        "notes": ["Colors are editable"]
                    }
                },
                "operations": [
                    {"type": "create_node", "kind": "ellipse", "id": "SettleShadow", "name": "Shadow", "geometry": {"cx": 200, "cy": 255, "rx": 50, "ry": 10}, "style": {"fill": "#000000", "opacity": 0.5}},
                    {"type": "add_timeline", "id": "t1"},
                    {"type": "add_keyframe", "timeline": "t1", "target": "DieBody", "property": "translation.y", "keyframes": [{"time": 0, "value": 0}, {"time": 800, "value": 0}]}
                ]
            }
        }"##;

        let result = parse_assistant_result(json_str);
        assert!(result.is_ok(), "parse_assistant_result must not return Err for dice plan with empty tracks");

        match result.unwrap() {
            AssistantResult::DocumentCreated { document, message, .. } => {
                assert!(!message.is_empty());
                assert!(!document.artboards.is_empty(), "document must have artboards");
                assert!(document.artboards[0].nodes.len() >= 1, "document must have nodes");
                assert!(document.timelines.len() >= 6, "document must have 6 timelines for dice faces");
                assert!(
                    document.timelines.iter().all(|timeline| !timeline.tracks.is_empty()),
                    "semantic fallback should enrich empty provider timelines with real tracks"
                );
                assert!(
                    document
                        .timelines
                        .iter()
                        .flat_map(|timeline| &timeline.tracks)
                        .any(|track| track.property == "opacity"),
                    "semantic fallback should add reveal opacity tracks so result states differ"
                );
                let layer_names = semantic_layer_names(&document);
                for expected in ["Face 1", "Face 2", "Face 3", "Face 4", "Face 5", "Face 6"] {
                    assert!(
                        layer_names.iter().any(|name| name == expected),
                        "semantic compiler should preserve provider-authored {expected} layer"
                    );
                }
            }
            other => panic!("expected DocumentCreated, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn provider_plan_with_scale_constraints_becomes_document() {
        let json_str = r##"{
            "kind": "document_created",
            "message": "Created a rolling dice animation system.",
            "document": {
                "plan": {
                    "id": "dice_roll_master",
                    "name": "Six-Sided Dice Roll",
                    "subject": {"classification": "dice", "label": "Rolling Dice"},
                    "parts": [
                        {"id": "DieBody", "name": "Die Body", "role": "primary", "geometry": {"kind": "rect", "x": 60, "y": 60, "width": 80, "height": 80, "rx": 14}, "style": {"fill": "#ffffff", "stroke": "#d1d5db", "stroke_width": 2}, "motion_roles": ["roll", "settle"], "constraints": {"editable": true, "allowed_properties": ["fill", "rotation", "translation.y"]}},
                        {"id": "FrontFace", "name": "Front Face", "role": "detail", "geometry": {"kind": "rect", "x": 60, "y": 60, "width": 80, "height": 80, "rx": 14}, "style": {"fill": "#f9fafb", "opacity": 0.5}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "TopFace", "name": "Top Face", "role": "detail", "geometry": {"kind": "path", "d": "M 60 60 L 140 60 L 125 45 L 45 45 Z"}, "style": {"fill": "#e5e7eb"}, "motion_roles": ["roll"], "constraints": {"editable": false, "allowed_properties": ["fill"]}},
                        {"id": "Pips", "name": "Pips", "role": "detail", "geometry": {"kind": "path", "d": "M 100 100 m -5 0 a 5 5 0 1 0 10 0 a 5 5 0 1 0 -10 0"}, "style": {"fill": "#111827"}, "motion_roles": ["settle"], "constraints": {"editable": true, "allowed_properties": ["opacity", "fill"]}},
                        {"id": "EdgeHighlight", "name": "Edge Highlight", "role": "accent", "geometry": {"kind": "rect", "x": 65, "y": 65, "width": 70, "height": 4, "rx": 2}, "style": {"fill": "#ffffff", "opacity": 0.6}, "motion_roles": ["roll"], "constraints": {"editable": false, "allowed_properties": ["opacity"]}},
                        {"id": "SettleShadow", "name": "Settle Shadow", "role": "environment", "geometry": {"kind": "ellipse", "cx": 100, "cy": 180, "rx": 40, "ry": 10}, "style": {"fill": "#000000", "opacity": 0.15}, "motion_roles": ["roll", "settle"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}}
                    ],
                    "motion_roles": [
                        {"id": "roll", "purpose": "Tumbling and bouncing during the toss", "part_refs": ["DieBody", "TopFace", "SettleShadow"]},
                        {"id": "settle", "purpose": "Final alignment and landing on a specific face", "part_refs": ["Pips", "DieBody"]},
                        {"id": "reveal", "purpose": "Face highlight reveal", "part_refs": ["FrontFace", "EdgeHighlight"]}
                    ],
                    "states": ["idle", "rolling", "face_1", "face_2", "face_3", "face_4", "face_5", "face_6"],
                    "timelines": [
                        {"id": "roll_1", "name": "Result 1", "state": "face_1", "duration_ms": 1400, "tracks": []},
                        {"id": "roll_2", "name": "Result 2", "state": "face_2", "duration_ms": 1400, "tracks": []},
                        {"id": "roll_3", "name": "Result 3", "state": "face_3", "duration_ms": 1400, "tracks": []},
                        {"id": "roll_4", "name": "Result 4", "state": "face_4", "duration_ms": 1400, "tracks": []},
                        {"id": "roll_5", "name": "Result 5", "state": "face_5", "duration_ms": 1400, "tracks": []},
                        {"id": "roll_6", "name": "Result 6", "state": "face_6", "duration_ms": 1400, "tracks": []}
                    ],
                    "editability": {"editable_parts": ["DieBody", "Pips", "SettleShadow"], "locked_parts": ["TopFace", "EdgeHighlight"], "notes": []}
                },
                "operations": [
                    {"type": "create_node", "kind": "ellipse", "id": "SettleShadow", "name": "Shadow", "geometry": {"cx": 100, "cy": 180, "rx": 40, "ry": 10}, "style": {"fill": "#000000", "opacity": 0.15}},
                    {"type": "add_keyframe", "timeline": "roll_1", "target": "DieCube", "property": "translation.y", "keyframes": [{"time": 0, "value": 0}, {"time": 1400, "value": 0}]}
                ]
            }
        }"##;

        let result = parse_assistant_result(json_str).expect("provider plan parses");

        match result {
            AssistantResult::DocumentCreated { document, .. } => {
                let nodes = count_document_nodes(&document);
                assert!(nodes >= 6, "derived document should include planned parts");
                assert!(document.timelines.len() >= 6);
                assert!(document.timelines.iter().all(|timeline| !timeline.tracks.is_empty()));
            }
            other => panic!("expected DocumentCreated, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn semantic_compiler_repairs_static_provider_tracks_without_canonical_parts() {
        let json_str = r##"{
            "plan": {
                "id": "dice_bad_tracks",
                "name": "Rolling Dice Six Faces",
                "subject": {"classification": "dice", "label": "Rolling Dice"},
                "parts": [
                    {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": 410, "y": 200, "width": 140, "height": 140, "rx": 18}, "style": {"fill": "#ffffff", "stroke": "#111827", "stroke_width": 4}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                    {"id": "FrontFace", "name": "Front Face", "role": "face", "geometry": {"kind": "rect", "x": 420, "y": 210, "width": 120, "height": 120, "rx": 16}, "style": {"fill": "#f8fafc", "stroke": "#cbd5e1", "stroke_width": 2}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                    {"id": "EdgeHighlight", "name": "Edge Highlight", "role": "highlight", "geometry": {"kind": "rect", "x": 432, "y": 220, "width": 96, "height": 8, "rx": 4}, "style": {"fill": "#ffffff", "opacity": 0.5}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                    {"id": "Pips", "name": "All Pips Blob", "role": "pip", "geometry": {"kind": "path", "d": "M480 270 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"}, "style": {"fill": "#111827", "opacity": 1}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                    {"id": "Shadow", "name": "Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 480, "cy": 350, "rx": 70, "ry": 14}, "style": {"fill": "#111827", "opacity": 0.18}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}}
                ],
                "motion_roles": [],
                "states": ["idle", "rolling", "face1", "face2", "face3", "face4", "face5", "face6"],
                "timelines": [
                    {"id": "face1", "name": "Face 1 Result", "state": "face1", "duration_ms": 900, "tracks": [
                        {"target": "Pips", "property": "opacity", "keyframes": [{"time": 0, "value": 1}, {"time": 900, "value": 1}]}
                    ]}
                ],
                "editability": {"editable_parts": ["DieBody"], "locked_parts": [], "notes": []}
            },
            "operations": []
        }"##;

        let planned = parse_test_planned_document(json_str).expect("dice plan compiles");
        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "All Pips Blob"));
        assert!(!names.iter().any(|name| name == "Center Pip"));
        assert!(!names.iter().any(|name| name == "Bottom Right Pip"));
        let face1 = planned
            .document
            .timelines
            .iter()
            .find(|timeline| timeline.name == "Face 1 Result")
            .expect("face1 timeline exists");
        let pip_opacity_tracks = face1
            .tracks
            .iter()
            .filter(|track| track.property == "opacity")
            .count();
        assert!(
            pip_opacity_tracks >= 1,
            "engine should repair static provider opacity without inventing subject-only layers, got {pip_opacity_tracks}: {:?}",
            face1
                .tracks
                .iter()
                .map(|track| track.property.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn provider_plan_with_loose_part_motion_roles_becomes_document() {
        let json_str = r##"{
            "kind": "document_created",
            "message": "Created a rolling dice animation with six distinct outcome timelines.",
            "document": {
                "plan": {
                    "id": "dice_roll_master",
                    "name": "Rolling Dice",
                    "subject": {"classification": "dice", "label": "Rolling Dice"},
                    "parts": [
                        {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": -40, "y": -40, "width": 80, "height": 80, "rx": 12}, "style": {"fill": "#ffffff", "stroke": "#d1d1d1", "stroke_width": 2}, "motion_roles": ["roll", "settle"], "constraints": {"editable": true, "allowed_properties": ["fill", "rotation", "translation.x", "translation.y"]}},
                        {"id": "EdgeHighlight", "name": "Edge Highlight", "role": "accent", "geometry": {"kind": "rect", "x": -36, "y": -36, "width": 72, "height": 72, "rx": 10}, "style": {"fill": "none", "stroke": "rgba(255,255,255,0.8)", "stroke_width": 1}, "motion_roles": ["idle"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "SettleShadow", "name": "Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 0, "cy": 50, "rx": 35, "ry": 8}, "style": {"fill": "rgba(0,0,0,0.15)", "opacity": 0.5}, "motion_roles": ["roll"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}},
                        {"id": "PipC", "name": "Center Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": 0, "cy": 0, "rx": 7, "ry": 7}, "style": {"fill": "#222222", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipTL", "name": "Top Left Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": -22, "cy": -22, "rx": 7, "ry": 7}, "style": {"fill": "#222222", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipTR", "name": "Top Right Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": 22, "cy": -22, "rx": 7, "ry": 7}, "style": {"fill": "#222222", "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}}
                    ],
                    "motion_roles": [
                        {"id": "roll", "purpose": "Tumbling rotation", "part_refs": ["DieBody", "SettleShadow"]},
                        {"id": "reveal", "purpose": "Outcome presentation", "part_refs": ["PipC", "PipTL", "PipTR"]}
                    ],
                    "states": ["idle", "rolling", "face1", "face2", "face3", "face4", "face5", "face6"],
                    "timelines": [
                        {"id": "roll_1", "name": "Roll to 1", "state": "face1", "duration_ms": 1200, "tracks": []},
                        {"id": "roll_2", "name": "Roll to 2", "state": "face2", "duration_ms": 1200, "tracks": []},
                        {"id": "roll_3", "name": "Roll to 3", "state": "face3", "duration_ms": 1200, "tracks": []},
                        {"id": "roll_4", "name": "Roll to 4", "state": "face4", "duration_ms": 1200, "tracks": []},
                        {"id": "roll_5", "name": "Roll to 5", "state": "face5", "duration_ms": 1200, "tracks": []},
                        {"id": "roll_6", "name": "Roll to 6", "state": "face6", "duration_ms": 1200, "tracks": []}
                    ],
                    "editability": {"editable_parts": ["DieBody", "PipC", "PipTL", "PipTR"], "locked_parts": ["SettleShadow", "EdgeHighlight"], "notes": []}
                },
                "operations": []
            }
        }"##;

        let result = parse_assistant_result(json_str).expect("loose per-part role labels should not reject the plan");

        match result {
            AssistantResult::DocumentCreated { document, .. } => {
                assert!(document.timelines.len() >= 6);
                assert!(document.timelines.iter().all(|timeline| !timeline.tracks.is_empty()));
                assert!(count_document_nodes(&document) >= 6);
            }
            other => panic!("expected DocumentCreated, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn assistant_result_parser_extracts_json_from_markdown_or_cli_text() {
        let text = r##"Here is the Strut JSON:
```json
{
  "kind": "document_created",
  "message": "Created a loader",
  "document": {
    "plan": {
      "id": "loader",
      "name": "Loader",
      "subject": {"classification": "loader", "label": "Loader"},
      "parts": [
        {"id": "Track", "name": "Track", "role": "base", "geometry": {"kind": "ellipse", "cx": 100, "cy": 100, "rx": 42, "ry": 42}, "style": {"fill": "#e5e7eb"}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
        {"id": "Segment", "name": "Segment", "role": "active", "geometry": {"kind": "path", "d": "M100 58 A42 42 0 0 1 142 100"}, "style": {"fill": "#22c55e"}, "constraints": {"editable": true, "allowed_properties": ["rotation"]}},
        {"id": "Dot", "name": "Dot", "role": "indicator", "geometry": {"kind": "ellipse", "cx": 142, "cy": 100, "rx": 6, "ry": 6}, "style": {"fill": "#0f172a"}, "constraints": {"editable": true, "allowed_properties": ["scale"]}},
        {"id": "Glow", "name": "Glow", "role": "accent", "geometry": {"kind": "ellipse", "cx": 100, "cy": 100, "rx": 50, "ry": 50}, "style": {"fill": "#86efac", "opacity": 0.25}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
        {"id": "Label", "name": "Label", "role": "text", "geometry": {"kind": "text", "x": 70, "y": 172, "value": "Loading", "size": 16}, "style": {"fill": "#111827"}, "constraints": {"editable": true, "allowed_properties": ["fill"]}}
      ],
      "motion_roles": [{"id": "spin", "purpose": "calm progress sweep", "part_refs": ["Segment", "Dot"]}],
      "states": ["idle", "loading"],
      "timelines": [{"id": "loading", "name": "Loading", "state": "loading", "duration_ms": 1200, "tracks": []}],
      "editability": {"editable_parts": ["Track", "Segment", "Dot", "Glow", "Label"], "locked_parts": [], "notes": []}
    },
    "operations": []
  }
}
```
Done."##;

        let result = parse_assistant_result_from_text(text).expect("json object should be extracted");
        assert!(matches!(result, AssistantResult::DocumentCreated { .. }));
    }

    #[test]
    fn codex_json_event_stream_unwraps_agent_message_text() {
        let inner = json!({
            "kind": "document_created",
            "message": "Created a loader",
            "document": {
                "plan": {
                    "id": "loader",
                    "name": "Loader",
                    "subject": {"classification": "loader", "label": "Loader"},
                    "parts": [
                        {"id": "Track", "name": "Track", "role": "base", "geometry": {"kind": "ellipse", "cx": 100, "cy": 100, "rx": 42, "ry": 42}, "style": {"fill": "#e5e7eb"}, "constraints": {"editable": true, "allowed_properties": ["fill"]}},
                        {"id": "Segment", "name": "Segment", "role": "active", "geometry": {"kind": "path", "d": "M100 58 A42 42 0 0 1 142 100"}, "style": {"fill": "#22c55e"}, "constraints": {"editable": true, "allowed_properties": ["rotation"]}},
                        {"id": "Dot", "name": "Dot", "role": "indicator", "geometry": {"kind": "ellipse", "cx": 142, "cy": 100, "rx": 6, "ry": 6}, "style": {"fill": "#0f172a"}, "constraints": {"editable": true, "allowed_properties": ["scale"]}},
                        {"id": "Glow", "name": "Glow", "role": "accent", "geometry": {"kind": "ellipse", "cx": 100, "cy": 100, "rx": 50, "ry": 50}, "style": {"fill": "#86efac", "opacity": 0.25}, "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "Label", "name": "Label", "role": "text", "geometry": {"kind": "text", "x": 70, "y": 172, "value": "Loading", "size": 16}, "style": {"fill": "#111827"}, "constraints": {"editable": true, "allowed_properties": ["fill"]}}
                    ],
                    "motion_roles": [{"id": "spin", "purpose": "calm progress sweep", "part_refs": ["Segment", "Dot"]}],
                    "states": ["idle", "loading"],
                    "timelines": [{"id": "loading", "name": "Loading", "state": "loading", "duration_ms": 1200, "tracks": []}],
                    "editability": {"editable_parts": ["Track", "Segment", "Dot", "Glow", "Label"], "locked_parts": [], "notes": []}
                },
                "operations": []
            }
        })
        .to_string();
        let stream = format!(
            "{}\n{}\n{}",
            json!({"type": "thread.started", "thread_id": "t1"}),
            json!({"type": "item.completed", "item": {"id": "item_0", "type": "agent_message", "text": inner}}),
            json!({"type": "turn.completed"})
        );

        let collected = cli_assistant_text(&stream);
        assert!(collected.contains("\"kind\":\"document_created\""));
        let result = parse_assistant_result_from_text(&collected)
            .expect("codex event stream should unwrap to Strut JSON");
        assert!(matches!(result, AssistantResult::DocumentCreated { .. }));
    }

    #[test]
    fn gemini_stream_json_delta_chunks_unwrap_to_assistant_result() {
        let inner = json!({
            "kind": "document_created",
            "message": "Created a rolling dice animation",
            "document": {
                "plan": {
                    "id": "rolling-dice-six-faces",
                    "name": "Rolling Dice",
                    "subject": {"classification": "dice", "label": "Rolling dice"},
                    "parts": [
                        {"id": "DieBody", "name": "Die Body", "role": "body", "geometry": {"kind": "rect", "x": 172, "y": 158, "width": 168, "height": 168, "rx": 24}, "style": {"fill": "#f8fafc", "stroke": "#0f172a", "stroke_width": 3, "opacity": 1}, "motion_roles": ["roll", "settle"], "constraints": {"editable": true, "allowed_properties": ["fill", "stroke", "translation.x", "translation.y", "rotation", "scale", "opacity"]}},
                        {"id": "SettleShadow", "name": "Settle Shadow", "role": "shadow", "geometry": {"kind": "ellipse", "cx": 256, "cy": 336, "rx": 86, "ry": 18}, "style": {"fill": "#111827", "stroke": "none", "stroke_width": 0, "opacity": 0.18}, "motion_roles": ["roll", "settle"], "constraints": {"editable": true, "allowed_properties": ["opacity", "scale"]}},
                        {"id": "PipCenter", "name": "Center Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": 256, "cy": 242, "rx": 11, "ry": 11}, "style": {"fill": "#0f172a", "stroke": "none", "stroke_width": 0, "opacity": 1}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipTopLeft", "name": "Top Left Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": 220, "cy": 206, "rx": 10, "ry": 10}, "style": {"fill": "#0f172a", "stroke": "none", "stroke_width": 0, "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}},
                        {"id": "PipBottomRight", "name": "Bottom Right Pip", "role": "pip", "geometry": {"kind": "ellipse", "cx": 292, "cy": 278, "rx": 10, "ry": 10}, "style": {"fill": "#0f172a", "stroke": "none", "stroke_width": 0, "opacity": 0}, "motion_roles": ["reveal"], "constraints": {"editable": true, "allowed_properties": ["opacity"]}}
                    ],
                    "motion_roles": [
                        {"id": "roll", "purpose": "small arcing tumble", "part_refs": ["DieBody", "SettleShadow"]},
                        {"id": "reveal", "purpose": "show final pips", "part_refs": ["PipCenter", "PipTopLeft", "PipBottomRight"]}
                    ],
                    "states": ["idle", "settle_face_1", "settle_face_2", "settle_face_3", "settle_face_4", "settle_face_5", "settle_face_6"],
                    "timelines": [
                        {"id": "roll_to_1", "name": "Roll to face 1", "state": "settle_face_1", "duration_ms": 1500, "tracks": []},
                        {"id": "roll_to_2", "name": "Roll to face 2", "state": "settle_face_2", "duration_ms": 1500, "tracks": []},
                        {"id": "roll_to_3", "name": "Roll to face 3", "state": "settle_face_3", "duration_ms": 1500, "tracks": []},
                        {"id": "roll_to_4", "name": "Roll to face 4", "state": "settle_face_4", "duration_ms": 1500, "tracks": []},
                        {"id": "roll_to_5", "name": "Roll to face 5", "state": "settle_face_5", "duration_ms": 1500, "tracks": []},
                        {"id": "roll_to_6", "name": "Roll to face 6", "state": "settle_face_6", "duration_ms": 1500, "tracks": []}
                    ],
                    "editability": {"editable_parts": ["DieBody", "SettleShadow", "PipCenter", "PipTopLeft", "PipBottomRight"], "locked_parts": [], "notes": []}
                },
                "operations": []
            }
        })
        .to_string();
        let split_at = inner.len() / 2;
        let (first, second) = inner.split_at(split_at);
        let stream = format!(
            "{}\n{}\n{}\n{}",
            json!({"type": "init", "model": "auto"}),
            json!({"type": "message", "role": "user", "content": "ignored"}),
            json!({"type": "message", "role": "assistant", "content": first, "delta": true}),
            json!({"type": "message", "role": "assistant", "content": second, "delta": true})
        );

        let collected = cli_assistant_text(&stream);
        let result = parse_assistant_result_from_text(&collected)
            .expect("gemini stream-json chunks should unwrap to Strut JSON");

        match result {
            AssistantResult::DocumentCreated { document, .. } => {
                assert!(document.timelines.len() >= 6);
                assert!(document.timelines.iter().all(|timeline| !timeline.tracks.is_empty()));
                assert!(count_document_nodes(&document) >= 5);
            }
            other => panic!("expected DocumentCreated, got {:?}", std::mem::discriminant(&other)),
        }
    }
