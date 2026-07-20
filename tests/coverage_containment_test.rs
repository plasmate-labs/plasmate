use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use plasmate::coverage::runner::{
    self, CoverageOptions, CoverageStatus, CoverageWorkerOutcome, FailureKind,
};

fn fixture_path() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/coverage_worker_fixture.rs");
            // Independent cargo invocations can run this test concurrently in
            // the same worktree. A process-unique path prevents one rustc from
            // replacing the fixture while another test process is executing it.
            let mut output = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("coverage-worker-fixture-{}", std::process::id()));
            if cfg!(windows) {
                output.set_extension("exe");
            }
            let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
            let compilation = Command::new(rustc)
                .args(["--edition=2021", "--crate-name", "coverage_worker_fixture"])
                .arg(&source)
                .arg("-o")
                .arg(&output)
                .output()
                .expect("failed to launch rustc for coverage worker fixture");
            assert!(
                compilation.status.success(),
                "fixture compilation failed: {}",
                String::from_utf8_lossy(&compilation.stderr)
            );
            output
        })
        .clone()
}

#[tokio::test]
async fn coordinator_continues_after_every_worker_failure_class() {
    let urls: Vec<String> = [
        "exit",
        "abort",
        "hang",
        "output",
        "malformed",
        "resource",
        "page_error",
        "blocked",
        "success",
    ]
    .into_iter()
    .map(|mode| format!("https://fixture.invalid/__fixture_{mode}__"))
    .collect();
    let options = CoverageOptions {
        concurrency: 1,
        timeout_ms: 1_000,
        worker_output_bytes: 4096,
        worker_executable: Some(fixture_path()),
        max_urls: None,
        ..CoverageOptions::default()
    };

    let report = tokio::time::timeout(Duration::from_secs(15), runner::run(&urls, &options))
        .await
        .expect("coverage coordinator exceeded its bounded deadline");
    assert_eq!(report.results.len(), urls.len());

    let result = |mode: &str| {
        report
            .results
            .iter()
            .find(|result| result.input_url.contains(&format!("__fixture_{mode}__")))
            .unwrap_or_else(|| panic!("missing result for {mode}"))
    };

    assert!(
        matches!(
            result("exit").failure_kind.as_ref(),
            Some(FailureKind::WorkerExit)
        ),
        "{:#?}",
        report.results
    );
    assert_eq!(result("exit").worker.as_ref().unwrap().exit_code, Some(23));
    assert!(matches!(
        result("abort").failure_kind.as_ref(),
        Some(FailureKind::WorkerCrash)
    ));
    assert!(result("abort").worker.as_ref().unwrap().signal.is_some() || cfg!(windows));
    assert!(matches!(
        result("hang").failure_kind.as_ref(),
        Some(FailureKind::Timeout)
    ));
    assert_eq!(
        result("output").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::OutputLimit
    );
    assert_eq!(
        result("malformed").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::MalformedOutput
    );
    assert!(matches!(
        result("resource").failure_kind.as_ref(),
        Some(FailureKind::WorkerResourceExhaustion)
    ));
    assert_eq!(
        result("resource").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::ResourceExhaustion
    );
    assert!(matches!(
        &result("page_error").status,
        CoverageStatus::Failed
    ));
    assert_eq!(
        result("page_error").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::PageError
    );
    assert!(matches!(&result("blocked").status, CoverageStatus::Blocked));
    assert_eq!(
        result("blocked").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::Blocked
    );
    assert!(matches!(&result("success").status, CoverageStatus::Ok));
    assert_eq!(
        result("success").worker.as_ref().unwrap().outcome,
        CoverageWorkerOutcome::Success
    );
    assert!(report.results.iter().all(|result| result
        .worker
        .as_ref()
        .is_some_and(|worker| worker.duration_ms < 10_000)));
}
