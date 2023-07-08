use crate::wallpaper::Prefered;
use crate::wallpaper::{CropData, Wallpaper};
use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use tokio::fs;

pub async fn crop_wallpaper(
    mut wallpaper: Wallpaper,
    width: u16,
    height: u16,
) -> Result<Wallpaper> {
    let mut cropped_image_pathbuf = wallpaper.downloaded_image_path.clone();
    cropped_image_pathbuf.set_file_name(format!(
        "{}_cropped",
        wallpaper
            .downloaded_image_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    ));

    if let Some(ext) = wallpaper.downloaded_image_path.extension() {
        cropped_image_pathbuf.set_extension(ext);
    }

    if cropped_image_pathbuf.exists() {
        return Ok(wallpaper);
    }

    let cropped_image_path_string: String = cropped_image_pathbuf.to_string_lossy().into();

    let downloaded_image_path_string: String =
        wallpaper.downloaded_image_path.to_string_lossy().into();

    let join = tokio::task::spawn_blocking(move || -> Result<()> {
        Command::new("magick")
            .args([
                &downloaded_image_path_string,
                "-resize",
                &format!("{width}x{height}^"),
                "resize.png",
            ])
            .stdin(Stdio::null())
            .spawn()
            .context("magick command failed to start, make sure to have ImageMagick installed")?
            .wait()?
            .exit_ok()?;
        Ok(())
    });
    join.await??;

    let join = tokio::task::spawn_blocking(move || -> Result<()> {
        Command::new("convert")
            .args([
                "resize.png",
                "-canny",
                "0x1+10%+30%",
                "-separate",
                "-evaluate-sequence",
                "max",
                "-blur",
                "0x20",
                "-equalize",
                "-resize",
                "10%x10%",
                "canny.png",
            ])
            .stdin(Stdio::null())
            .spawn()
            .context("convert command failed to start, make sure to have ImageMagick installed")?
            .wait()?
            .exit_ok()?;
        Ok(())
    });
    join.await??;

    let join = tokio::task::spawn_blocking(move || -> Result<String> {
        let child = Command::new("magick")
            .args([
                "compare",
                "-metric",
                "rmse",
                "-subimage-search",
                "-dissimilarity-threshold",
                "1",
                "canny.png",
                "(",
                "-size",
                &format!("{}x{}", width / 10, height / 10),
                "xc:white",
                ")",
                "null:",
            ])
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("magick command failed to start, make sure to have ImageMagick installed")?;
        let output = child.wait_with_output()?;
        Ok(String::from_utf8(output.stderr)?)
    });
    let output = join.await??;

    let (_, coords) = output.split_once('@').unwrap();
    let (x_off_str, y_off_str) = coords.split_once(',').unwrap();
    let x_off: i32 = x_off_str.trim().parse()?;
    let y_off: i32 = y_off_str.parse()?;

    fs::remove_file("canny.png").await?;
    let join = tokio::task::spawn_blocking(move || -> Result<()> {
        Command::new("convert")
            .args([
                "resize.png",
                "-crop",
                &format!("{width}x{height}+{x_off}+{y_off}"),
                "+repage",
                &cropped_image_path_string,
            ])
            .stdin(Stdio::null())
            .spawn()
            .context("convert command failed to start, make sure to have ImageMagick installed")?
            .wait()?
            .exit_ok()?;
        Ok(())
    });
    join.await??;

    fs::remove_file("resize.png").await?;

    wallpaper.crop_data = Some(CropData {
        cropped_image_path: cropped_image_pathbuf,
        crop_offset_x: x_off,
        crop_offset_y: y_off,
    });

    wallpaper.prefered = Prefered::Cropped;
    Ok(wallpaper)
}
