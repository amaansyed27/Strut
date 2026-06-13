use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_strut"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("strut-cli-test-{suffix}"));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn run(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(cli())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "command failed: {}\nstderr:\n{}\nstdout:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).expect("json output")
}

fn run_failure(args: &[&str], cwd: &Path) -> Output {
    let output = Command::new(cli())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run cli");
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}\nstdout:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout)
    );
    output
}

fn copied_sample_scene(root: &Path, temp: &Path) -> PathBuf {
    let scene = temp.join("scene.strut");
    fs::copy(root.join("samples/login-button.strut"), &scene).expect("copy scene");
    scene
}

fn sample_project(root: &Path, temp: &Path) -> PathBuf {
    let scene_dir = temp.join("scenes");
    let operations_dir = temp.join("operations");
    let ui_dir = temp.join("ui");
    fs::create_dir_all(&scene_dir).expect("scene dir");
    fs::create_dir_all(&operations_dir).expect("operations dir");
    fs::create_dir_all(&ui_dir).expect("ui dir");
    fs::copy(
        root.join("samples/login-button.strut"),
        scene_dir.join("main.strut"),
    )
    .expect("copy scene");
    fs::write(
        temp.join("strut.project.json"),
        serde_json::to_vec_pretty(&json!({
            "format": "strut.project",
            "version": "0.1.0",
            "name": "Phase 6 CLI Gallery Test",
            "mainScene": "scenes/main.strut",
            "operationBatches": "operations/operation-batches.json",
            "studioState": "ui/studio-state.json"
        }))
        .expect("manifest json"),
    )
    .expect("write manifest");
    fs::write(operations_dir.join("operation-batches.json"), "[]").expect("write batches");
    fs::write(ui_dir.join("studio-state.json"), "{}").expect("write state");
    temp.to_path_buf()
}

fn plan_json(root: &Path, instruction: &str) -> Value {
    run(
        &["plan", instruction, "--json", "--dry-run", "--explain"],
        root,
    )
}

fn sprite_plan_json(root: &Path, instruction: &str) -> Value {
    run(
        &[
            "sprite",
            "plan",
            instruction,
            "--json",
            "--dry-run",
            "--explain",
        ],
        root,
    )
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("write json");
}

fn part_names(plan: &Value) -> Vec<String> {
    plan["planSummary"]["partNames"]
        .as_array()
        .expect("part names")
        .iter()
        .map(|name| name.as_str().expect("part name").to_string())
        .collect()
}

fn assert_no_mascot_anatomy(names: &[String]) {
    let forbidden = ["Body", "Head", "Eyes", "Arms", "Legs", "Face", "Smile"];
    assert!(
        names.iter().all(|name| !forbidden.contains(&name.as_str())),
        "non-mascot names included mascot anatomy: {names:?}"
    );
}

#[test]
fn agentic_cli_smoke_path_validates_patch_render_and_export() {
    let root = repo_root();
    let temp = temp_dir();
    let scene = copied_sample_scene(&root, &temp);

    let inspect = run(
        &["inspect", "scene", scene.to_str().unwrap(), "--json"],
        &root,
    );
    assert_eq!(inspect["validation"]["ok"], true);
    assert_eq!(inspect["summary"]["name"], "Login Button");

    let plan = plan_json(&root, "make a calm dice animation");
    assert_eq!(plan["format"], "strut.cli.plan.v1");
    assert_eq!(plan["planSummary"]["subjectClassification"], "dice");
    let plan_path = temp.join("plan.json");
    write_json(&plan_path, &plan);

    let before = fs::read(&scene).expect("before");
    let dry_patch = run(
        &[
            "patch",
            "--scene",
            scene.to_str().unwrap(),
            "--from",
            plan_path.to_str().unwrap(),
            "--dry-run",
            "--json",
        ],
        &root,
    );
    assert_eq!(dry_patch["dryRun"], true);
    assert_eq!(fs::read(&scene).expect("after dry run"), before);

    let patch = run(
        &[
            "patch",
            "--scene",
            scene.to_str().unwrap(),
            "--from",
            plan_path.to_str().unwrap(),
            "--json",
        ],
        &root,
    );
    assert_eq!(patch["ok"], true);
    assert_eq!(patch["nextDocument"]["name"], "Rolling Dice Motion");
    assert_eq!(
        patch["nextDocument"]["name"],
        plan["batch"]["operations"][0]["nextDocument"]["name"]
    );

    let verify = run(&["verify", scene.to_str().unwrap(), "--json"], &root);
    assert_eq!(verify["ok"], true);
    assert_eq!(verify["summary"]["name"], "Rolling Dice Motion");

    let proof = temp.join("proof.svg");
    let render = run(
        &[
            "render",
            "--scene",
            scene.to_str().unwrap(),
            "--state",
            "settle",
            "--out",
            proof.to_str().unwrap(),
            "--json",
            "--no-open",
        ],
        &root,
    );
    assert_eq!(render["backend"], "cpu-fallback-svg-proof");
    assert!(fs::read_to_string(proof)
        .expect("proof")
        .contains("Rolling Dice Motion"));

    let export = run(
        &[
            "export",
            "react",
            "--scene",
            scene.to_str().unwrap(),
            "--out",
            temp.join("react-export").to_str().unwrap(),
            "--dry-run",
            "--json",
        ],
        &root,
    );
    assert_eq!(export["dryRun"], true);
    assert_eq!(export["files"].as_array().expect("files").len(), 3);
}

