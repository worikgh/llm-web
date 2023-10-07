use std::process::Command;

fn main() {
    // Execute the custom build.sh script
    let output = Command::new("./build.sh")
        .output()
        .expect("Failed to execute build script");

    // Check the exit code of the script
    if !output.status.success() {
        panic!("Build script exited with an error");
    }
}
