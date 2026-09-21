pub mod alert;
pub mod anilist;
pub mod confirm_remove;
pub mod cover_prompt;
pub mod install_sources;
pub mod kcc_missing;
pub mod settings;
pub mod source_select;

pub use alert::AlertModal;
pub use anilist::AnilistModal;
pub use confirm_remove::ConfirmRemoveModal;
pub use cover_prompt::ProcessModal;
pub use install_sources::InstallSourcesModal;
pub use kcc_missing::KccMissingModal;
pub use settings::SettingsModal;
pub use source_select::SourceSelectModal;