#[test]
fn phase6_gallery_runs_full_cli_project_flow_for_all_examples() {
    let root = repo_root();
    let cases = [
        (
            "dice",
            "make rolling dice settle softly",
            "dice",
            "Rolling Dice Motion",
            "settle",
        ),
        (
            "logo",
            "make an abstract logo reveal",
            "logo",
            "Abstract Logo Motion",
            "reveal",
        ),
        (
            "loader",
            "make a calm progress loader animation",
            "loader",
            "Progress Loader Motion",
            "loading",
        ),
        (
            "mascot",
            "make a low energy companion mascot idle animation",
            "mascot",
            "Companion Mascot Motion",
            "idle",
        ),
        (
            "ui",
            "make a button UI microinteraction",
            "ui",
            "Button Microinteraction Motion",
            "hover",
        ),
        (
            "icon",
            "make a success icon badge animation",
            "badge",
            "Icon Badge Motion",
            "success",
        ),
    ];

    for (slug, instruction, classification, document_name, state) in cases {
        let temp = temp_dir();
        let project = sample_project(&root, &temp);
        let scene = project.join("scenes/main.strut");

        let project_inspect = run(
            &["inspect", "project", project.to_str().unwrap(), "--json"],
            &root,
        );
        assert_eq!(project_inspect["currentDocument"]["name"], "Login Button");
        assert_eq!(project_inspect["canonicalFiles"][1]["exists"], true);

        let plan = sprite_plan_json(&root, instruction);
        assert_eq!(plan["format"], "strut.cli.plan.v1");
        assert_eq!(plan["dryRun"], true);
        assert!(plan["backend"]
            .as_str()
            .expect("backend")
            .starts_with("sprite-python"));
        assert_eq!(plan["planSummary"]["subjectClassification"], classification);
        assert_eq!(plan["document"]["name"], document_name);
        assert!(plan["envelope"]["document"].is_null());
        assert!(
            plan["envelope"]["operations"]
                .as_array()
                .expect("operations")
                .len()
                >= 10
        );
        assert_eq!(plan["batch"]["operations"][0]["type"], "replace_document");
        let names = part_names(&plan);
        assert!(
            names.len() >= 5,
            "{slug} should have editable semantic parts"
        );
        assert!(plan["planSummary"]["timelineNames"]
            .as_array()
            .expect("timelines")
            .iter()
            .all(|name| name.as_str().is_some_and(|value| !value.trim().is_empty())));
        if classification == "mascot" {
            assert!(names.iter().any(|name| name == "Body"));
            assert!(plan["envelope"]["plan"]["motionRoles"][0]["purpose"]
                .as_str()
                .expect("purpose")
                .contains("quiet"));
        } else {
            assert_no_mascot_anatomy(&names);
        }

        let plan_path = temp.join(format!("{slug}.plan.json"));
        write_json(&plan_path, &plan);
        let before = fs::read(&scene).expect("before");
        let dry_patch = run(
            &[
                "patch",
                "--scene",
                scene.to_str().unwrap(),
                "--from",
                plan_path.to_str().unwrap(),
                "--dry-run",
                "--json",
            ],
            &root,
        );
        assert_eq!(dry_patch["dryRun"], true);
        assert_eq!(fs::read(&scene).expect("after dry run"), before);

        let patch = run(
            &[
                "patch",
                "--scene",
                scene.to_str().unwrap(),
                "--from",
                plan_path.to_str().unwrap(),
                "--json",
            ],
            &root,
        );
        assert_eq!(patch["nextDocument"]["name"], document_name);

        let scene_inspect = run(
            &["inspect", "scene", scene.to_str().unwrap(), "--json"],
            &root,
        );
        assert_eq!(scene_inspect["validation"]["ok"], true);
        assert_eq!(scene_inspect["summary"]["name"], document_name);
        assert!(scene_inspect["nodes"].as_array().expect("nodes").len() >= names.len());
        assert!(scene_inspect["semanticRoles"]
            .as_array()
            .expect("roles")
            .iter()
            .any(|role| role["name"] == names[0]));

        let verify = run(&["verify", scene.to_str().unwrap(), "--json"], &root);
        assert_eq!(verify["ok"], true);
        assert_eq!(verify["summary"]["name"], document_name);

        let proof = temp.join(format!("{slug}.proof.svg"));
        let render = run(
            &[
                "render",
                "--scene",
                scene.to_str().unwrap(),
                "--state",
                state,
                "--out",
                proof.to_str().unwrap(),
                "--json",
                "--no-open",
            ],
            &root,
        );
        assert_eq!(render["backend"], "cpu-fallback-svg-proof");
        assert!(fs::read_to_string(&proof)
            .expect("proof")
            .contains(document_name));

        let export_dir = temp.join(format!("{slug}-react-export"));
        let export = run(
            &[
                "export",
                "react",
                "--scene",
                scene.to_str().unwrap(),
                "--out",
                export_dir.to_str().unwrap(),
                "--json",
            ],
            &root,
        );
        assert_eq!(export["ok"], true);
        let scene_json = export_dir.join("scene.json");
        let component = export_dir.join("StrutAnimation.tsx");
        let readme = export_dir.join("README.md");
        assert!(scene_json.exists());
        assert!(component.exists());
        assert!(readme.exists());
        let exported_document: Value =
            serde_json::from_slice(&fs::read(scene_json).expect("scene json"))
                .expect("exported scene json");
        assert_eq!(exported_document["name"], document_name);
        let component_source = fs::read_to_string(component).expect("component");
        assert!(component_source.contains("export function StrutAnimation"));
        assert!(component_source.contains("data-strut-node"));
        assert!(component_source.contains("data-strut-id"));
        assert!(component_source.contains("function animationCss"));
        assert!(component_source.contains("@keyframes strut-"));
        assert!(component_source.contains("playAll = true"));
        assert!(fs::read_to_string(readme)
            .expect("readme")
            .contains("<StrutAnimation state=\"idle\" playAll />"));
    }
}

