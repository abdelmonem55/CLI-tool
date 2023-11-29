use tempdir::TempDir;
use utility::Error;

#[test]
fn test_copy_files() {
    let file_modes = vec![0600, 0640, 0644, 0700, 0755];
    // let dir = std::env::temp_dir();
    for mode in file_modes {
        let src_dir = setup_source_folder(2, mode);
        assert!(src_dir.is_ok());
        let dest_dir = TempDir::new("00openfaas-test-destination-");
        assert!(dest_dir.is_ok());
        let src_dir = src_dir.unwrap();
        let dest_dir = dest_dir.unwrap();
        let des = dest_dir.path().to_str().unwrap();
        let src = src_dir.path().to_str().unwrap();
        let res = builder::copy_files(src, format!("{}/", des).as_str());
        assert!(res.is_ok());
        assert!(check_destination_files(des, 2, mode).is_ok())
    }
}
fn check_destination_files(_dir: &str, files_count: i32, _mode: i32) -> Result<(), utility::Error> {
    for _i in 1..=files_count {
        #[cfg(target_os = "unix")]
        {
            let info = std::fs::metadata(format!("{}/test-file-{}", _dir, _i))?;
            use std::os::unix::fs::PermissionsExt;
            if std::fs::Permissions::from_mode(_mode) != info.permissions() {
                return Err(utility::Error::Custom("expected mode not match".into()));
            }
        }
    }
    Ok(())
}
fn setup_source_folder(files_count: i32, _mode: i32) -> Result<TempDir, Error> {
    //let dir = std::env::temp_dir();
    let data = "open faas";
    let tmp_dir = TempDir::new("00openfaas-test-source-")?;

    for i in 1..=files_count {
        let src_file = tmp_dir.path().join(format!("test-file-{}", i));

        #[cfg(target_os = "unix")]
        {
            let file = std::fs::File::create(&src_file)?;
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(_mode))?;
        }
        std::fs::write(src_file, data.as_bytes())?;
    }

    Ok(tmp_dir)
}
