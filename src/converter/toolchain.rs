use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ToolchainStatus {
    pub kcc_cli_path: Option<PathBuf>,
    pub kindlegen_path: Option<PathBuf>,
}

impl ToolchainStatus {
    pub fn is_ready(&self) -> bool {
        self.kcc_cli_path.is_some() && self.kindlegen_path.is_some()
    }

    #[allow(dead_code)]
    pub fn check() -> Self {
        Self::check_with_custom(None)
    }

    pub fn check_with_custom(custom_kindlegen: Option<&Path>) -> Self {
        let kcc_cli = Self::find_executable("kcc-c2e");
        let kindlegen = Self::find_kindlegen(custom_kindlegen);

        Self {
            kcc_cli_path: kcc_cli,
            kindlegen_path: kindlegen,
        }
    }

    /// Finds kindlegen using:
    /// 1. Custom path explicitly configured by user in config.toml
    /// 2. KINDLEGEN_PATH environment variable
    /// 3. Kindle Previewer 3.app on macOS (standard modern location)
    /// 4. Standard Linux & Unix PATH / package manager locations
    fn find_kindlegen(custom_path: Option<&Path>) -> Option<PathBuf> {
        // 1. Explicit custom path from config.toml
        if let Some(path) = custom_path {
            if path.is_file() {
                return Some(path.to_path_buf());
            }
        }

        // 2. Environment variable KINDLEGEN_PATH
        if let Ok(env_path) = std::env::var("KINDLEGEN_PATH") {
            let p = PathBuf::from(env_path);
            if p.is_file() {
                return Some(p);
            }
        }

        // 3. Check inside Kindle Previewer 3.app (macOS official location)
        #[cfg(target_os = "macos")]
        {
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
        }

        // 4. Check standard executable locations & PATH (Linux / macOS / Windows)
        Self::find_executable("kindlegen")
    }

    fn find_executable(name: &str) -> Option<PathBuf> {
        // 1. Check via PATH
        if let Ok(path) = which::which(name) {
            return Some(path);
        }

        // 2. Standard unix / Linux / macOS directories
        let standard_dirs = [
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/opt/kindlegen"),
        ];

        for dir in &standard_dirs {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // 3. User local bin & Python virtualenvs (Linux / macOS)
        if let Some(home) = dirs::home_dir() {
            let user_candidates = [
                home.join(".local/bin").join(name),
                home.join("bin").join(name),
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
            missing.push("kindlegen (KindleGen)");
        }
        missing
    }

    pub fn install_instructions(&self) -> String {
        let mut instructions = Vec::new();

        if self.kcc_cli_path.is_none() {
            #[cfg(target_os = "macos")]
            instructions.push("• Kindle Comic Converter CLI (kcc-c2e):\n  Para conversão silenciosa em segundo plano no macOS:\n  brew install pipx && pipx install git+https://github.com/ciromattia/kcc.git");

            #[cfg(not(target_os = "macos"))]
            instructions.push("• Kindle Comic Converter CLI (kcc-c2e):\n  Instale o KCC CLI via pipx:\n  pipx install git+https://github.com/ciromattia/kcc.git\n  (ou: pip install --user git+https://github.com/ciromattia/kcc.git)");
        }

        if self.kindlegen_path.is_none() {
            #[cfg(target_os = "macos")]
            instructions.push("• KindleGen (via Kindle Previewer 3):\n  No macOS, instale o Kindle Previewer que contém o kindlegen embutido:\n  brew install --cask kindle-previewer");

            #[cfg(not(target_os = "macos"))]
            instructions.push("• KindleGen (Linux):\n  O kindlegen oficial foi descontinuado pela Amazon, mas ainda é utilizável no Linux:\n  - Arch Linux / Manjaro: yay -S kindlegen\n  - Debian / Ubuntu / Fedora: Baixe o kindlegen e coloque em ~/.local/bin/kindlegen (chmod +x)\n  - Ou configure o caminho no ~/.config/neo-mangal/config.toml:\n    kindlegen_path = \"/caminho/para/kindlegen\"");
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
        println!("Toolchain status: CLI={:?}, kindlegen={:?}", status.kcc_cli_path, status.kindlegen_path);
        let _ = status.missing_tools();
        let _ = status.install_instructions();
        let _ = status.is_ready();
    }

    #[test]
    fn test_toolchain_custom_kindlegen_path() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_test_kg");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let fake_kg = temp_dir.join("kindlegen");
        std::fs::write(&fake_kg, b"binary").unwrap();

        let status = ToolchainStatus::check_with_custom(Some(&fake_kg));
        assert_eq!(status.kindlegen_path, Some(fake_kg.clone()));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
