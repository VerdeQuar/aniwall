use crate::crop::crop_wallpaper;
use crate::rating::Category;
use crate::wallpaper::{set_wallpaper, Prefered, Wallpaper};
use crate::wallpaper_history::{save_history, History};

use crate::rating::CategoryPrompt;
use anyhow::Result;
use inquire::{InquireError, Select};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs, select,
    sync::{
        mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task,
};
use tokio_util::sync::CancellationToken;

pub enum ProcessingStatus {
    Interupted,
    ToogleCropped,
    Done,
}
pub async fn process_wallpaper(
    wallpaper: &mut Wallpaper,
    is_cropped: bool,
) -> Result<ProcessingStatus> {
    let original_width = wallpaper.original_width;
    let original_height = wallpaper.original_height;
    let md5 = wallpaper.md5.clone();
    let category = task::spawn_blocking(move || {
        let mut options = vec![
            CategoryPrompt::Liked,
            CategoryPrompt::Disliked,
            CategoryPrompt::Borked,
        ];

        match is_cropped {
            true => {
                options.insert(0, CategoryPrompt::DidNotNeedCropping);
            }
            false => {
                options.insert(0, CategoryPrompt::NeedsCropping);
            }
        }

        let category = Select::new(
            &format!("{}\t Size: {}x{}.", md5, original_width, original_height),
            options,
        )
        .prompt();

        category
    })
    .await?;

    match category {
        Ok(CategoryPrompt::DidNotNeedCropping) | Ok(CategoryPrompt::NeedsCropping) => {
            Ok(ProcessingStatus::ToogleCropped)
        }
        Ok(category) => {
            wallpaper.category = Some(Category::try_from(category)?);
            Ok(ProcessingStatus::Done)
        }
        Err(InquireError::OperationInterrupted) | Err(InquireError::OperationCanceled) => {
            Ok(ProcessingStatus::Interupted)
        }
        Err(err) => Err(anyhow::Error::from(err)),
    }
}

pub async fn process_wallpapers(
    mut wallpapers_rx: Receiver<Wallpaper>,
    token: CancellationToken,
    history: History,
    wallpapers_dir: PathBuf,
    config_dir: PathBuf,
    screen_width: u16,
    screen_height: u16,
    set_wallpaper_command_override: Option<String>,
) -> Result<()> {
    let (cropped_tx, mut cropped_rx): (Sender<Wallpaper>, Receiver<Wallpaper>) = mpsc::channel(10);
    let (process_tx, mut process_rx): (Sender<Wallpaper>, Receiver<Wallpaper>) = mpsc::channel(10);
    let (shutdown_tx, mut shutdown_rx): (UnboundedSender<()>, UnboundedReceiver<()>) =
        mpsc::unbounded_channel();
    let shutdown_tx2 = shutdown_tx.clone();
    let history = Arc::new(Mutex::new(history));
    let prompt = Arc::new(Mutex::new(()));

    tokio::task::spawn({
        let token = token.clone();
        let token_cloned = token.clone();
        let history = history.clone();
        let prompt = prompt.clone();
        let config_dir = config_dir.clone();
        let wallpapers_dir = wallpapers_dir.clone();
        let set_wallpaper_command_override = set_wallpaper_command_override.clone();

        async move {
            let _ = shutdown_tx2.clone();
            select! {
                biased;
                _ = token_cloned.cancelled() => {}
                _ = async move || -> Result<()> {
                    'outer: while let Some(mut wallpaper) = process_rx.recv().await {
                        let prompt_lock = prompt.lock().await;
                        let mut is_cropped = wallpaper.prefered == Prefered::Cropped;
                        loop {
                            if is_cropped {
                                if let Some(crop_data) = &wallpaper.crop_data {
                                    set_wallpaper(&config_dir, &crop_data.cropped_image_path, set_wallpaper_command_override.clone())?;
                                }
                                wallpaper.prefered = Prefered::Cropped;
                            } else {
                                set_wallpaper(&config_dir, &wallpaper.downloaded_image_path, set_wallpaper_command_override.clone())?;
                                wallpaper.prefered = Prefered::Original;
                            }

                            match process_wallpaper(
                                &mut wallpaper,
                                is_cropped,
                            )
                            .await?
                            {
                                ProcessingStatus::Interupted => {
                                    let mut history = history.lock().await;
                                    if let Some(md5) = history.prev() {
                                        Wallpaper::from_md5(&wallpapers_dir, &md5)
                                            ?
                                            .set_prefered(&config_dir, set_wallpaper_command_override)
                                            ?;
                                    }
                                    token.cancel();
                                    break 'outer;
                                }
                                ProcessingStatus::ToogleCropped => {
                                    is_cropped = !is_cropped;
                                }
                                ProcessingStatus::Done => {
                                    let path = wallpapers_dir.join(&wallpaper.md5).with_extension("json");
                                    let json = serde_json::to_string(&wallpaper)?;

                                    fs::write(path, json.as_bytes()).await?;
                                    break;
                                }
                            }
                        }
                        drop(prompt_lock);
                    }
                    Ok(())
                }() => {}
            }
        }
    });

    tokio::task::spawn({
        let token = token.clone();
        let history_clone = history.clone();

        async move {
            let _ = shutdown_tx.clone();
            select! {
                biased;
                _ = token.cancelled() => {}
                _ = {
                    async move || -> Result<()> {
                        while let Some(original) = cropped_rx.recv().await {
                            let wallpaper = crop_wallpaper(
                                original,
                                screen_width,
                                screen_height,
                            )
                            .await?;

                            let mut history = history_clone.lock().await;
                            history.push(wallpaper.md5.clone());
                            drop(history);

                            process_tx.send(wallpaper).await?;
                        }
                        Ok(())
                    }
                }() => {}
            }
        }
    });

    let token = token.clone();
    let token_cloned = token.clone();
    let history_clone = history.clone();
    select! {
        biased;
        _ = token_cloned.cancelled() => {}
        _ = {
                let wallpapers_dir = wallpapers_dir.clone();
            async move || -> Result<()> {

                while let Some(mut wallpaper) = wallpapers_rx.recv().await {

                    let prompt_lock = prompt.lock().await;
                    let is_cropped = wallpaper.set_prefered(&config_dir, set_wallpaper_command_override.clone())? == Prefered::Cropped;

                    let mut history = history_clone.lock().await;
                    history.push(wallpaper.md5.clone());
                    drop(history);

                    match process_wallpaper(
                        &mut wallpaper,
                        is_cropped
                    )
                    .await?
                    {
                        ProcessingStatus::Interupted => {
                            let mut history = history_clone.lock().await;
                            if let Some(md5) = history.prev() {
                                Wallpaper::from_md5(&wallpapers_dir, &md5)
                                    ?
                                    .set_prefered(&config_dir, set_wallpaper_command_override)
                                    ?;
                            }
                            token.cancel();
                            break;
                        }
                        ProcessingStatus::ToogleCropped => {
                            cropped_tx.send(wallpaper).await?;
                        }
                        ProcessingStatus::Done => {
                            let path = wallpapers_dir.join(&wallpaper.md5).with_extension("json");
                            let json = serde_json::to_string(&wallpaper)?;

                            fs::write(path, json.as_bytes()).await?;
                        }
                    };
                    drop(prompt_lock);
                };
                Ok(())

            }
        }() => {}
    };
    shutdown_rx.recv().await;

    let history = history.lock().await;
    save_history(&wallpapers_dir, &*history)?;
    Ok(())
}
