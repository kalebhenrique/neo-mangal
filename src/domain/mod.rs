pub mod favorite;
pub mod job;
pub mod manga;

#[allow(unused_imports)]
pub use favorite::{FavoriteManager, FavoriteManga};
#[allow(unused_imports)]
pub use job::{DownloadJob, JobStatus};
#[allow(unused_imports)]
pub use manga::{Chapter, Manga, Page};
