#[test]
fn test_build_image_name_default_format() {
    let want = "img:latest";
    let got =
        schema::image::build_image_name(schema::image::DEFAULT_FORMAT, "img", "ef384", "master");

    assert_eq!(want, got.as_str());
}
