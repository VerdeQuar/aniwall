use futures::StreamExt;

use crate::wallpaper::{KonachanWallpaper, Wallpaper};
use anyhow::Result;
use std::path::Path;
use tokio::{fs, fs::File, sync::mpsc};

pub async fn download_wallpapers(
    wallpaper_list: Vec<KonachanWallpaper>,
    wallpapers_dir: &Path,
    tx: mpsc::Sender<Wallpaper>,
) -> Result<()> {
    for konachan_wallpaper in wallpaper_list {
        let path = wallpapers_dir
            .join(&konachan_wallpaper.md5)
            .with_extension("png");
        let wallpaper = if wallpapers_dir
            .join(&konachan_wallpaper.md5)
            .with_extension("json")
            .exists()
        {
            let wallpaper = Wallpaper::from_md5(wallpapers_dir, &konachan_wallpaper.md5)?;
            match wallpaper.category {
                Some(_) => continue,
                None => wallpaper,
            }
        } else {
            let mut dest = File::create(&path).await?;

            let mut content = reqwest::get(&konachan_wallpaper.file_url)
                .await?
                .bytes_stream();
            while let Some(chunk) = content.next().await {
                tokio::io::copy(&mut chunk?.as_ref(), &mut dest).await?;
            }
            let wallpaper = Wallpaper::from_konachan(konachan_wallpaper.clone(), path);

            if let Ok(json) = serde_json::to_string(&wallpaper) {
                fs::write(
                    wallpapers_dir
                        .join(&wallpapers_dir.join(&wallpaper.md5).with_extension("json"))
                        .as_path(),
                    json.as_bytes(),
                )
                .await?;
            }
            wallpaper
        };
        let _ = tx.send(wallpaper).await;
    }
    Ok(())
}
