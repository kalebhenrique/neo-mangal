use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ToolchainStatus {
    pub kcc_cli_path: Option<PathBuf>,
    pub kcc_app_path: Option<PathBuf>,
    pub kindlegen_path: Option<PathBuf>,
}

impl ToolchainStatus {
    pub fn is_ready(&self) -> bool {
        self.kcc_cli_path.is_some() && self.kindlegen_path.is_some()
    }

    #[allow(dead_code)]
    pub fn has_gui_app(&self) -> bool {
        self.kcc_app_path.is_some()
    }

    pub fn check() -> Self {
        let kcc_cli = Self::find_executable("kcc-c2e");
        let kcc_app = Self::find_kcc_app();
        let kindlegen = Self::find_kindlegen();

        Self {
            kcc_cli_path: kcc_cli,
            kcc_app_path: kcc_app,
            kindlegen_path: kindlegen,
        }
    }

    fn find_kcc_app() -> Option<PathBuf> {
        let app_paths = [
            PathBuf::from("/Applications/Kindle Comic Converter.app"),
            dirs::home_dir().map(|h| h.join("Applications/Kindle Comic Converter.app")).unwrap_or_default(),
        ];

        for path in &app_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }
        None
    }

    /// Finds kindlegen, automatically checking inside Kindle Previewer 3.app on macOS
    fn find_kindlegen() -> Option<PathBuf> {
        // 1. Check inside Kindle Previewer 3.app (the official modern way to get kindlegen on macOS)
        let previewer_paths = [
            PathBuf::from("/Applications/Kindle Previewer 3.app/Contents/lib/fc/bin/kindlegen"),
            PathBuf::from("/Applications/Kindle Previewer 3.app/Contents/MacOS/kindlegen"),
            dirs::home_dir().map(|h| h.join("Applications/Kindle Previewer 3.app/Contents/lib/fc/bin/kindlegen")).unwrap_or_default(),
        ];

        for path in &previewer_paths {
            if path.is_file() {
                return Some(path.clone());
            }
        }

        // 2. Check standard executable locations & PATH
        Self::find_executable("kindlegen")
    }

    fn find_executable(name: &str) -> Option<PathBuf> {
        // 1. Check via PATH
        if let Ok(path) = which::which(name) {
            return Some(path);
        }

        // 2. Standard unix / macOS directories
        let standard_dirs = [
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/bin"),
        ];

        for dir in &standard_dirs {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // 3. User local bin & Python virtualenvs
        if let Some(home) = dirs::home_dir() {
            let user_candidates = [
                home.join(".local/bin").join(name),
                home.join(".cargo/bin").join(name),
                home.join(".kcc-venv/bin").join(name),
            ];

            for candidate in &user_candidates {
                if candidate.is_file() {
                    return Some(candidate.clone());
                }
            }

            // Check ~/Library/Python/*/bin (macOS pip user installs)
            let python_lib = home.join("Library/Python");
            if python_lib.is_dir() {
                for entry in WalkDir::new(&python_lib).max_depth(3).into_iter().flatten() {
                    if entry.file_name() == name && entry.file_type().is_file() {
                        return Some(entry.into_path());
                    }
                }
            }
        }

        None
    }

    pub fn missing_tools(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.kcc_cli_path.is_none() {
            missing.push("kcc-c2e (KCC CLI)");
        }
        if self.kindlegen_path.is_none() {
            missing.push("kindlegen (Kindle Previewer)");
        }
        missing
    }

    pub fn install_instructions(&self) -> String {
        let mut instructions = Vec::new();

        if self.kcc_cli_path.is_none() {
            instructions.push("• Kindle Comic Converter CLI (kcc-c2e):\n  O pacote DMG do macOS traz apenas a interface gráfica. Para conversão 100% silenciosa em segundo plano:\n  brew install pipx && pipx install git+https://github.com/ciromattia/kcc.git");
        }

        if self.kindlegen_path.is_none() {
            instructions.push("• Kindle Previewer 3 (Contém o KindleGen embutido):\n  O pacote kindlegen standalone foi descontinuado pela Amazon. O neo-mangal detecta o KindleGen embutido no Kindle Previewer:\n  brew install --cask kindle-previewer");
        }

        instructions.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_check_does_not_panic() {
        let status = ToolchainStatus::check();
        println!("Toolchain status: CLI={:?}, App={:?}, kindlegen={:?}", status.kcc_cli_path, status.kcc_app_path, status.kindlegen_path);
        assert!(status.has_gui_app(), "Kindle Comic Converter GUI app should be detected on this machine");
        assert!(status.kindlegen_path.is_some(), "kindlegen should be detected on this machine");
    }
}
