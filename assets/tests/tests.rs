use sundile_assets::*;
use sundile_graphics::*;
use sundile_common::*;

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
    expect_model_ref(map.try_get_asset::<Model>("cube").unwrap().as_ref()); //test type elision
    assert_eq!(map.try_get_asset::<Model>("err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetNotFound("err".into()));
    assert_eq!(map.try_get_asset::<i32>("err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetMapNotFound);

    // try_take_asset
    assert_eq!(map.try_take_asset::<Model>("err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetNotFound("err".into()));
    assert_eq!(map.try_take_asset::<i32>("err").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetMapNotFound);

    let gotten = map.try_get_asset::<Model>("cube").unwrap();
    assert_eq!(map.try_take_asset::<Model>("cube").unwrap_err().downcast::<AssetError>().unwrap(), AssetError::InvalidTake);
    drop(gotten);
    let taken = map.try_take_asset::<Model>("cube").unwrap();

    // try_insert_asset
    assert!(map.try_insert_asset("cube", taken).unwrap().is_none()); // new value in existing map
    assert!(map.try_insert_asset("4", 4).unwrap().is_none()); // new map
    assert!(map.try_insert_asset("4", 4).unwrap().is_some()); // override value in existing map

    // try_get_asset_map
    assert_eq!(map.try_get_asset_map::<String>().unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetMapNotFound);
    map.try_get_asset_map::<i32>().unwrap();

    // try_take_asset_map
    assert_eq!(map.try_take_asset_map::<String>().unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetMapNotFound);
    map.try_take_asset_map::<i32>().unwrap();
    assert_eq!(map.try_take_asset_map::<i32>().unwrap_err().downcast::<AssetError>().unwrap(), AssetError::AssetMapNotFound);
}

fn expect_model_ref(_: &Model) {}