use std::fs::File;
use std::io::Read;
use std::path::Path;
use napi::bindgen_prelude::BigInt;
use steamworks::{Client, PublishedFileId};
use tokio::sync::oneshot;
use steamworksjs::api::cloud::cloud::{delete_file, write_file, write_file_by_buffer};
use steamworksjs::client;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use napi::bindgen_prelude::BigInt;
    use napi::tokio::sync::oneshot;
    use napi::TypedArrayType::BigInt64;
    use steamworks::{ItemDetailsQuery, PublishedFileId};
    use tokio::time::{sleep, Timeout, timeout};
    use steamworksjs::api::localplayer::localplayer;
    use steamworksjs::api::localplayer::localplayer::{get_steam_id, set_rich_presence};
    use steamworksjs::api::workshop::workshop::{create_item, UgcUpdate, update_item};

    #[test]
    fn it_works() {
        let results = steamworksjs::init(480);
        println!("{:#?}", results);
        let name = localplayer::get_name();
        println!("{:#?}", name);
    }

    fn cast(a: PublishedFileId) -> u64 {
        a.0
    }

    #[test]
    fn it_ugc() {
        steamworksjs::init(1634680);
        let client = steamworksjs::client::get_client();
        let a = client.ugc().subscribed_items();
        let v: Vec<_> = a.iter().map(|x| x.0).collect();

        println!("{:#?}", v);
    }

    #[test]
    fn test_query_item() {
        query_item_fetch();
    }

    pub async fn query_item_fetch() {
        // steamworksjs::init(1634680);
        // let client = steamworksjs::client::get_client();
        // let a = client.ugc().subscribed_items();
        //
        // let (tx, rx) = oneshot::channel();
        // let mut r = client.ugc().query_item(a[0]);
        // let a = match r {
        //     Ok(x) => x,
        //     Err(error) => panic!("Problem opening the file: {:?}", error),
        // };
        // a.fetch(|result| {
        //     tx.send(Ok(result)).unwrap();
        // });
        //
        // let result = rx.await.unwrap();
    }

    #[test]
    fn get_steam_id_test() {
        steamworksjs::init(1634680);
        let client = steamworksjs::client::get_client();
        let id = get_steam_id();
        println!("{:#?}", id.account_id);
    }

    #[test]
    fn set_rich_presence_test() {
        steamworksjs::init(1634680);
        set_rich_presence(String::from("videoName"), String::from("foo"));
        set_rich_presence(String::from("steam_display"), String::from("#VideoBeingWatched"));
    }

    #[tokio::test]
    async fn set_update_item() {
        steamworksjs::init(1634680);
        let id = BigInt::from(2892196504);
        let update = UgcUpdate {
            title: Some(String::from("ttt")),
            description: Some(String::from("ttt")),
            change_note: Some(String::from("ttt")),
            preview_path: None,
            content_path: None,
            tags: None,
        };
        let b = update_item(id, update);
    }

    async fn ten_sec<F>(cb: F)
        where
            F: FnOnce(u32) -> (),
    {
        sleep(Duration::from_millis(10000)).await;
        cb(10000);
    }

    fn print_after_ten_sec<F>(cb: F)
        where
            F: FnOnce(u32) -> (),
    {

        ten_sec(cb);
    }
    #[tokio::test]
    async fn test(){
        let (tx, rx) = oneshot::channel();

        print_after_ten_sec(|result| {
            tx.send(result).unwrap();
        });
        let result = rx.await;
        println!("{:#?}", result);
        println!("100 ms have elapsed");
    }
}


#[test]
fn ttttt(){

}

fn read_file_to_string(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn read_zip_to_vec(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    Ok(content)
}

#[test]
fn test_cloud() {
    steamworksjs::init(2492750);
    let client = steamworksjs::client::get_client();
    // delete_file(String::from("test-23-08-9 1.20.32.zip"));
    let fileString = read_zip_to_vec("C:\\Users\\colin\\AppData\\Roaming\\Electron\\saviorData\\140280942\\backup\\1145360\\manualBackup\\23-08-9 1.20.32.zip");
    write_file_by_buffer(String::from("test.zip"),fileString.unwrap());
}
