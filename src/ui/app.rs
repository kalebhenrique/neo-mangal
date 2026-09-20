use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
    Frame,
};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::converter::cbz::CbzPacker;
use crate::converter::kcc::KccRunner;
use crate::converter::toolchain::ToolchainStatus;
use crate::domain::favorite::{FavoriteManga, FavoriteManager};
use crate::domain::job::JobStatus;
use crate::domain::manga::{Chapter, Manga};
use crate::downloader::ChapterDownloader;
use crate::error::NeoError;
use crate::i18n::{AppLanguage, I18n};
use crate::ui::components::{
    ChapterListComponent, FavoritesListComponent, HeaderComponent, MangaListComponent,
    ProgressBarComponent, SearchBarComponent, StatusBarComponent,
};
use crate::scraper::manager::SourceManager;
use crate::ui::events::AppEvent;
use crate::ui::modals::{
    AlertModal, ConfirmRemoveModal, InstallSourcesModal, KccMissingModal, ProcessModal,
    SettingsModal, SourceSelectModal,
};

#[derive(Debug, PartialEq, Eq)]
pub enum CurrentView {
    Favorites,       // Browsing favorites on home screen
    Search,          // Typing inside search input
    SearchUnfocused, // Search input unfocused (can press 's' for settings, 'q' to quit)
    MangaResults,    // Browsing manga search results
    ChapterList,     // Browsing chapter list with pagination & filter
}

pub enum ModalState {
    Settings(SettingsModal),
    Process(ProcessModal),
    KccMissing(KccMissingModal),
    Alert(AlertModal),
    InstallSources(InstallSourcesModal),
    SourceSelect(SourceSelectModal),
    ConfirmRemove(ConfirmRemoveModal),
}

pub struct App {
    pub config: Config,
    pub current_view: CurrentView,
    pub active_modal: Option<ModalState>,
    pub available_sources: Vec<String>,
    pub active_sources: Vec<String>,
    pub active_source: String,

    // Favorites data
    pub favorites: Vec<FavoriteManga>,
    pub favorites_list_state: ListState,

    // Search and Manga data
    pub search_input: String,
    pub mangas: Vec<Manga>,
    pub manga_list_state: ListState,
    pub selected_manga: Option<Manga>,

    // Chapter data & pagination & filter
    pub chapters: Vec<Chapter>,
    pub chapter_list_state: ListState,
    pub selected_chapter_ids: HashSet<String>,
    pub chapter_filter: String,
    pub is_chapter_filter_active: bool,
    pub chapter_page: usize,
    pub chapter_page_size: usize,

    // App state
    pub is_busy: bool,
    pub job_status: JobStatus,
    pub should_quit: bool,
    pub previous_view: Option<CurrentView>,

    // Async task event sender
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl App {
    pub fn new(config: Config, event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        let available_sources = SourceManager::list_sources();
        let active_sources = if !config.active_sources.is_empty() {
            let valid: Vec<String> = config
                .active_sources
                .iter()
                .filter(|s| available_sources.contains(s))
                .cloned()
                .collect();
            if valid.is_empty() {
                available_sources.clone()
            } else {
                valid
            }
        } else {
            available_sources.clone()
        };
        let active_source = if active_sources.is_empty() {
            "None".to_string()
        } else if active_sources.len() == 1 {
            active_sources[0].clone()
        } else if active_sources.len() == 2 {
            format!("{} + {}", active_sources[0], active_sources[1])
        } else {
            format!("Multi ({})", active_sources.len())
        };

        let active_modal = if available_sources.is_empty() {
            Some(ModalState::InstallSources(InstallSourcesModal::new()))
        } else {
            None
        };

        let favorites = FavoriteManager::load_favorites();
        let mut favorites_list_state = ListState::default();
        let current_view = if !favorites.is_empty() {
            favorites_list_state.select(Some(0));
            CurrentView::Favorites
        } else {
            CurrentView::Search
        };

        Self {
            config,
            current_view,
            active_modal,
            available_sources,
            active_sources,
            active_source,
            favorites,
            favorites_list_state,
            search_input: String::new(),
            mangas: Vec::new(),
            manga_list_state: ListState::default(),
            selected_manga: None,
            chapters: Vec::new(),
            chapter_list_state: ListState::default(),
            selected_chapter_ids: HashSet::new(),
            chapter_filter: String::new(),
            is_chapter_filter_active: false,
            chapter_page: 0,
            chapter_page_size: 20,
            is_busy: false,
            job_status: JobStatus::Idle,
            should_quit: false,
            previous_view: None,
            event_tx,
        }
    }

    pub fn update_active_source_label(&mut self) {
        if self.active_sources.is_empty() {
            self.active_source = "None".to_string();
        } else if self.active_sources.len() == 1 {
            self.active_source = self.active_sources[0].clone();
        } else if self.active_sources.len() == 2 {
            self.active_source = format!("{} + {}", self.active_sources[0], self.active_sources[1]);
        } else {
            self.active_source = format!("Multi ({})", self.active_sources.len());
        }
    }

    pub fn current_language(&self) -> AppLanguage {
        AppLanguage::from_code(&self.config.language)
    }

    /// Returns chapters matching current inline chapter filter
    pub fn filtered_chapters(&self) -> Vec<&Chapter> {
        if self.chapter_filter.trim().is_empty() {
            self.chapters.iter().collect()
        } else {
            let q = self.chapter_filter.to_lowercase();
            self.chapters
                .iter()
                .filter(|c| c.title.to_lowercase().contains(&q))
                .collect()
        }
    }

    /// Total pages for chapter pagination
    pub fn total_chapter_pages(&self) -> usize {
        let total = self.filtered_chapters().len();
        if total == 0 {
            1
        } else {
            (total + self.chapter_page_size - 1) / self.chapter_page_size
        }
    }

    /// Returns the slice of chapters on the current page
    pub fn current_page_chapters(&self) -> Vec<Chapter> {
        let filtered = self.filtered_chapters();
        let start = self.chapter_page * self.chapter_page_size;
        filtered
            .into_iter()
            .skip(start)
            .take(self.chapter_page_size)
            .cloned()
            .collect()
    }

    /// Process events received by the main loop
    pub fn handle_event(&mut self, event: AppEvent) {
        let lang = self.current_language();
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::Paste(text) => self.handle_paste(&text),
            AppEvent::Tick => {}
            AppEvent::SearchResults(res) => {
                self.is_busy = false;
                match res {
                    Ok(list) => {
                        self.mangas = list;
                        if !self.mangas.is_empty() {
                            self.manga_list_state.select(Some(0));
                            self.current_view = CurrentView::MangaResults;
                        } else {
                            self.current_view = CurrentView::SearchUnfocused;
                            self.active_modal = Some(ModalState::Alert(AlertModal::new(
                                I18n::no_results_title(lang),
                                I18n::no_results_msg(lang),
                                false,
                            )));
                        }
                    }
                    Err(e) => {
                        self.current_view = CurrentView::SearchUnfocused;
                        self.active_modal = Some(ModalState::Alert(AlertModal::new(
                            I18n::search_error_title(lang),
                            &format!("{}", e),
                            true,
                        )));
                    }
                }
            }
            AppEvent::ChaptersResults(res) => {
                self.is_busy = false;
                match res {
                    Ok(list) => {
                        self.chapters = list;
                        self.selected_chapter_ids.clear();
                        self.chapter_filter.clear();
                        self.is_chapter_filter_active = false;
                        self.chapter_page = 0;
                        if !self.chapters.is_empty() {
                            self.chapter_list_state.select(Some(0));
                            self.current_view = CurrentView::ChapterList;
                        } else {
                            self.active_modal = Some(ModalState::Alert(AlertModal::new(
                                I18n::no_chapters_title(lang),
                                I18n::no_chapters_msg(lang),
                                false,
                            )));
                        }
                    }
                    Err(e) => {
                        self.active_modal = Some(ModalState::Alert(AlertModal::new(
                            I18n::chapters_error_title(lang),
                            &format!("{}", e),
                            true,
                        )));
                    }
                }
            }
            AppEvent::DownloadStatus(status) => {
                self.job_status = status;
            }
            AppEvent::KccStatus(msg) => {
                self.job_status = JobStatus::ConvertingKcc { message: msg };
            }
            AppEvent::OperationSuccess(msg) => {
                self.is_busy = false;
                self.job_status = JobStatus::Done(msg.clone());
                self.active_modal = Some(ModalState::Alert(AlertModal::new(
                    I18n::success_title(lang),
                    &msg,
                    false,
                )));
            }
            AppEvent::OperationError(err) => {
                self.is_busy = false;
                self.job_status = JobStatus::Failed(err.clone());
                self.active_modal = Some(ModalState::Alert(AlertModal::new(
                    I18n::operation_error_title(lang),
                    &err,
                    true,
                )));
            }
            AppEvent::ChapterDownloaded {
                manga_url,
                chapter_title,
            } => {
                FavoriteManager::update_last_downloaded(
                    &mut self.favorites,
                    &manga_url,
                    &chapter_title,
                );
            }
        }
    }

