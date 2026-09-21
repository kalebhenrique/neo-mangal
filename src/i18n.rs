#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLanguage {
    English,
    Portuguese,
}

impl AppLanguage {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "pt" | "pt-br" | "pt_br" | "portuguese" => AppLanguage::Portuguese,
            _ => AppLanguage::English,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppLanguage::English => "en",
            AppLanguage::Portuguese => "pt-br",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AppLanguage::English => "English (Default)",
            AppLanguage::Portuguese => "Português (Brasil)",
        }
    }
}

pub struct I18n;

impl I18n {
    // Header
    pub fn status_ready(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " [READY] ",
            AppLanguage::Portuguese => " [PRONTO] ",
        }
    }

    pub fn status_busy(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " [BUSY] ",
            AppLanguage::Portuguese => " [OCUPADO] ",
        }
    }

    // Search bar
    pub fn search_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " Search Manga ",
            AppLanguage::Portuguese => " Buscar Mangá ",
        }
    }

    pub fn search_placeholder(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Type manga name and press Enter (e.g. 'One Piece', 'Berserk')...",
            AppLanguage::Portuguese => "Digite o nome do mangá e pressione Enter (ex: 'One Piece', 'Berserk')...",
        }
    }

    // Manga list
    pub fn series_found(lang: AppLanguage, count: usize) -> String {
        match lang {
            AppLanguage::English => format!(" Series Found ({}) ", count),
            AppLanguage::Portuguese => format!(" Mangás Encontrados ({}) ", count),
        }
    }

    // Chapter list
    pub fn filter_chapters_title(lang: AppLanguage, title: &str) -> String {
        match lang {
            AppLanguage::English => format!(" Filter Chapters for: \"{}\" ", title),
            AppLanguage::Portuguese => format!(" Filtrar Capítulos de: \"{}\" ", title),
        }
    }

    pub fn filter_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Filter: ",
            AppLanguage::Portuguese => "Filtro: ",
        }
    }

    pub fn pagination_info(lang: AppLanguage, current: usize, total: usize, matching: usize) -> String {
        match lang {
            AppLanguage::English => format!("Page {}/{} ({} matching)", current, total, matching),
            AppLanguage::Portuguese => format!("Página {}/{} ({} correspondentes)", current, total, matching),
        }
    }

    pub fn pagination_hint(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "   [← / → to switch page]",
            AppLanguage::Portuguese => "   [← / → para mudar de página]",
        }
    }

    pub fn chapters_box_title(lang: AppLanguage, count: usize) -> String {
        match lang {
            AppLanguage::English => format!(" Chapters ({}) ", count),
            AppLanguage::Portuguese => format!(" Capítulos ({}) ", count),
        }
    }

    // Progress bar
    pub fn progress_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " Progress ",
            AppLanguage::Portuguese => " Progresso ",
        }
    }

    pub fn progress_ready(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Ready",
            AppLanguage::Portuguese => "Pronto",
        }
    }

    pub fn progress_downloading(lang: AppLanguage, current: usize, total: usize, pct: u16) -> String {
        match lang {
            AppLanguage::English => format!("Downloading pages {}/{} ({}%)", current, total, pct),
            AppLanguage::Portuguese => format!("Baixando páginas {}/{} ({}%)", current, total, pct),
        }
    }

    pub fn progress_packaging(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Packaging CBZ archive...",
            AppLanguage::Portuguese => "Empacotando arquivo CBZ...",
        }
    }

    pub fn progress_packaging_pdf(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Generating PDF document...",
            AppLanguage::Portuguese => "Gerando documento PDF...",
        }
    }

    pub fn progress_converting(lang: AppLanguage, msg: &str) -> String {
        match lang {
            AppLanguage::English => format!("KCC Converting: {}", msg),
            AppLanguage::Portuguese => format!("Convertendo KCC: {}", msg),
        }
    }

    pub fn progress_done(lang: AppLanguage, msg: &str) -> String {
        match lang {
            AppLanguage::English => format!("Done: {}", msg),
            AppLanguage::Portuguese => format!("Concluído: {}", msg),
        }
    }

    pub fn progress_error(lang: AppLanguage, err: &str) -> String {
        match lang {
            AppLanguage::English => format!("Error: {}", err),
            AppLanguage::Portuguese => format!("Erro: {}", err),
        }
    }

    // Settings Modal
    pub fn settings_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " ⚙ Settings ",
            AppLanguage::Portuguese => " ⚙ Configurações ",
        }
    }

    pub fn download_dir_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " Download / Output Directory ",
            AppLanguage::Portuguese => " Diretório de Download / Saída ",
        }
    }

    pub fn dir_valid(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "✓ Accessible directory found",
            AppLanguage::Portuguese => "✓ Diretório existente e acessível",
        }
    }

    pub fn dir_will_create(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "ℹ New directory: will be created automatically upon download",
            AppLanguage::Portuguese => "ℹ Novo diretório: será criado automaticamente ao baixar",
        }
    }

    pub fn dir_empty_err(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "✗ Path cannot be empty",
            AppLanguage::Portuguese => "✗ Caminho não pode estar vazio",
        }
    }

    pub fn lang_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " Interface Language [Press 'l' or Tab to switch] ",
            AppLanguage::Portuguese => " Idioma da Interface [Pressione 'l' ou Tab para alternar] ",
        }
    }

    pub fn lang_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Active Language: ",
            AppLanguage::Portuguese => "Idioma Ativo: ",
        }
    }

    pub fn kcc_profile_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " KCC Device Profile [Press 'p' to switch] ",
            AppLanguage::Portuguese => " Perfil de Dispositivo KCC [Pressione 'p' para alternar] ",
        }
    }

    pub fn model_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Model: ",
            AppLanguage::Portuguese => "Modelo: ",
        }
    }

    pub fn format_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "  Format: ",
            AppLanguage::Portuguese => "  Formato: ",
        }
    }

    pub fn settings_instructions(lang: AppLanguage) -> Vec<(&'static str, &'static str)> {
        match lang {
            AppLanguage::English => vec![
                ("[p] ", "Kindle Model   "),
                ("[o] ", "Format   "),
                ("[l] ", "Language   "),
                ("[Enter] ", "Save & Persist   "),
                ("[Esc] ", "Cancel"),
            ],
            AppLanguage::Portuguese => vec![
                ("[p] ", "Modelo Kindle   "),
                ("[o] ", "Formato   "),
                ("[l] ", "Idioma   "),
                ("[Enter] ", "Salvar e Persistir   "),
                ("[Esc] ", "Cancelar"),
            ],
        }
    }

    // AniList Integration
    pub fn anilist_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " AniList Sync ",
            AppLanguage::Portuguese => " Sincronização AniList ",
        }
    }

    pub fn anilist_status_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Status: ",
            AppLanguage::Portuguese => "Status: ",
        }
    }

    pub fn anilist_connected_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "[ Connected ]",
            AppLanguage::Portuguese => "[ Conectado ]",
        }
    }

    pub fn anilist_disconnected_label(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "[ Disconnected ]",
            AppLanguage::Portuguese => "[ Desconectado ]",
        }
    }

    pub fn anilist_connect_prompt(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " [a] Connect / Login ",
            AppLanguage::Portuguese => " [a] Conectar / Login ",
        }
    }

    pub fn anilist_disconnect_prompt(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " [d] Disconnect ",
            AppLanguage::Portuguese => " [d] Desconectar ",
        }
    }

    // Cover Prompt Modal
    pub fn cover_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " 🖼 Custom Cover ",
            AppLanguage::Portuguese => " 🖼 Capa Customizada ",
        }
    }

    pub fn cover_question(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Would you like to add a custom cover for ",
            AppLanguage::Portuguese => "Deseja adicionar uma capa customizada para ",
        }
    }

    pub fn volume_fusion_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " 📚 Volume Fusion ",
            AppLanguage::Portuguese => " 📚 Fusão de Volume ",
        }
    }

    pub fn volume_fusion_question(lang: AppLanguage, count: usize) -> String {
        match lang {
            AppLanguage::English => format!(
                "Fuse {} selected chapters into a single Volume archive? Drop a Volume Cover below:",
                count
            ),
            AppLanguage::Portuguese => format!(
                "Fundir {} capítulos selecionados em um único arquivo de Volume? Solte a Capa do Volume abaixo:",
                count
            ),
        }
    }

    pub fn cover_instructions(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Drag and drop an image file directly here, or paste/type the path:",
            AppLanguage::Portuguese => "Arraste e solte o arquivo de imagem diretamente aqui, ou cole/digite o caminho:",
        }
    }

    pub fn cover_input_box(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => " [Drag & Drop Image or Enter Path] ",
            AppLanguage::Portuguese => " [Arraste e Solte a Imagem ou Digite o Caminho] ",
        }
    }

    pub fn cover_placeholder(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Drop file here (e.g. /path/to/cover.jpg)...",
            AppLanguage::Portuguese => "Solte o arquivo aqui (ex: /caminho/da/capa.jpg)...",
        }
    }

    pub fn cover_valid(lang: AppLanguage, path: &str) -> String {
        match lang {
            AppLanguage::English => format!("Valid image detected: {}", path),
            AppLanguage::Portuguese => format!("Imagem válida detectada: {}", path),
        }
    }

    pub fn cover_none(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "ℹ No cover selected. KCC will use default chapter page 1.",
            AppLanguage::Portuguese => "ℹ Nenhuma capa selecionada. O KCC usará a 1ª página padrão.",
        }
    }

    pub fn cover_invalid(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "File not found or unsupported format (.jpg, .png, .webp)",
            AppLanguage::Portuguese => "Arquivo não encontrado ou formato não suportado (.jpg, .png, .webp)",
        }
    }

    pub fn cover_actions(lang: AppLanguage) -> Vec<(&'static str, &'static str)> {
        match lang {
            AppLanguage::English => vec![
                ("[Enter] ", "Confirm with Cover   "),
                ("[n] / [s] ", "Skip Cover (Use Default)   "),
                ("[Esc] ", "Cancel"),
            ],
            AppLanguage::Portuguese => vec![
                ("[Enter] ", "Confirmar com Capa   "),
                ("[n] / [s] ", "Pular Capa (Usar Padrão)   "),
                ("[Esc] ", "Cancelar"),
            ],
        }
    }

    // Alerts
    pub fn no_results_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "No Results",
            AppLanguage::Portuguese => "Nenhum Resultado",
        }
    }

    pub fn no_results_msg(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "No manga found for this search query.",
            AppLanguage::Portuguese => "Nenhum mangá encontrado para esta busca.",
        }
    }

    pub fn search_error_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Search Error",
            AppLanguage::Portuguese => "Erro na Busca",
        }
    }

    pub fn no_chapters_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "No Chapters",
            AppLanguage::Portuguese => "Sem Capítulos",
        }
    }

    pub fn no_chapters_msg(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "No chapters found for this series.",
            AppLanguage::Portuguese => "Não foram encontrados capítulos para este mangá.",
        }
    }

    pub fn chapters_error_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Error Loading Chapters",
            AppLanguage::Portuguese => "Erro ao Carregar Capítulos",
        }
    }

    pub fn success_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Success!",
            AppLanguage::Portuguese => "Sucesso!",
        }
    }

    pub fn operation_error_title(lang: AppLanguage) -> &'static str {
        match lang {
            AppLanguage::English => "Operation Error",
            AppLanguage::Portuguese => "Erro na Operação",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_languages() {
        assert_eq!(AppLanguage::from_code("en"), AppLanguage::English);
        assert_eq!(AppLanguage::from_code("pt-br"), AppLanguage::Portuguese);
        assert_eq!(AppLanguage::from_code("pt"), AppLanguage::Portuguese);
        assert_eq!(AppLanguage::from_code("random"), AppLanguage::English); // Fallback to English default

        assert_eq!(I18n::search_title(AppLanguage::English), " Search Manga ");
        assert_eq!(I18n::search_title(AppLanguage::Portuguese), " Buscar Mangá ");

        assert_eq!(I18n::status_ready(AppLanguage::English), " [READY] ");
        assert_eq!(I18n::status_ready(AppLanguage::Portuguese), " [PRONTO] ");
    }
}
