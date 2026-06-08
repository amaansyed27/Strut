use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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

#[test]
fn agentic_cli_smoke_path_validates_patch_render_and_export() {
    let root = repo_root();
    let temp = temp_dir();
    let scene = temp.join("scene.strut");
    fs::copy(root.join("samples/login-button.strut"), &scene).expect("copy scene");

    let inspect = run(
        &["inspect", "scene", scene.to_str().unwrap(), "--json"],
        &root,
    );
    assert_eq!(inspect["validation"]["ok"], true);
    assert_eq!(inspect["summary"]["name"], "Login Button");

    let plan = run(
        &[
            "plan",
            "make a calm dice animation",
            "--json",
            "--dry-run",
            "--explain",
        ],
        &root,
    );
    assert_eq!(plan["format"], "strut.cli.plan.v1");
    assert_eq!(plan["planSummary"]["subjectClassification"], "dice");
    let plan_path = temp.join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec_pretty(&plan).expect("plan json"),
    )
    .expect("write plan");

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
