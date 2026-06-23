//! Integration tests for the `sqlx` tracing-layer front-end.
//!
//! These run the `test-sqlx-08` and `test-sqlx-09` `basic` examples as
//! subprocesses and assert on their reports. The same hotpath layer feeds both:
//! the `sqlx::query` event field schema (`db.statement` / `summary` /
//! `elapsed_secs`) is identical across sqlx 0.8 and 0.9, so a single layer
//! covers both. These tests also pin that schema - a sqlx upgrade that renames
//! or drops those fields would empty the SQL report and fail here.
#[cfg(test)]
pub mod tests {
    use std::process::Command;

    fn run_basic(package: &str, format: Option<&str>) -> String {
        let mut cmd = Command::new("cargo");
        cmd.args([
            "run",
            "-p",
            package,
            "--example",
            "basic",
            "--features",
            "hotpath",
        ]);
        if let Some(fmt) = format {
            cmd.env("HOTPATH_OUTPUT_FORMAT", fmt);
        }
        let output = cmd.output().expect("Failed to execute command");
        assert!(
            output.status.success(),
            "Command failed with status: {}",
            output.status
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn assert_table_output(package: &str, completion_msg: &str) {
        let stdout = run_basic(package, None);

        let all_expected = [
            completion_msg,
            "sql - SQL query execution time statistics.",
            "INSERT INTO users (name, age) VALUES (?, ?)",
            // Short query (4 words) arrives via `summary`, not `db.statement`.
            "SELECT COUNT(*) FROM users",
            // Inline literals normalized into one bucket.
            "SELECT name FROM users WHERE age = ?",
            // Different-arity IN lists collapse to one bucket.
            "SELECT * FROM users WHERE id IN (?)",
        ];
        for expected in all_expected {
            assert!(
                stdout.contains(expected),
                "[{package}] Expected:\n{expected}\n\nGot:\n{stdout}",
            );
        }
    }

    fn assert_transaction_queries_captured(package: &str) {
        // 50 loop inserts + 1 transaction-internal insert = 51. A pool wrapper
        // would miss the transaction-internal query; the layer captures it.
        let stdout = run_basic(package, Some("json"));

        let all_expected = [
            "\"sql\"",
            "\"INSERT INTO users (name, age) VALUES (?, ?)\"",
            "\"count\":51",
        ];
        for expected in all_expected {
            assert!(
                stdout.contains(expected),
                "[{package}] Expected:\n{expected}\n\nGot:\n{stdout}",
            );
        }
    }

    #[test]
    fn test_table_output_sqlx_08() {
        assert_table_output("test-sqlx-08", "sqlx 0.8 tracing-layer example completed!");
    }

    #[test]
    fn test_table_output_sqlx_09() {
        assert_table_output("test-sqlx-09", "sqlx 0.9 tracing-layer example completed!");
    }

    #[test]
    fn test_transaction_queries_captured_sqlx_08() {
        assert_transaction_queries_captured("test-sqlx-08");
    }

    #[test]
    fn test_transaction_queries_captured_sqlx_09() {
        assert_transaction_queries_captured("test-sqlx-09");
    }
}
