use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use crate::config::Config;
use crate::converter::toolchain::ToolchainStatus;
use crate::error::{NeoError, Result};

pub struct KccRunner;

impl KccRunner {
    /// Runs kcc-c2e or launches Kindle Comic Converter.app with the CBZ preloaded
    pub async fn convert(
        cbz_path: &Path,
        output_dir: &Path,
        config: &Config,
        progress_tx: Option<mpsc::Sender<String>>,
    ) -> Result<PathBuf> {
        let toolchain = ToolchainStatus::check();

        // 1. If CLI kcc-c2e is available, run full automated headless conversion
        if let Some(kcc_binary) = toolchain.kcc_cli_path {
            let mut cmd = Command::new(kcc_binary);
            let kcc_arg_format = match config.kcc_format.to_uppercase().as_str() {
                "AZW3" | "MOBI" => "MOBI",
                "EPUB" => "EPUB",
                "KFX" => "KFX",
                _ => "MOBI",
            };

            cmd.arg("-p")
                .arg(&config.kcc_profile)
                .arg("-f")
                .arg(kcc_arg_format)
                .arg("-o")
                .arg(output_dir);

            if config.kcc_manga_style {
                cmd.arg("-m");
            }

            cmd.arg(cbz_path);

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd
                .spawn()
                .map_err(|e| NeoError::Kcc(format!("Failed to spawn kcc-c2e: {}", e)))?;

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let tx_out = progress_tx.clone();
            let stdout_handle = tokio::spawn(async move {
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            if let Some(ref tx) = tx_out {
                                let _ = tx.send(trimmed.to_string()).await;
                            }
                        }
                    }
                }
            });

            let tx_err = progress_tx.clone();
            let stderr_handle = tokio::spawn(async move {
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            if let Some(ref tx) = tx_err {
                                let _ = tx.send(format!("ERR: {}", trimmed)).await;
                            }
                        }
                    }
                }
            });

            let status = child
                .wait()
                .await
                .map_err(|e| NeoError::Kcc(format!("kcc-c2e process error: {}", e)))?;

            let _ = stdout_handle.await;
            let _ = stderr_handle.await;

            if !status.success() {
                return Err(NeoError::Kcc(format!(
                    "kcc-c2e exited with code: {:?}",
                    status.code()
                )));
            }

            let stem = cbz_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            let final_path = Self::post_process_kcc_output(output_dir, stem, config);
            return Ok(final_path);
        }

        // 2. If kcc-c2e is not in PATH, but Kindle Comic Converter.app exists, open with app
        if let Some(_app_path) = toolchain.kcc_app_path {
            if let Some(ref tx) = progress_tx {
                let _ = tx
                    .send("Abrindo arquivo CBZ no Kindle Comic Converter.app...".to_string())
                    .await;
            }

            let status = Command::new("open")
                .arg("-a")
                .arg("Kindle Comic Converter")
                .arg(cbz_path)
                .status()
                .await
                .map_err(|e| NeoError::Kcc(format!("Failed to open KCC app: {}", e)))?;

            if status.success() {
                return Ok(cbz_path.to_path_buf());
            } else {
                return Err(NeoError::Kcc(
                    "Falha ao abrir o arquivo no Kindle Comic Converter.app".to_string(),
                ));
            }
        }

        Err(NeoError::ToolchainMissing(
            "Kindle Comic Converter não foi encontrado no sistema.".to_string(),
        ))
    }

    /// Post-processes KCC output, automatically renaming .mobi to .azw3 for best Kindle KF8 quality
    pub fn post_process_kcc_output(output_dir: &Path, stem: &str, config: &Config) -> PathBuf {
        let should_rename_azw3 = config.rename_to_azw3
            || config.kcc_format.eq_ignore_ascii_case("AZW3")
            || config.kcc_format.eq_ignore_ascii_case("MOBI");

        if should_rename_azw3 {
            let target_azw3 = output_dir.join(format!("{}.azw3", stem));
            let expected_mobi = output_dir.join(format!("{}.mobi", stem));

            if expected_mobi.exists() {
                let _ = std::fs::rename(&expected_mobi, &target_azw3);
                return target_azw3;
            }

            // If exact file not matched yet, check directory in case KCC sanitized the stem
            if let Ok(entries) = std::fs::read_dir(output_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext.eq_ignore_ascii_case("mobi") {
                                if let Some(s) = path.file_stem().and_then(|s| s.to_str()) {
                                    if s == stem || s.starts_with(stem) {
                                        let azw3 = output_dir.join(format!("{}.azw3", s));
                                        let _ = std::fs::rename(&path, &azw3);
                                        return azw3;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if target_azw3.exists() {
                return target_azw3;
            }
        }

        let ext = match config.kcc_format.to_uppercase().as_str() {
            "EPUB" => "epub",
            "KFX" => "kfx",
            _ => "mobi",
        };
        output_dir.join(format!("{}.{}", stem, ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_post_process_kcc_output_renames_mobi_to_azw3() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("neo_mangal_test_kcc_{}", nanos));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mobi_file = temp_dir.join("Chapter_01.mobi");
        File::create(&mobi_file).unwrap();

        let config = Config::default(); // default format is AZW3 and rename_to_azw3 = true
        let final_path = KccRunner::post_process_kcc_output(&temp_dir, "Chapter_01", &config);

        assert_eq!(final_path.extension().and_then(|e| e.to_str()), Some("azw3"));
        assert!(final_path.exists());
        assert!(!mobi_file.exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
