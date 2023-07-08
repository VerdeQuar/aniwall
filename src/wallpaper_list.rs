use crate::cli::Range;
use crate::rating::KonachanRatingFilter;
use crate::wallpaper::KonachanWallpaper;
use anyhow::Result;
use chrono::Duration;
use std::path::Path;
use tokio::fs::{self, File};

async fn fetch_wallpaper_list(
    width: &Range,
    height: &Range,
    tags: &Option<String>,
    rating: &KonachanRatingFilter,
) -> Result<Vec<KonachanWallpaper>, reqwest::Error> {
    let mut page = 0;
    let limit = 1000;
    let mut wallpapers: Vec<KonachanWallpaper> = vec![];
    let tags = tags
        .clone()
        .map_or("".to_string(), |s| s.replace(' ', "%20"));

    loop {
        page += 1;
        let mut response = reqwest::get(format!(
            "https://konachan.net/post.json?limit={limit}&tags={tags}%20rating%3A{rating}%20width%3A{width}+height%3A{height}&page={page}"
        ))
        .await?
        .json::<Vec<KonachanWallpaper>>()
        .await?;

        wallpapers.append(&mut response);

        if response.len() < limit {
            break;
        }
    }
    // wallpapers.sort_by(|a, b| (a.width / a.height).cmp(&(b.width / b.height)));
    wallpapers.sort_by(|a, b| a.score.cmp(&b.score));

    Ok(wallpapers)
}

fn compute_hash_for_filters(
    width: &Range,
    height: &Range,
    tags: &Option<String>,
    rating: &KonachanRatingFilter,
) -> String {
    let tags = tags.clone().unwrap_or("".to_string());
    let digest = md5::compute(format!("{tags}{rating}{width}{height}").as_bytes());
    format!("{:x}", digest)
}
async fn get_cached_wallpaper_list(
    cache_dir: &Path,
    width: &Range,
    height: &Range,
    tags: &Option<String>,
    rating: &KonachanRatingFilter,
) -> Result<Vec<KonachanWallpaper>> {
    let content =
        fs::read_to_string(cache_dir.join(compute_hash_for_filters(width, height, tags, rating)))
            .await?;
    Ok(serde_json::from_str(&content)?)
}

pub async fn get_wallpaper_list(
    cache_dir: &Path,
    width: &Range,
    height: &Range,
    tags: &Option<String>,
    rating: &KonachanRatingFilter,
) -> Result<Vec<KonachanWallpaper>> {
    let cached_wallpaper_list: Result<Vec<KonachanWallpaper>> =
        get_cached_wallpaper_list(cache_dir, width, height, tags, rating).await;
    let filters_hashed = compute_hash_for_filters(width, height, tags, rating);

    let cache_age: Result<i64> = try {
        Duration::from_std(
            File::open(cache_dir.join(&filters_hashed))
                .await?
                .metadata()
                .await?
                .modified()?
                .elapsed()?,
        )
        .map(|dur| dur.num_days())?
    };

    let wallpapers: Vec<KonachanWallpaper> = match (cached_wallpaper_list, cache_age) {
        (Ok(cached), Ok(..=5)) if !cached.is_empty() => cached,
        (cached, _) => {
            let fetched_wallpaper_list: Result<Vec<KonachanWallpaper>, reqwest::Error> =
                fetch_wallpaper_list(width, height, tags, rating).await;

            match fetched_wallpaper_list {
                Ok(fetched) => {
                    if let Ok(json) = serde_json::to_string(&fetched) {
                        fs::write(cache_dir.join(&filters_hashed).as_path(), json.as_bytes())
                            .await?
                    }
                    fetched
                }
                Err(_) => cached?,
            }
        }
    };
    Ok(wallpapers)
}
