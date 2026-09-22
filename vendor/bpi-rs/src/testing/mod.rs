#[cfg(test)]
mod live_gate_tests {
    use std::path::Path;

    const OFFLINE_ASYNC_TEST_FILES: &[&str] = &["src/probe/run.rs"];

    #[test]
    fn legacy_tokio_tests_are_ignored_by_default() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut unguarded = Vec::new();

        for file in rust_files(root.join("src")) {
            let relative = file
                .strip_prefix(root)
                .expect("source file should be under root");
            let relative = relative.to_string_lossy().replace('\\', "/");
            if OFFLINE_ASYNC_TEST_FILES.contains(&relative.as_str()) {
                continue;
            }

            let source = std::fs::read_to_string(&file).expect("source file should be readable");
            let lines = source.lines().collect::<Vec<_>>();
            for (index, line) in lines.iter().enumerate() {
                if line.trim() != "#[tokio::test]" {
                    continue;
                }

                let has_ignore = lines[..index]
                    .iter()
                    .rev()
                    .take_while(|candidate| candidate.trim_start().starts_with("#["))
                    .any(|candidate| candidate.trim_start().starts_with("#[ignore"));

                if !has_ignore {
                    unguarded.push(format!("{}:{}", relative, index + 1));
                }
            }
        }

        assert!(
            unguarded.is_empty(),
            "legacy #[tokio::test] functions must be ignored by default:\n{}",
            unguarded.join("\n")
        );
    }

    fn rust_files(root: impl AsRef<Path>) -> Vec<std::path::PathBuf> {
        let mut output = Vec::new();
        collect_rust_files(root.as_ref(), &mut output);
        output
    }

    fn collect_rust_files(dir: &Path, output: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("source directory should be readable") {
            let entry = entry.expect("directory entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                collect_rust_files(&path, output);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                output.push(path);
            }
        }
    }
}