#[test]
fn sprite_plan_supports_prompt_specific_procedural_assets() {
    let root = repo_root();
    let plan = sprite_plan_json(&root, "animate a twitter bird taking flight");

    assert_eq!(plan["planSummary"]["subjectClassification"], "bird_icon");
    assert_eq!(
        plan["planSummary"]["subjectLabel"],
        "Twitter Bird Taking Flight"
    );
    assert_eq!(
        plan["document"]["name"],
        "Twitter Bird Taking Flight Motion"
    );
    assert_eq!(plan["batch"]["operations"][0]["type"], "replace_document");

    let names = part_names(&plan);
    assert!(names
        .iter()
        .any(|name| name == "Twitter Bird Taking Flight Body"));
    assert!(names
        .iter()
        .any(|name| name == "Twitter Bird Taking Flight Wing"));
    assert!(names
        .iter()
        .any(|name| name == "Twitter Bird Taking Flight Motion Trail"));
    assert_no_mascot_anatomy(&names);
}

#[test]
fn sprite_plan_keeps_compound_subjects_out_of_fixed_badge_fixture() {
    let root = repo_root();
    let plan = sprite_plan_json(
        &root,
        "make a glassy crystal volcano badge with lava shimmer and tiny smoke orbit, no face",
    );

    assert_eq!(
        plan["planSummary"]["subjectClassification"],
        "dynamic_asset"
    );
    assert_eq!(
        plan["planSummary"]["subjectLabel"],
        "Glassy Crystal Volcano Badge"
    );
    assert_eq!(
        plan["document"]["name"],
        "Glassy Crystal Volcano Badge Motion"
    );

    let names = part_names(&plan);
    assert!(names
        .iter()
        .any(|name| name.starts_with("Glassy Crystal Volcano")));
    assert!(names.iter().any(|name| name.contains("Lava")));
    assert!(names.iter().any(|name| name.contains("Smoke")));
    assert_no_mascot_anatomy(&names);
}

