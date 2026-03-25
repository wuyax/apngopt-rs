use assert_cmd::Command;
use std::fs;

#[test]
fn test_cli_optimize_default() {
    let input_path = "test_opt.png";
    let output_path = "test_opt_opt.png"; // Default output naming

    // Ensure we start clean
    let _ = fs::remove_file(output_path);

    let mut cmd = Command::cargo_bin("apngopt-rs").unwrap();
    let assert = cmd.arg(input_path).assert();

    assert.success();

    // Check if the output file was created
    assert!(fs::metadata(output_path).is_ok());

    // Clean up
    let _ = fs::remove_file(output_path);
}

#[test]
fn test_cli_optimize_zopfli() {
    let input_path = "test_opt.png";
    let output_path = "test_opt_zopfli.png";

    // Ensure we start clean
    let _ = fs::remove_file(output_path);

    let mut cmd = Command::cargo_bin("apngopt-rs").unwrap();
    let assert = cmd
        .arg("-z")
        .arg("2") // zopfli
        .arg("-i")
        .arg("1") // 1 iteration for speed in testing
        .arg(input_path)
        .arg(output_path) // Explicit output
        .assert();

    assert.success();

    // Check if the output file was created
    assert!(fs::metadata(output_path).is_ok());

    // Clean up
    let _ = fs::remove_file(output_path);
}
