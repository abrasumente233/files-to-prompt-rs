use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_basic_functionality() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2").unwrap();

    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd.arg(test_dir.to_str().unwrap()).assert();

    assert.success().stdout(
        predicate::str::contains("file1.txt")
            .and(predicate::str::contains("Contents of file1"))
            .and(predicate::str::contains("file2.txt"))
            .and(predicate::str::contains("Contents of file2")),
    );
}

#[test]
fn test_include_hidden() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join(".hidden.txt"), "Contents of hidden file").unwrap();

    // Default (no --include-hidden)
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd.arg(test_dir.to_str().unwrap()).assert();

    // Should NOT contain hidden file
    assert
        .success()
        .stdout(predicate::str::contains(".hidden.txt").not());

    // Now with --include-hidden
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([test_dir.to_str().unwrap(), "--include-hidden"])
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains(".hidden.txt"))
        .stdout(predicate::str::contains("Contents of hidden file"));
}

#[test]
fn test_ignore_gitignore() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join(".gitignore"), "ignored.txt").unwrap();
    fs::write(test_dir.join("ignored.txt"), "This file should be ignored").unwrap();
    fs::write(test_dir.join("included.txt"), "This file should be included").unwrap();

    // Normal run: .gitignore is respected
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd.arg(test_dir.to_str().unwrap()).assert();
    assert.success().stdout(
        predicate::str::contains("ignored.txt").not().and(
            predicate::str::contains("included.txt")
                .and(predicate::str::contains("This file should be included")),
        ),
    );

    // With --ignore-gitignore
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([test_dir.to_str().unwrap(), "--ignore-gitignore"])
        .assert();
    // now it should include "ignored.txt"
    assert.success().stdout(
        predicate::str::contains("ignored.txt").and(predicate::str::contains(
            "This file should be ignored",
        )),
    );
}

#[test]
fn test_multiple_paths() {
    let tmp = tempdir().unwrap();
    let test_dir1 = tmp.path().join("test_dir1");
    let test_dir2 = tmp.path().join("test_dir2");
    fs::create_dir(&test_dir1).unwrap();
    fs::create_dir(&test_dir2).unwrap();

    fs::write(test_dir1.join("file1.txt"), "Contents of file1").unwrap();
    fs::write(test_dir2.join("file2.txt"), "Contents of file2").unwrap();
    fs::write(tmp.path().join("single_file.txt"), "Contents of single file").unwrap();

    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([
            test_dir1.to_str().unwrap(),
            test_dir2.to_str().unwrap(),
            tmp.path().join("single_file.txt").to_str().unwrap(),
        ])
        .assert();

    assert.success().stdout(
        predicate::str::contains("test_dir1/file1.txt")
            .and(predicate::str::contains("Contents of file1"))
            .and(predicate::str::contains("test_dir2/file2.txt"))
            .and(predicate::str::contains("Contents of file2"))
            .and(predicate::str::contains("single_file.txt"))
            .and(predicate::str::contains("Contents of single file")),
    );
}

#[test]
fn test_ignore_patterns() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(
        test_dir.join("file_to_ignore.txt"),
        "This file should be ignored due to ignore patterns",
    )
    .unwrap();
    fs::write(
        test_dir.join("file_to_include.txt"),
        "This file should be included",
    )
    .unwrap();

    // ignoring all *.txt
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([test_dir.to_str().unwrap(), "--ignore", "*.txt"])
        .assert();

    assert.success().stdout(
        predicate::str::contains("file_to_ignore.txt").not().and(
            predicate::str::contains("file_to_include.txt").not(), // because *.txt matches both
        ),
    );

    // ignoring only file_to_ignore.*
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([test_dir.to_str().unwrap(), "--ignore", "file_to_ignore.*"])
        .assert();

    assert.success().stdout(
        predicate::str::contains("file_to_ignore.txt").not().and(
            predicate::str::contains("file_to_include.txt")
                .and(predicate::str::contains("This file should be included")),
        ),
    );
}

#[test]
fn test_specific_extensions() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir").join("two");
    fs::create_dir_all(&test_dir).unwrap();

    fs::write(tmp.path().join("test_dir").join("one.txt"), "This is one.txt").unwrap();
    fs::write(tmp.path().join("test_dir").join("one.py"), "This is one.py").unwrap();
    fs::write(test_dir.join("two.txt"), "This is two/two.txt").unwrap();
    fs::write(test_dir.join("two.py"), "This is two/two.py").unwrap();
    fs::write(
        tmp.path().join("test_dir").join("three.md"),
        "This is three.md",
    )
    .unwrap();

    // Only .py and .md
    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([
            tmp.path().join("test_dir").to_str().unwrap(),
            "-e",
            "py",
            "-e",
            "md",
        ])
        .assert();

    assert.success().stdout(
        predicate::str::contains("one.txt").not().and(
            predicate::str::contains("two.txt").not().and(
                predicate::str::contains("one.py")
                    .and(predicate::str::contains("two.py"))
                    .and(predicate::str::contains("three.md")),
            ),
        ),
    );
}

#[test]
fn test_binary_file_warning() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("binary_file.bin"), vec![0xff, 0x00, 0x12]).unwrap();
    fs::write(test_dir.join("text_file.txt"), "This is a text file").unwrap();

    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd.arg(test_dir.to_str().unwrap()).assert();

    // We expect to see the text file, but not the binary file
    assert.success().stdout(
        predicate::str::contains("text_file.txt")
            .and(predicate::str::contains("This is a text file"))
            .and(predicate::str::contains("binary_file.bin").not()),
    ).stderr(
        predicate::str::contains("Warning: Skipping file").and(
            predicate::str::contains("binary_file.bin").and(
                predicate::str::contains("due to").or(predicate::str::contains("invalid UTF-8")),
            ),
        ),
    );
}

#[test]
fn test_xml_format_dir() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1.txt").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2.txt").unwrap();

    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([test_dir.to_str().unwrap(), "--cxml"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    // Check structure
    assert!(stdout.contains("<documents>"));
    assert!(stdout.contains(r#"<document index="1">"#));
    assert!(stdout.contains("<source>"));
    assert!(stdout.contains("<document_content>"));
    assert!(stdout.contains("Contents of file1.txt"));
    assert!(stdout.contains("Contents of file2.txt"));
    assert!(stdout.contains("</documents>"));
}

#[test]
fn test_output_option() {
    let tmp = tempdir().unwrap();
    let test_dir = tmp.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "Contents of file1.txt").unwrap();
    fs::write(test_dir.join("file2.txt"), "Contents of file2.txt").unwrap();

    let output_path = tmp.path().join("output.txt");

    let mut cmd = Command::cargo_bin("files-to-prompt").unwrap();
    let assert = cmd
        .args([
            test_dir.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // No stdout
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.is_empty());

    // But the file has the content
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("test_dir/file1.txt"));
    assert!(contents.contains("Contents of file1.txt"));
    assert!(contents.contains("test_dir/file2.txt"));
    assert!(contents.contains("Contents of file2.txt"));
}

