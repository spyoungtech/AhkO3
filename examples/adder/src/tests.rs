use std::io::{Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, Output};

fn run_script(script_text: String) -> Output {
    let mut child = Command::new("autohotkeyv2.exe")
        .arg("/CP65001")
        .arg("/ErrorStdout=UTF-8")
        .arg("*")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(script_text.as_bytes()).expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().expect("Failed to read stdout");
    output


}

fn get_adder_ahk_location() -> PathBuf {
    let current_file_path = file!();
    let path = Path::new(current_file_path);
    let grandparent_path = path.parent().and_then(Path::parent).expect("Failed to find grandparent directory");
    grandparent_path.join("adder.ahk").iter().skip(2).collect()
}

fn get_dll_location() -> PathBuf {
    let current_file_path = file!();
    let path = Path::new(current_file_path);
    let grandparent_path = path.parent().and_then(Path::parent).expect("Failed to find grandparent directory");
    grandparent_path.join("target").join("x86_64-pc-windows-gnu").join("debug").join("adder.dll").iter().skip(2).collect()
}

fn make_script(script_text: &str) -> String {
    let header = format!("\
    #Warn All, Stdout\n\
    #DllLoad \"{}\" \n\
    #Include \"{}\"\n\
    stdout := FileOpen(\"*\", \"w\", \"UTF-8\")\n\
    stderr := FileOpen(\"**\", \"w\", \"UTF-8\")\n\
    writestdout(message) {{\n\
        stdout.Write(message)\n\
        stdout.Read(0)\n\
    }}\
    writestderr(message) {{\n\
        stderr.Write(message)\n\
        stderr.Read(0)\n\
    }}\
    ", get_dll_location().to_str().unwrap(), get_adder_ahk_location().to_str().unwrap());
    format!("{}\n\
    main(){{\n\
    {}\n\
    }}\n\
    try {{
        main()\n\
    }} catch Any as e {{\n\
        msg := Format(\"Error {{}} (line {{}}). The error message was: {{}}. Specifically: {{}}`nStack:`n{{}}\", e.what, e.line, e.message, e.extra, e.stack)\n\
        writestderr(msg)\n\
        Exit 1\n\
    }}\n\
    \r\n", header, script_text)
}


#[test]
fn test_noop() {
    run_script("obj := {}".to_string());
}

#[test]
fn test_noop_script() {
    let script = make_script("obj := {}");
    let output = run_script(script);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr, "");
    assert_eq!(stdout.to_string(), String::from(""));
    assert!(output.status.success());
}

#[test]
fn test_concatenate() {
    let script = make_script("f := concatenate(\"foo\", \"bar\")\nwritestdout(f)");
    let output = run_script(script);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr, "");
    assert_eq!(stdout.to_string(), String::from("foobar"));
    assert!(output.status.success());
}

#[test]
fn test_add() {
    let script = make_script("f := add(1, 2)\nwritestdout(f)");
    let output = run_script(script);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr, "");
    assert_eq!(stdout.to_string(), String::from("3"));
    assert!(output.status.success());

}