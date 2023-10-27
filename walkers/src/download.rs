use egui::Context;
use futures::StreamExt;
use reqwest::header::USER_AGENT;

use crate::{mercator::TileId, providers::Provider, tiles::Tile};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Http(reqwest::Error),

    #[error("error while decoding the image: {0}")]
    Image(String),
}

/// Download and decode the tile.
async fn download_and_decode(client: &reqwest::Client, url: &str) -> Result<Tile, Error> {
    let image = client
        .get(url)
        .header(USER_AGENT, "Walkers")
        .send()
        .await
        .map_err(Error::Http)?;

    log::debug!("Downloaded {:?}.", image.status());

    let image = image
        .error_for_status()
        .map_err(Error::Http)?
        .bytes()
        .await
        .map_err(Error::Http)?;

    Tile::from_image_bytes(&image).map_err(Error::Image)
}

async fn download_continuously_impl(
    source: Provider,
    mut request_rx: futures::channel::mpsc::Receiver<TileId>,
    mut tile_tx: futures::channel::mpsc::Sender<(TileId, Tile)>,
    egui_ctx: Context,
) -> Result<(), ()> {
    // Keep outside the loop to reuse it as much as possible.
    let client = reqwest::Client::new();

    loop {
        let request = request_rx.next().await.ok_or(())?;
        let url = source.tile_url(request);

        log::debug!("Getting {:?} from {}.", request, url);

        match download_and_decode(&client, &url).await {
            Ok(tile) => {
                tile_tx.try_send((request, tile)).map_err(|_| ())?;
                egui_ctx.request_repaint();
            }
            Err(e) => {
                log::warn!("Could not download '{}': {}", &url, e);
            }
        }
    }
}

/// Continuously download tiles requested via request channel.
pub async fn download_continuously(
    source: Provider,
    request_rx: futures::channel::mpsc::Receiver<TileId>,
    tile_tx: futures::channel::mpsc::Sender<(TileId, Tile)>,
    egui_ctx: Context,
) {
    if download_continuously_impl(source, request_rx, tile_tx, egui_ctx)
        .await
        .is_err()
    {
        log::error!("Error from IO runtime.");
    }
}
