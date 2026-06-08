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

fn plan_json(root: &Path, instruction: &str) -> Value {
    run(
        &["plan", instruction, "--json", "--dry-run", "--explain"],
        root,
    )
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("write json");
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
