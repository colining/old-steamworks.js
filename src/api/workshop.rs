use napi_derive::napi;
use steamworks::PublishedFileVisibility;
#[napi]
pub mod workshop {
    use napi::bindgen_prelude::{BigInt, Error};
    use std::path::Path;
    use steamworks::{AppIDs, FileType, PublishedFileId, PublishedFileVisibility, QueryResult, SteamError, SteamId, UGCType, UserList, UserListOrder};
    use tokio::sync::oneshot;
    use crate::api::localplayer::PlayerSteamId;

    #[napi(object)]
    pub struct UgcResult {
        pub item_id: BigInt,
        pub needs_to_accept_agreement: bool,
    }

    #[napi(object)]
    pub struct UgcUpdate {
        pub title: Option<String>,
        pub description: Option<String>,
        pub change_note: Option<String>,
        pub preview_path: Option<String>,
        pub content_path: Option<String>,
        pub tags: Option<Vec<String>>,
    }

    #[napi(object)]
    pub struct InstallInfo {
        pub folder: String,
        pub size_on_disk: BigInt,
        pub timestamp: u32,
    }

    #[napi(object)]
    pub struct DownloadInfo {
        pub current: BigInt,
        pub total: BigInt,
    }

    #[napi(object)]
    pub struct QueryItem {
        pub published_file_id: BigInt,
        pub owner_steam_id: PlayerSteamId,
        pub title: String,
        pub description: String,
        pub tags: Vec<String>,
        pub url: String,
    }