#[test]
fn patch_rejects_tampered_top_level_document_without_mutating_scene() {
    let root = repo_root();
    let temp = temp_dir();
    let scene = copied_sample_scene(&root, &temp);
    let mut plan = plan_json(&root, "make a calm dice animation");
    plan["document"]["name"] = Value::String("Unrelated Top Level Document".to_string());
    let plan_path = temp.join("tampered-plan.json");
    write_json(&plan_path, &plan);

    let before = fs::read(&scene).expect("before");
    let output = run_failure(
        &[
            "patch",
            "--scene",
            scene.to_str().unwrap(),
            "--from",
            plan_path.to_str().unwrap(),
            "--json",
        ],
        &root,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("plan document mismatch"), "{stderr}");
    assert_eq!(fs::read(&scene).expect("after failure"), before);
}

#[test]
fn patch_rejects_invalid_replacement_document_without_mutating_scene() {
    let root = repo_root();
    let temp = temp_dir();
    let scene = copied_sample_scene(&root, &temp);
    let mut plan = plan_json(&root, "make a calm dice animation");
    plan["batch"]["operations"][0]["nextDocument"]["artboards"][0]["width"] = json!(0.0);
    let plan_path = temp.join("invalid-replacement-plan.json");
    write_json(&plan_path, &plan);

    let before = fs::read(&scene).expect("before");
    let output = run_failure(
        &[
            "patch",
            "--scene",
            scene.to_str().unwrap(),
            "--from",
            plan_path.to_str().unwrap(),
            "--json",
        ],
        &root,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("nextDocument failed validation")
            || stderr.contains("artboard")
            || stderr.contains("dimensions"),
        "{stderr}"
    );
    assert_eq!(fs::read(&scene).expect("after failure"), before);
}

#[test]
fn react_export_preflights_conflicts_and_force_overwrites_all_files() {
    let root = repo_root();
    let temp = temp_dir();
    let scene = copied_sample_scene(&root, &temp);
    let export_dir = temp.join("react-export");
    fs::create_dir_all(&export_dir).expect("export dir");
    let component = export_dir.join("StrutAnimation.tsx");
    fs::write(&component, "old component").expect("conflict");

    let dry_run_dir = temp.join("dry-run-export");
    let dry_run = run(
        &[
            "export",
            "react",
            "--scene",
            scene.to_str().unwrap(),
            "--out",
            dry_run_dir.to_str().unwrap(),
            "--dry-run",
            "--json",
        ],
        &root,
    );
    assert_eq!(dry_run["dryRun"], true);
    assert!(
        !dry_run_dir.exists(),
        "dry-run must not create export directory"
    );

    let output = run_failure(
        &[
            "export",
            "react",
            "--scene",
            scene.to_str().unwrap(),
            "--out",
            export_dir.to_str().unwrap(),
            "--json",
        ],
        &root,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("refusing to overwrite"), "{stderr}");
    assert_eq!(
        fs::read_to_string(&component).expect("component"),
        "old component"
    );
    assert!(!export_dir.join("scene.json").exists());
    assert!(!export_dir.join("README.md").exists());

    let force = run(
        &[
            "export",
            "react",
            "--scene",
            scene.to_str().unwrap(),
            "--out",
            export_dir.to_str().unwrap(),
            "--force",
            "--json",
        ],
        &root,
    );
    assert_eq!(force["ok"], true);
    assert!(export_dir.join("scene.json").exists());
    assert!(export_dir.join("README.md").exists());
    assert_ne!(
        fs::read_to_string(&component).expect("component"),
        "old component"
    );
}