    /// Handles bracketed paste (e.g. file drag & drop into terminal)
    fn handle_paste(&mut self, text: &str) {
        if let Some(ModalState::Process(ref mut modal)) = self.active_modal {
            modal.handle_paste(text);
        }
    }

    /// Handles keyboard events based on active modal or current view
    fn handle_key(&mut self, key: KeyEvent) {
        // Global quit on Ctrl+C or Ctrl+Q
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('q'))
        {
            self.should_quit = true;
            return;
        }

        // Global settings shortcut (F2 or Ctrl+S) available anytime
        if key.code == KeyCode::F(2)
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s'))
        {
            self.active_modal = Some(ModalState::Settings(SettingsModal::new(&self.config)));
            return;
        }

        // Global source selection shortcut: Ctrl+P available anytime
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P'))
        {
            if !self.available_sources.is_empty() {
                self.active_modal = Some(ModalState::SourceSelect(SourceSelectModal::new(
                    self.available_sources.clone(),
                    &self.active_sources,
                )));
                return;
            }
        }

        // 1. Delegate to active modal if any
        if let Some(ref mut modal) = self.active_modal {
            match modal {
                ModalState::Settings(s) => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        let new_path = s.input_path.trim();
                        if !new_path.is_empty() {
                            let _ = self.config.update_download_dir(new_path);
                        }
                        self.config.language = s.language.clone();
                        self.config.kcc_profile = s.profile.clone();
                        self.config.kcc_format = s.format.clone();
                        let _ = self.config.save();
                        self.active_modal = None;
                    }
                    KeyCode::Tab | KeyCode::Char('p') => {
                        s.cycle_profile();
                    }
                    KeyCode::Char('o') => {
                        s.cycle_format();
                    }
                    KeyCode::Char('l') => {
                        s.cycle_language();
                    }
                    KeyCode::Backspace => {
                        s.handle_backspace();
                    }
                    KeyCode::Char(c) => {
                        s.handle_char(c);
                    }
                    _ => {}
                },
                ModalState::Process(p) => match key.code {
                    KeyCode::Esc => {
                        self.config.kcc_format = p.format.clone();
                        let _ = self.config.save();
                        self.active_modal = None;
                    }
                    KeyCode::Char('o') => {
                        p.cycle_format();
                        self.config.kcc_format = p.format.clone();
                        let _ = self.config.save();
                    }
                    KeyCode::Char('c') => {
                        p.toggle_kcc();
                    }
                    KeyCode::Char('f') => {
                        p.toggle_fusion();
                    }
                    KeyCode::Backspace => {
                        p.handle_backspace();
                    }
                    KeyCode::Char(c) => {
                        p.handle_char(c);
                    }
                    KeyCode::Enter => {
                        let is_kcc = p.is_kcc();
                        let selected_format = p.format.clone();
                        let fuse_volume = p.fuse_volume;
                        let cover_opt = p.get_validated_path();

                        self.config.kcc_format = selected_format;
                        let _ = self.config.save();

                        if is_kcc {
                            let status = ToolchainStatus::check();
                            if !status.is_ready() {
                                self.active_modal = Some(ModalState::KccMissing(KccMissingModal::new(status)));
                                return;
                            }
                        }

                        self.active_modal = None;

                        if fuse_volume {
                            self.start_fusion_processing(cover_opt, is_kcc);
                        } else {
                            self.start_batch_processing(cover_opt, is_kcc);
                        }
                    }
                    _ => {}
                },
                ModalState::KccMissing(_) | ModalState::Alert(_) => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.active_modal = None;
                    }
                    _ => {}
                },
                ModalState::InstallSources(im) => match (&im.state, key.code) {
                    (crate::ui::modals::install_sources::InstallState::Prompt, KeyCode::Enter) => {
                        if let Ok(sources) = im.execute_install() {
                            self.available_sources = sources;
                            self.active_sources = self.available_sources.clone();
                            self.update_active_source_label();
                        }
                    }
                    (crate::ui::modals::install_sources::InstallState::Prompt, KeyCode::Esc) => {
                        self.active_modal = None;
                    }
                    (crate::ui::modals::install_sources::InstallState::Success(_), KeyCode::Enter) => {
                        if !self.available_sources.is_empty() {
                            self.active_sources = self.available_sources.clone();
                            self.update_active_source_label();
                            self.active_modal = Some(ModalState::SourceSelect(SourceSelectModal::new(
                                self.available_sources.clone(),
                                &self.active_sources,
                            )));
                        } else {
                            self.active_modal = None;
                        }
                    }
                    (
                        crate::ui::modals::install_sources::InstallState::Success(_)
                        | crate::ui::modals::install_sources::InstallState::Error(_),
                        KeyCode::Enter | KeyCode::Esc,
                    ) => {
                        self.active_modal = None;
                    }
                    _ => {}
                },
                ModalState::SourceSelect(ref mut sm) => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        let selected = sm.get_selected_sources();
                        self.active_sources = selected;
                        self.config.active_sources = self.active_sources.clone();
                        let _ = self.config.save();
                        self.update_active_source_label();
                        self.active_modal = None;
                        if self.current_view != CurrentView::Favorites
                            && self.current_view != CurrentView::MangaResults
                            && self.current_view != CurrentView::ChapterList
                        {
                            self.current_view = CurrentView::Search;
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        self.should_quit = true;
                    }
                    KeyCode::Char(' ') => {
                        sm.toggle_check();
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        sm.toggle_all();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        sm.move_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        sm.move_down();
                    }
                    _ => {}
                },
                ModalState::ConfirmRemove(ref cm) => match key.code {
                    KeyCode::Enter
                    | KeyCode::Char('y')
                    | KeyCode::Char('Y')
                    | KeyCode::Char('s')
                    | KeyCode::Char('S') => {
                        FavoriteManager::remove_favorite(&mut self.favorites, &cm.manga.url);
                        if self.favorites.is_empty() {
                            self.favorites_list_state.select(None);
                            if self.current_view == CurrentView::Favorites {
                                self.current_view = CurrentView::Search;
                            }
                        } else {
                            let curr = self.favorites_list_state.selected().unwrap_or(0);
                            let next_idx = curr.min(self.favorites.len().saturating_sub(1));
                            self.favorites_list_state.select(Some(next_idx));
                        }
                        self.active_modal = None;
                    }
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.active_modal = None;
                    }
                    _ => {}
                },
            }
            return;
        }

        // 2. View-specific controls
        match self.current_view {
            // Browsing favorites on home screen
            CurrentView::Favorites => match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('/') | KeyCode::Tab => {
                    self.current_view = CurrentView::Search;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if !self.available_sources.is_empty() {
                        self.active_modal = Some(ModalState::SourceSelect(SourceSelectModal::new(
                            self.available_sources.clone(),
                            &self.active_sources,
                        )));
                    }
                }
                KeyCode::Char('s') => {
                    self.active_modal = Some(ModalState::Settings(SettingsModal::new(&self.config)));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_favorite_selection(-1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_favorite_selection(1);
                }
                KeyCode::Char('f') => {
                    if let Some(idx) = self.favorites_list_state.selected() {
                        if let Some(fav) = self.favorites.get(idx) {
                            self.active_modal = Some(ModalState::ConfirmRemove(
                                ConfirmRemoveModal::new(fav.to_manga()),
                            ));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.favorites_list_state.selected() {
                        if let Some(fav) = self.favorites.get(idx).cloned() {
                            let manga = fav.to_manga();
                            let provider = manga.provider.clone();
                            self.selected_manga = Some(manga.clone());
                            self.previous_view = Some(CurrentView::Favorites);
                            self.trigger_fetch_chapters(manga.url, provider);
                        }
                    }
                }
                _ => {}
            },

            // Actively typing in search
            CurrentView::Search => match key.code {
                KeyCode::Esc => {
                    if !self.favorites.is_empty() {
                        self.current_view = CurrentView::Favorites;
                    } else {
                        self.current_view = CurrentView::SearchUnfocused;
                    }
                }
                KeyCode::Enter => {
                    let q = self.search_input.trim().to_string();
                    if !q.is_empty() && !self.is_busy {
                        self.trigger_search(q);
                    }
                }
                KeyCode::Tab | KeyCode::Down => {
                    if !self.mangas.is_empty() {
                        self.current_view = CurrentView::MangaResults;
                    } else if !self.favorites.is_empty() {
                        self.current_view = CurrentView::Favorites;
                    } else {
                        self.current_view = CurrentView::SearchUnfocused;
                    }
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                }
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                }
                _ => {}
            },

            // Unfocused search bar
            CurrentView::SearchUnfocused => match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('s') => {
                    self.active_modal = Some(ModalState::Settings(SettingsModal::new(&self.config)));
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if !self.available_sources.is_empty() {
                        self.active_modal = Some(ModalState::SourceSelect(SourceSelectModal::new(
                            self.available_sources.clone(),
                            &self.active_sources,
                        )));
                    }
                }
                KeyCode::Char('/') | KeyCode::Enter => {
                    self.current_view = CurrentView::Search;
                }
                KeyCode::Down | KeyCode::Tab => {
                    if !self.mangas.is_empty() {
                        self.current_view = CurrentView::MangaResults;
                    } else if !self.favorites.is_empty() {
                        self.current_view = CurrentView::Favorites;
                    }
                }
                _ => {}
            },

            // Manga results list
            CurrentView::MangaResults => match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Esc | KeyCode::Char('b') => {
                    if !self.favorites.is_empty() {
                        self.current_view = CurrentView::Favorites;
                    } else {
                        self.current_view = CurrentView::SearchUnfocused;
                    }
                }
                KeyCode::Char('/') => {
                    self.current_view = CurrentView::Search;
                }
                KeyCode::Char('f') => {
                    if let Some(idx) = self.manga_list_state.selected() {
                        if let Some(manga) = self.mangas.get(idx) {
                            if FavoriteManager::is_favorite(&self.favorites, &manga.url) {
                                self.active_modal = Some(ModalState::ConfirmRemove(
                                    ConfirmRemoveModal::new(manga.clone()),
                                ));
                            } else {
                                FavoriteManager::toggle_favorite(&mut self.favorites, manga);
                            }
                        }
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if !self.available_sources.is_empty() {
                        self.active_modal = Some(ModalState::SourceSelect(SourceSelectModal::new(
                            self.available_sources.clone(),
                            &self.active_sources,
                        )));
                    }
                }
                KeyCode::Char('s') => {
                    self.active_modal = Some(ModalState::Settings(SettingsModal::new(&self.config)));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_manga_selection(-1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_manga_selection(1);
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.manga_list_state.selected() {
                        if let Some(manga) = self.mangas.get(idx).cloned() {
                            let provider = manga.provider.clone();
                            self.selected_manga = Some(manga.clone());
                            self.previous_view = Some(CurrentView::MangaResults);
                            self.trigger_fetch_chapters(manga.url, provider);
                        }
                    }
                }
                _ => {}
            },

            // Chapter list screen with pagination and filter
            CurrentView::ChapterList => {
                if self.is_chapter_filter_active {
                    // Actively typing in chapter filter
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            self.is_chapter_filter_active = false;
                        }
                        KeyCode::Backspace => {
                            self.chapter_filter.pop();
                            self.chapter_page = 0;
                            self.chapter_list_state.select(Some(0));
                        }
                        KeyCode::Char(c) => {
                            self.chapter_filter.push(c);
                            self.chapter_page = 0;
                            self.chapter_list_state.select(Some(0));
                        }
                        _ => {}
                    }
                } else {
                    // Browsing chapters
                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Esc | KeyCode::Char('b') => {
                            if let Some(prev) = self.previous_view.take() {
                                self.current_view = prev;
                            } else if !self.favorites.is_empty() {
                                self.current_view = CurrentView::Favorites;
                            } else if !self.mangas.is_empty() {
                                self.current_view = CurrentView::MangaResults;
                            } else {
                                self.current_view = CurrentView::Search;
                            }
                        }
                        KeyCode::Char('s') => {
                            self.active_modal =
                                Some(ModalState::Settings(SettingsModal::new(&self.config)));
                        }
                        KeyCode::Char('/') => {
                            self.is_chapter_filter_active = true;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            // Previous page with wrap-around!
                            let total = self.total_chapter_pages();
                            if total > 0 {
                                if self.chapter_page == 0 {
                                    self.chapter_page = total - 1; // Wrap to last page
                                } else {
                                    self.chapter_page -= 1;
                                }
                                self.chapter_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            // Next page with wrap-around!
                            let total = self.total_chapter_pages();
                            if total > 0 {
                                if self.chapter_page + 1 >= total {
                                    self.chapter_page = 0; // Wrap to first page
                                } else {
                                    self.chapter_page += 1;
                                }
                                self.chapter_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.move_chapter_selection(-1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.move_chapter_selection(1);
                        }
                        KeyCode::Char(' ') => {
                            // Toggle checkbox for current item
                            let page_chapters = self.current_page_chapters();
                            if let Some(idx) = self.chapter_list_state.selected() {
                                if let Some(ch) = page_chapters.get(idx) {
                                    if self.selected_chapter_ids.contains(&ch.id) {
                                        self.selected_chapter_ids.remove(&ch.id);
                                    } else {
                                        self.selected_chapter_ids.insert(ch.id.clone());
                                    }
                                }
                            }
                        }
                        KeyCode::Char('a') => {
                            // Select all matching chapters
                            let ids: Vec<String> = self
                                .filtered_chapters()
                                .into_iter()
                                .map(|c| c.id.clone())
                                .collect();
                            let all_selected = ids
                                .iter()
                                .all(|id| self.selected_chapter_ids.contains(id));
                            if all_selected {
                                for id in &ids {
                                    self.selected_chapter_ids.remove(id);
                                }
                            } else {
                                for id in ids {
                                    self.selected_chapter_ids.insert(id);
                                }
                            }
                        }
                        // Enter opens the unified Process Modal (KCC, Fusion, Cover Preview, Direct CBZ)
                        KeyCode::Enter => {
                            self.trigger_process_modal();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn move_favorite_selection(&mut self, delta: isize) {
        if self.favorites.is_empty() {
            return;
        }
        let current = self.favorites_list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.favorites.len() as isize - 1) as usize;
        self.favorites_list_state.select(Some(next));
    }

    fn move_manga_selection(&mut self, delta: isize) {
        if self.mangas.is_empty() {
            return;
        }
        let current = self.manga_list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.mangas.len() as isize - 1) as usize;
        self.manga_list_state.select(Some(next));
    }

    fn move_chapter_selection(&mut self, delta: isize) {
        let page_chapters = self.current_page_chapters();
        if page_chapters.is_empty() {
            return;
        }
        let current = self.chapter_list_state.selected().unwrap_or(0) as isize;
        let candidate = current + delta;
        let total = self.total_chapter_pages();

        if candidate < 0 {
            // Wrap to previous page or wrap around to last page
            if self.chapter_page == 0 {
                self.chapter_page = total - 1;
            } else {
                self.chapter_page -= 1;
            }
            let prev_len = self.current_page_chapters().len();
            self.chapter_list_state
                .select(Some(prev_len.saturating_sub(1)));
        } else if candidate as usize >= page_chapters.len() {
            // Wrap to next page or wrap around to first page
            if self.chapter_page + 1 >= total {
                self.chapter_page = 0;
            } else {
                self.chapter_page += 1;
            }
            self.chapter_list_state.select(Some(0));
        } else {
            self.chapter_list_state.select(Some(candidate as usize));
        }
    }

    /// Asynchronous search trigger across all active sources concurrently
    fn trigger_search(&mut self, query: String) {
        self.is_busy = true;
        self.job_status = JobStatus::Idle;
        let sources_to_search: Vec<String> = if !self.active_sources.is_empty() {
            self.active_sources.clone()
        } else {
            vec![self.active_source.clone()]
        };
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut tasks = Vec::new();
            for src_name in sources_to_search {
                let q = query.clone();
                let task = tokio::task::spawn_blocking(move || {
                    let lua_scraper = SourceManager::load_source(&src_name)?;
                    lua_scraper.search(&q)
                });
                tasks.push(task);
            }

            let mut all_results = Vec::new();
            let mut any_success = false;
            let mut last_err = None;

            for task in tasks {
                match task.await {
                    Ok(Ok(mangas)) => {
                        any_success = true;
                        all_results.extend(mangas);
                    }
                    Ok(Err(e)) => {
                        last_err = Some(e);
                    }
                    Err(e) => {
                        last_err = Some(NeoError::Other(e.to_string()));
                    }
                }
            }

            if any_success || (all_results.is_empty() && last_err.is_none()) {
                let _ = tx.send(AppEvent::SearchResults(Ok(all_results)));
            } else if let Some(err) = last_err {
                let _ = tx.send(AppEvent::SearchResults(Err(err)));
            } else {
                let _ = tx.send(AppEvent::SearchResults(Ok(Vec::new())));
            }
        });
    }

    /// Asynchronous chapter list fetch trigger
    fn trigger_fetch_chapters(&mut self, series_url: String, provider: String) {
        self.is_busy = true;
        self.job_status = JobStatus::Idle;
        let active_source = if !provider.is_empty() {
            provider
        } else if let Some(first) = self.active_sources.first() {
            first.clone()
        } else {
            self.active_source.clone()
        };
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let lua_scraper = SourceManager::load_source(&active_source)?;
                lua_scraper.chapters(&series_url)
            })
            .await
            .unwrap_or_else(|e| Err(NeoError::Other(e.to_string())));

            let _ = tx.send(AppEvent::ChaptersResults(res));
        });
    }

    /// Opens the unified Process Modal for the current chapter selection
    fn trigger_process_modal(&mut self) {
        let page_chapters = self.current_page_chapters();
        let count = if self.selected_chapter_ids.is_empty() {
            if self.chapter_list_state.selected().is_none() {
                return;
            }
            1
        } else {
            self.selected_chapter_ids.len()
        };

        let target_title = if self.selected_chapter_ids.is_empty() {
            let idx = self.chapter_list_state.selected().unwrap_or(0);
            page_chapters
                .get(idx)
                .map(|c| c.title.clone())
                .unwrap_or_else(|| "Chapter".to_string())
        } else if let Some(manga) = &self.selected_manga {
            format!("{} ({} chapters)", manga.title, count)
        } else {
            format!("Volume ({} chapters)", count)
        };

        let toolchain = ToolchainStatus::check();
        let default_kcc = toolchain.is_ready();

        self.active_modal = Some(ModalState::Process(ProcessModal::new(
            &target_title,
            count,
            &self.config.kcc_format,
            default_kcc,
        )));
    }

    /// Executes chapter processing in batch (either direct CBZ or KCC conversion)
    fn start_batch_processing(&mut self, custom_cover: Option<PathBuf>, convert_kcc: bool) {
        let page_chapters = self.current_page_chapters();
        let target_chapters: Vec<Chapter> = if self.selected_chapter_ids.is_empty() {
            if let Some(sel) = self.chapter_list_state.selected() {
                page_chapters.get(sel).cloned().into_iter().collect()
            } else {
                vec![]
            }
        } else {
            self.chapters
                .iter()
                .filter(|c| self.selected_chapter_ids.contains(&c.id))
                .cloned()
                .collect()
        };

        if target_chapters.is_empty() {
            return;
        }

        let manga = match self.selected_manga.clone() {
            Some(m) => m,
            None => return,
        };

        let config = self.config.clone();
        let active_source = if !manga.provider.is_empty() {
            manga.provider.clone()
        } else {
            self.active_source.clone()
        };
        let tx = self.event_tx.clone();

        self.is_busy = true;

        tokio::spawn(async move {
            for chapter in target_chapters {
                let _ = tx.send(AppEvent::DownloadStatus(JobStatus::Downloading {
                    current: 0,
                    total: 1,
                }));
                let chap_url = chapter.url.clone();
                let src_name = active_source.clone();
                let pages_res = tokio::task::spawn_blocking(move || {
                    let lua_scraper = SourceManager::load_source(&src_name)?;
                    lua_scraper.pages(&chap_url)
                })
                .await
                .unwrap_or_else(|e| Err(NeoError::Other(e.to_string())));

                let pages = match pages_res {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = tx.send(AppEvent::OperationError(format!(
                            "Failed to get pages for {}: {}",
                            chapter.title, e
                        )));
                        return;
                    }
                };

                let sanitized_manga = ChapterDownloader::sanitize_name(&manga.title);
                let sanitized_chapter = ChapterDownloader::sanitize_name(&chapter.title);

                // Use a temporary folder for downloading images so no raw PNG folder is left in output dir
                let temp_dir = std::env::temp_dir().join(format!("neo_mangal_{}_{}", sanitized_manga, sanitized_chapter));

                let (status_tx, mut status_rx) = mpsc::channel(16);
                let forward_tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(status) = status_rx.recv().await {
                        let _ = forward_tx.send(AppEvent::DownloadStatus(status));
                    }
                });

                let downloader = ChapterDownloader::new(config.concurrent_downloads);
                if let Err(e) = downloader
                    .download_chapter(&pages, &temp_dir, Some(status_tx))
                    .await
                {
                    let _ = fs::remove_dir_all(&temp_dir);
                    let _ = tx.send(AppEvent::OperationError(format!(
                        "Download failed for {}: {}",
                        chapter.title, e
                    )));
                    return;
                }

                let _ = tx.send(AppEvent::DownloadStatus(JobStatus::PackagingCbz));
                let cbz_path = config
                    .download_dir
                    .join(&sanitized_manga)
                    .join(format!("{}.cbz", sanitized_chapter));

                let pack_res = CbzPacker::package(&temp_dir, custom_cover.as_deref(), &cbz_path);
                // Clean up raw image folder immediately!
                let _ = fs::remove_dir_all(&temp_dir);

                if let Err(e) = pack_res {
                    let _ = tx.send(AppEvent::OperationError(format!(
                        "CBZ Packaging failed: {}",
                        e
                    )));
                    return;
                }

                if convert_kcc {
                    let (kcc_tx, mut kcc_rx) = mpsc::channel(16);
                    let forward_kcc_tx = tx.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = kcc_rx.recv().await {
                            let _ = forward_kcc_tx.send(AppEvent::KccStatus(msg));
                        }
                    });

                    let output_dir = config.download_dir.join(&sanitized_manga);
                    match KccRunner::convert(&cbz_path, &output_dir, &config, Some(kcc_tx)).await {
                        Ok(final_path) => {
                            let _ = tx.send(AppEvent::ChapterDownloaded {
                                manga_url: manga.url.clone(),
                                chapter_title: chapter.title.clone(),
                            });
                            let _ = tx.send(AppEvent::OperationSuccess(format!(
                                "Kindle output ready:\n{}",
                                final_path.display()
                            )));
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::OperationError(format!(
                                "KCC Conversion failed: {}",
                                e
                            )));
                            return;
                        }
                    }
                } else {
                    let _ = tx.send(AppEvent::ChapterDownloaded {
                        manga_url: manga.url.clone(),
                        chapter_title: chapter.title.clone(),
                    });
                    let _ = tx.send(AppEvent::OperationSuccess(format!(
                        "CBZ direct save complete:\n{}",
                        cbz_path.display()
                    )));
                }
            }
        });
    }

    /// Executes Volume Fusion: fuses multiple selected chapters into a single volume CBZ with volume cover,
    /// and optionally converts the resulting volume using KCC.
    fn start_fusion_processing(&mut self, volume_cover: Option<PathBuf>, convert_kcc: bool) {
        let mut target_chapters: Vec<Chapter> = if self.selected_chapter_ids.is_empty() {
            let page_chapters = self.current_page_chapters();
            if let Some(sel) = self.chapter_list_state.selected() {
                page_chapters.get(sel).cloned().into_iter().collect()
            } else {
                vec![]
            }
        } else {
            self.chapters
                .iter()
                .filter(|c| self.selected_chapter_ids.contains(&c.id))
                .cloned()
                .collect()
        };

        if target_chapters.is_empty() {
            return;
        }

        // Sort chapters chronologically / by chapter number if available
        target_chapters.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let manga = match self.selected_manga.clone() {
            Some(m) => m,
            None => return,
        };

        let first_title = target_chapters.first().map(|c| c.title.as_str()).unwrap_or("Start");
        let last_title = target_chapters.last().map(|c| c.title.as_str()).unwrap_or("End");
        let last_chapter_name = last_title.to_string();
        let volume_name = if target_chapters.len() == 1 {
            first_title.to_string()
        } else {
            format!("{} - Volume ({} to {})", manga.title, first_title, last_title)
        };

        let sanitized_manga = ChapterDownloader::sanitize_name(&manga.title);
        let sanitized_volume = ChapterDownloader::sanitize_name(&volume_name);

        let config = self.config.clone();
        let active_source = if !manga.provider.is_empty() {
            manga.provider.clone()
        } else {
            self.active_source.clone()
        };
        let tx = self.event_tx.clone();

        self.is_busy = true;

        tokio::spawn(async move {
            let temp_fusion_dir = std::env::temp_dir().join(format!("neo_mangal_fusion_{}", sanitized_volume));
            let _ = fs::create_dir_all(&temp_fusion_dir);

            let mut global_page_idx = 1;
            let total_ch = target_chapters.len();

            for (ch_idx, chapter) in target_chapters.into_iter().enumerate() {
                let _ = tx.send(AppEvent::DownloadStatus(JobStatus::ConvertingKcc {
                    message: format!("Fetching chapter {}/{}...", ch_idx + 1, total_ch),
                }));

                let chap_url = chapter.url.clone();
                let src_name = active_source.clone();
                let pages_res = tokio::task::spawn_blocking(move || {
                    let lua_scraper = SourceManager::load_source(&src_name)?;
                    lua_scraper.pages(&chap_url)
                })
                .await
                .unwrap_or_else(|e| Err(NeoError::Other(e.to_string())));

                let pages = match pages_res {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = fs::remove_dir_all(&temp_fusion_dir);
                        let _ = tx.send(AppEvent::OperationError(format!(
                            "Failed to get pages for {}: {}",
                            chapter.title, e
                        )));
                        return;
                    }
                };

                let ch_temp = temp_fusion_dir.join(format!("ch_{:04}", ch_idx + 1));
                let downloader = ChapterDownloader::new(config.concurrent_downloads);
                if let Err(e) = downloader.download_chapter(&pages, &ch_temp, None).await {
                    let _ = fs::remove_dir_all(&temp_fusion_dir);
                    let _ = tx.send(AppEvent::OperationError(format!(
                        "Download failed during fusion: {}",
                        e
                    )));
                    return;
                }

                // Move and re-index all pages sequentially
                if let Ok(entries) = fs::read_dir(&ch_temp) {
                    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
                    files.sort();
                    for file in files {
                        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let new_dest = temp_fusion_dir.join(format!("{:06}.{}", global_page_idx, ext));
                        let _ = fs::rename(file, new_dest);
                        global_page_idx += 1;
                    }
                }
                let _ = fs::remove_dir_all(&ch_temp);
            }

            // Package into single volume CBZ
            let _ = tx.send(AppEvent::DownloadStatus(JobStatus::PackagingCbz));
            let cbz_path = config
                .download_dir
                .join(&sanitized_manga)
                .join(format!("{}.cbz", sanitized_volume));

            let pack_res = CbzPacker::package(&temp_fusion_dir, volume_cover.as_deref(), &cbz_path);
            let _ = fs::remove_dir_all(&temp_fusion_dir);

            if let Err(e) = pack_res {
                let _ = tx.send(AppEvent::OperationError(format!(
                    "Volume Fusion CBZ packaging failed: {}",
                    e
                )));
                return;
            }

            if convert_kcc {
                let (kcc_tx, mut kcc_rx) = mpsc::channel(16);
                let forward_kcc_tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(msg) = kcc_rx.recv().await {
                        let _ = forward_kcc_tx.send(AppEvent::KccStatus(msg));
                    }
                });

                let output_dir = config.download_dir.join(&sanitized_manga);
                match KccRunner::convert(&cbz_path, &output_dir, &config, Some(kcc_tx)).await {
                    Ok(final_path) => {
                        let _ = tx.send(AppEvent::ChapterDownloaded {
                            manga_url: manga.url.clone(),
                            chapter_title: last_chapter_name.clone(),
                        });
                        let _ = tx.send(AppEvent::OperationSuccess(format!(
                            "Kindle Volume ready:\n{}",
                            final_path.display()
                        )));
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::OperationError(format!(
                            "KCC Conversion failed: {}",
                            e
                        )));
                        return;
                    }
                }
            } else {
                let _ = tx.send(AppEvent::ChapterDownloaded {
                    manga_url: manga.url.clone(),
                    chapter_title: last_chapter_name.clone(),
                });
                let _ = tx.send(AppEvent::OperationSuccess(format!(
                    "Volume Fusion complete!\nSaved at: {}",
                    cbz_path.display()
                )));
            }
        });
    }

    /// Master render function for the TUI
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let lang = self.current_language();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Length(3), // Search bar
                Constraint::Min(8),    // Main list (manga or chapters)
                Constraint::Length(3), // Progress bar
                Constraint::Length(2), // Status bar
            ])
            .split(area);

        // 1. Header
        HeaderComponent::render(frame, chunks[0], &self.active_source, self.is_busy, lang);

        // 2. Search Bar
        let is_search_focused = self.current_view == CurrentView::Search;
        SearchBarComponent::render(frame, chunks[1], &self.search_input, is_search_focused, lang);

        // 3. Main Content
        match self.current_view {
            CurrentView::Favorites => {
                FavoritesListComponent::render(
                    frame,
                    chunks[2],
                    &self.favorites,
                    &mut self.favorites_list_state,
                    true,
                    lang,
                );
            }
            CurrentView::Search | CurrentView::SearchUnfocused => {
                if self.mangas.is_empty() {
                    FavoritesListComponent::render(
                        frame,
                        chunks[2],
                        &self.favorites,
                        &mut self.favorites_list_state,
                        false,
                        lang,
                    );
                } else {
                    MangaListComponent::render(
                        frame,
                        chunks[2],
                        &self.mangas,
                        &self.favorites,
                        &mut self.manga_list_state,
                        false,
                        lang,
                    );
                }
            }
            CurrentView::MangaResults => {
                MangaListComponent::render(
                    frame,
                    chunks[2],
                    &self.mangas,
                    &self.favorites,
                    &mut self.manga_list_state,
                    true,
                    lang,
                );
            }
            CurrentView::ChapterList => {
                let title = self
                    .selected_manga
                    .as_ref()
                    .map(|m| m.title.as_str())
                    .unwrap_or("Manga");
                let page_chapters = self.current_page_chapters();
                let total_matching = self.filtered_chapters().len();
                let total_pages = self.total_chapter_pages();

                ChapterListComponent::render(
                    frame,
                    chunks[2],
                    title,
                    &page_chapters,
                    self.chapter_page,
                    total_pages,
                    total_matching,
                    &self.selected_chapter_ids,
                    &self.chapter_filter,
                    self.is_chapter_filter_active,
                    &mut self.chapter_list_state,
                    lang,
                );
            }
        }

        // 4. Progress Bar
        ProgressBarComponent::render(frame, chunks[3], &self.job_status, lang);

        // 5. Status Bar
        let view_str = match self.current_view {
            CurrentView::Favorites => "favorites",
            CurrentView::Search => "search",
            CurrentView::SearchUnfocused => "search_unfocused",
            CurrentView::MangaResults => "manga_list",
            CurrentView::ChapterList => {
                if self.is_chapter_filter_active {
                    "chapter_filter"
                } else {
                    "chapter_list"
                }
            }
        };
        let out_str = self.config.download_dir.to_string_lossy();
        StatusBarComponent::render(frame, chunks[4], view_str, &out_str, lang);

        // 6. Overlaid Modals
        if let Some(ref mut modal) = self.active_modal {
            match modal {
                ModalState::Settings(s) => s.render(frame, area),
                ModalState::Process(p) => p.render(frame, area, lang),
                ModalState::KccMissing(km) => km.render(frame, area, lang),
                ModalState::Alert(a) => a.render(frame, area),
                ModalState::InstallSources(im) => im.render(frame, area, lang),
                ModalState::SourceSelect(ss) => ss.render(frame, area, lang),
                ModalState::ConfirmRemove(cr) => cr.render(frame, area, lang),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_app() -> App {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(Config::default(), tx);
        app.chapters = vec![
            Chapter { id: "1".into(), manga_id: "m".into(), title: "Chapter 1 - Dawn".into(), url: "u1".into(), number: Some(1.0) },
            Chapter { id: "2".into(), manga_id: "m".into(), title: "Chapter 2 - Luffy".into(), url: "u2".into(), number: Some(2.0) },
            Chapter { id: "3".into(), manga_id: "m".into(), title: "Chapter 100 - Legend".into(), url: "u3".into(), number: Some(100.0) },
            Chapter { id: "4".into(), manga_id: "m".into(), title: "Chapter 101 - Entry".into(), url: "u4".into(), number: Some(101.0) },
            Chapter { id: "5".into(), manga_id: "m".into(), title: "Chapter 102 - Grand Line".into(), url: "u5".into(), number: Some(102.0) },
        ];
        app.chapter_page_size = 2;
        app
    }

    #[test]
    fn test_pagination_and_filter() {
        let mut app = dummy_app();
        assert_eq!(app.total_chapter_pages(), 3);
        assert_eq!(app.current_page_chapters().len(), 2);
        assert_eq!(app.current_page_chapters()[0].id, "1");

        // Page 1 (2nd page)
        app.chapter_page = 1;
        assert_eq!(app.current_page_chapters().len(), 2);
        assert_eq!(app.current_page_chapters()[0].id, "3");

        // Filter chapters
        app.chapter_filter = "10".into();
        assert_eq!(app.filtered_chapters().len(), 3); // 100, 101, 102
        assert_eq!(app.total_chapter_pages(), 2);
    }

    #[test]
    fn test_wrap_around_pagination() {
        let mut app = dummy_app();
        // At page 0, wrap around to last page
        let total = app.total_chapter_pages();
        assert_eq!(total, 3);
        
        // Emulate wrap left from page 0
        if app.chapter_page == 0 {
            app.chapter_page = total - 1;
        }
        assert_eq!(app.chapter_page, 2);

        // Emulate wrap right from last page
        if app.chapter_page + 1 >= total {
            app.chapter_page = 0;
        }
        assert_eq!(app.chapter_page, 0);
    }

    #[test]
    fn test_initial_screen_and_source_modal() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(Config::default(), tx);

        if !app.available_sources.is_empty() {
            // Should NOT force open SourceSelectModal by default on launch anymore!
            assert!(app.active_modal.is_none());

            // Global shortcut Ctrl+P opens SourceSelect modal from any view
            app.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
            match &app.active_modal {
                Some(ModalState::SourceSelect(_)) => {}
                _ => panic!("Expected SourceSelect modal when pressing Ctrl+P"),
            }

            // Pressing Space toggles selection
            app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

            // Pressing Enter chooses the selected sources and closes modal
            app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
            assert!(app.active_modal.is_none());
            assert!(!app.active_sources.is_empty());

            // In Favorites view, 'p' also opens the modal
            app.current_view = CurrentView::Favorites;
            app.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
            match &app.active_modal {
                Some(ModalState::SourceSelect(_)) => {}
                _ => panic!("Expected SourceSelect modal when pressing 'p' in Favorites view"),
            }

            // Pressing Esc closes the modal AND saves the selected sources!
            app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
            assert!(app.active_modal.is_none());
            assert!(!app.active_sources.is_empty());
        }
    }

    #[test]
    fn test_favorite_toggle_and_last_chapter_event() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(Config::default(), tx);

        let manga = Manga {
            id: "one-piece".into(),
            title: "One Piece".into(),
            url: "https://example.com/one-piece".into(),
            cover_url: None,
            provider: "WeebCentral".into(),
        };

        app.mangas = vec![manga.clone()];
        app.manga_list_state.select(Some(0));
        app.current_view = CurrentView::MangaResults;

        // Press 'f' to favorite the selected manga
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(app.favorites.iter().any(|f| f.url == manga.url));

        // Simulate chapter downloaded event
        app.handle_event(AppEvent::ChapterDownloaded {
            manga_url: manga.url.clone(),
            chapter_title: "Chapter 1111".into(),
        });

        let fav = app.favorites.iter().find(|f| f.url == manga.url).unwrap();
        assert_eq!(fav.last_downloaded_chapter.as_deref(), Some("Chapter 1111"));

        // Press 'f' again opens confirmation modal
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(matches!(app.active_modal, Some(ModalState::ConfirmRemove(_))));

        // Confirm removal with Enter
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.favorites.iter().any(|f| f.url == manga.url));
        assert!(app.active_modal.is_none());
    }

    #[tokio::test]
    async fn test_navigation_stack_and_persistence() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(Config::default(), tx);

        let manga = Manga {
            id: "naruto".into(),
            title: "Naruto".into(),
            url: "https://example.com/naruto".into(),
            cover_url: None,
            provider: "WeebCentral".into(),
        };

        // Add a favorite
        app.favorites = vec![FavoriteManga::from_manga(&manga)];
        app.favorites_list_state.select(Some(0));
        app.current_view = CurrentView::Favorites;

        // User had previously searched something
        app.mangas = vec![manga.clone()];

        // User enters favorite manga chapters from Favorites view
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.previous_view, Some(CurrentView::Favorites));

        // Emulate chapters loaded
        app.current_view = CurrentView::ChapterList;

        // User presses Esc -> MUST return to Favorites, NOT MangaResults!
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::Favorites);

        // Now test entering chapters from MangaResults
        app.current_view = CurrentView::MangaResults;
        app.manga_list_state.select(Some(0));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.previous_view, Some(CurrentView::MangaResults));

        app.current_view = CurrentView::ChapterList;
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::MangaResults);
    }

    #[test]
    fn test_confirm_remove_favorite_flow() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(Config::default(), tx);

        let manga = Manga {
            id: "bleach".into(),
            title: "Bleach".into(),
            url: "https://example.com/bleach".into(),
            cover_url: None,
            provider: "MangaDex".into(),
        };

        app.favorites = vec![FavoriteManga::from_manga(&manga)];
        app.favorites_list_state.select(Some(0));
        app.current_view = CurrentView::Favorites;

        // 1. Pressing 'd' or Delete should do NOTHING
        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        assert!(app.active_modal.is_none());
        assert_eq!(app.favorites.len(), 1);

        app.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        assert!(app.active_modal.is_none());
        assert_eq!(app.favorites.len(), 1);

        // 2. Pressing 'f' opens confirmation modal
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(matches!(app.active_modal, Some(ModalState::ConfirmRemove(_))));

        // 3. Pressing 'n' or Esc cancels the modal
        app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        assert!(app.active_modal.is_none());
        assert_eq!(app.favorites.len(), 1);

        // 4. Pressing 'f' again and confirming with 's' (Portuguese Sim) or 'y' or Enter
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(matches!(app.active_modal, Some(ModalState::ConfirmRemove(_))));

        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        assert!(app.active_modal.is_none());
        assert_eq!(app.favorites.len(), 0);
        assert_eq!(app.current_view, CurrentView::Search);
    }
}