    #[napi]
    pub async fn create_item() -> Result<UgcResult, Error> {
        let client = crate::client::get_client();
        let appid = client.utils().app_id();

        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .create_item(appid, FileType::Community, |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn update_item(
        item_id: BigInt,
        update_details: UgcUpdate,
    ) -> Result<UgcResult, Error> {
        let client = crate::client::get_client();
        let appid = client.utils().app_id();

        let (tx, rx) = oneshot::channel();

        {
            let mut update = client
                .ugc()
                .start_item_update(appid, PublishedFileId(item_id.get_u64().1));

            if let Some(title) = update_details.title {
                update = update.title(title.as_str());
            }

            if let Some(description) = update_details.description {
                update = update.description(description.as_str());
            }

            if let Some(preview_path) = update_details.preview_path {
                update = update.preview_path(Path::new(&preview_path));
            }

            if let Some(tags) = update_details.tags {
                update = update.tags(tags);
            }

            if let Some(content_path) = update_details.content_path {
                update = update.content_path(Path::new(&content_path));
            }
            update = update.visibility(PublishedFileVisibility::Public);
            let change_note = update_details.change_note.as_deref();

            update.submit(change_note, |result| {
                tx.send(result).unwrap();
            });
        }

        let result = rx.await.unwrap();
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn query_item(
        item_id: BigInt,
    ) -> Result<QueryItem,Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        {
            let temp = client
                .ugc()
                .query_item(PublishedFileId(item_id.get_u64().1));
            let itemDetailQuery = temp.unwrap();

            itemDetailQuery.fetch(|result| {
                let qs = match result {
                    Ok(t) => Ok(t.get(0).unwrap()),
                    Err(e) => Err(e),
                };
                tx.send(qs).unwrap();
            });
        }

        let result = rx.await.unwrap();
        match result {
            Ok(r) => Ok(QueryItem {
                published_file_id: BigInt::from(r.published_file_id.0),
                owner_steam_id: PlayerSteamId::from_steamid(r.owner),
                title: r.title,
                description: r.description,
                tags: r.tags,
                url: r.url,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn query_user(page: u32) -> Result<Vec<QueryItem>,Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        let account = client.user().steam_id().account_id();
        let appId = client.utils().app_id();
        {
            let temp = client
                .ugc()
                .query_user(
                    account,
                    UserList::Subscribed,
                    UGCType::Items,
                    UserListOrder::SubscriptionDateDesc,
                    AppIDs::Both {creator: appId,consumer:appId},
                    page);
            let userListQuery = temp.unwrap();

            userListQuery.fetch(|result| {
                let qs = match result {
                    Ok(t) => Ok(t.iter().map(|x| x.unwrap()).collect::<Vec<QueryResult>>()),
                    Err(e) => Err(e),
                };
                tx.send(qs).unwrap();
            });
        }

        let result = rx.await.unwrap();
        match result {
            Ok(res) => Ok(res.iter().map(|r| QueryItem {
                published_file_id: BigInt::from(r.published_file_id.0),
                owner_steam_id: PlayerSteamId::from_steamid(r.owner),
                title: String::from(r.title.clone()),
                description: String::from(r.description.clone()),
                tags:r.tags.clone(),
                url:r.url.clone(),
            }).collect::<Vec<QueryItem>>()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }


    #[napi]
    pub async fn query_user_total_results_num() -> Result<u32,Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        let account = client.user().steam_id().account_id();
        let appId = client.utils().app_id();
        {
            let temp = client
                .ugc()
                .query_user(
                    account,
                    UserList::Subscribed,
                    UGCType::Items,
                    UserListOrder::CreationOrderAsc,
                    AppIDs::Both {creator: appId,consumer:appId},
                    1);
            let userListQuery = temp.unwrap();

            userListQuery.fetch(|result| {
                let qs = match result {
                    Ok(t) => Ok(t.total_results()),
                    Err(e) => Err(e),
                };
                tx.send(qs).unwrap();
            });
        }

        let result = rx.await.unwrap();
        match result {
            Ok(res) => Ok(res),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Subscribe to a workshop item. It will be downloaded and installed as soon as possible.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem}
    #[napi]
    pub async fn subscribe(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .subscribe_item(PublishedFileId(item_id.get_u64().1), |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Unsubscribe from a workshop item. This will result in the item being removed after the game quits.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem}
    #[napi]
    pub async fn unsubscribe(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .unsubscribe_item(PublishedFileId(item_id.get_u64().1), |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
    #[napi]
    pub async fn delete_item(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .delete_item(PublishedFileId(item_id.get_u64().1), |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Gets the current state of a workshop item on this client. States can be combined.
    ///
    /// @returns a number with the current item state, e.g. 9
    /// 9 = 1 (The current user is subscribed to this item) + 8 (The item needs an update)
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState}
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#EItemState}
    #[napi]
    pub fn state(item_id: BigInt) -> u32 {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_state(PublishedFileId(item_id.get_u64().1));

        result.bits()
    }

    /// Gets info about currently installed content on the disc for workshop item.
    ///
    /// @returns an object with the the properties {folder, size_on_disk, timestamp}
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo}
    #[napi]
    pub fn install_info(item_id: BigInt) -> Option<InstallInfo> {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_install_info(PublishedFileId(item_id.get_u64().1));

        match result {
            Some(install_info) => Some(InstallInfo {
                folder: install_info.folder,
                size_on_disk: BigInt::from(install_info.size_on_disk),
                timestamp: install_info.timestamp,
            }),
            None => None,
        }
    }

    /// Get info about a pending download of a workshop item.
    ///
    /// @returns an object with the properties {current, total}
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo}
    #[napi]
    pub fn download_info(item_id: BigInt) -> Option<DownloadInfo> {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_download_info(PublishedFileId(item_id.get_u64().1));

        match result {
            Some(download_info) => Some(DownloadInfo {
                current: BigInt::from(download_info.0),
                total: BigInt::from(download_info.1),
            }),
            None => None,
        }
    }

    /// Download or update a workshop item.
    ///
    /// @param highPriority - If high priority is true, start the download in high priority mode, pausing any existing in-progress Steam downloads and immediately begin downloading this workshop item.
    /// @returns true or false
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem}
    #[napi]
    pub fn download(item_id: BigInt, high_priority: bool) -> bool {
        let client = crate::client::get_client();
        client
            .ugc()
            .download_item(PublishedFileId(item_id.get_u64().1), high_priority)
    }

    #[napi]
    pub fn subscribed_items() -> Vec<u64>  {
        let client = crate::client::get_client();
        let results = client.ugc().subscribed_items();
        results.iter().map(|x| x.0).collect()
    }
}
