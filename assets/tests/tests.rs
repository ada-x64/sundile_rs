use sundile_assets::prelude::*;
use sundile_graphics::prelude::*;

#[test]
fn test_all() {
    // --
    // Serializer and Deserializer tests
    // --
    Serializer::default()
        .with_asset_directory("C:/dev/Quell/sundile_rs/assets/tests/assets")
        .with_out_path("C:/dev/Quell/sundile_rs/assets/tests/")
        .serialize();

    let hrt = futures::executor::block_on(
        HeadlessRenderTarget::new(false, None)
    );
    let bin = std::fs::read("C:/dev/Quell/sundile_rs/assets/tests/data.bin").unwrap();
    let mut map = Deserializer::default()
        .with_panic(true)
        .deserialize(&bin[..], &hrt);

    // --
    // AssetTypeMap tests
    // --

    dbg!(&map);
    //TODO: Check that this matches expected results.

    // try_get_asset
    map.try_get_asset::<&str, Model>("models", "cube").unwrap();
    assert_eq!(map.try_get_asset::<&str, Model>("err", "err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetTypeNotFound("err".into()));
    assert_eq!(map.try_get_asset::<&str, Model>("models", "err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetNotFound("err".into()));
    assert_eq!(map.try_get_asset::<&str, i32>("models", "cube").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidType);

    // try_take_asset
    assert_eq!(map.try_take_asset::<&str, Model>("err", "err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetTypeNotFound("err".into()));
    assert_eq!(map.try_take_asset::<&str, Model>("models", "err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetNotFound("err".into()));
    assert_eq!(map.try_take_asset::<&str, i32>("models", "cube").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidType);

    let gotten = map.try_get_asset::<&str, Model>("models", "cube").unwrap();
    assert_eq!(map.try_take_asset::<&str, Model>("models", "cube").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidTake);
    drop(gotten);
    let taken = map.try_take_asset::<&str, Model>("models", "cube").unwrap();

    // try_insert_asset
    assert!(map.try_insert_asset("models", "cube", taken).unwrap().is_none()); // new value in existing map
    assert!(map.try_insert_asset("numbers", "4", 4).unwrap().is_none()); // new map
    assert!(map.try_insert_asset("numbers", "4", 4).unwrap().is_some()); // override value in existing map
    assert_eq!(map.try_insert_asset("numbers", "5", 5.0).unwrap_err().0, AssetError::InvalidType);

    // try_get_asset_map
    assert_eq!(map.try_get_asset_map::<&str, Model>("numbers").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidType);
    assert_eq!(map.try_get_asset_map::<&str, i32>("err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetTypeNotFound("err".into()));
    map.try_get_asset_map::<&str, i32>("numbers").unwrap();

    // try_take_asset_map
    assert_eq!(map.try_take_asset_map::<&str, Model>("numbers").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidType);
    map.try_take_asset_map::<&str, i32>("numbers").unwrap();
    assert_eq!(map.try_take_asset_map::<&str, i32>("numbers").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetTypeNotFound("numbers".into()));
}