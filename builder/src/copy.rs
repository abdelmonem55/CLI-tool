use std::fs;

// package builder
//
// Copy "recursivelies copy a file object from source to dest while perserving
// import (
// "fmt"
// "io"
// "io/ioutil"
// "os"
// "path"
// "path/filepath"
// )
//
// // CopyFiles copies files from src to destination.
// /func CopyFiles(src, dest string) error {
// info, err := os.Stat(src)
// if err != nil {
// return err
// }
//
// if info.IsDir() {
// debugPrint(fmt.Sprintf("Creating directory: %s at %s", info.Name(), dest))
// return copyDir(src, dest)
// }
//
// debugPrint(fmt.Sprintf("cp - %s %s", src, dest))
// return copyFile(src, dest)
// }

//CopyFiles copies files from src to destination.

pub fn copy_files(src: &str, dest: &str) -> Result<(), utility::Error> {
    debug_print(format!("after path \n{}\n{}", src, dest));
    let info = std::fs::metadata(src)?;

    if info.is_dir() {
        debug_print(format!("Creating directory: {} at {}", src, dest));
        return copy_dir(src, dest);
    }

    debug_print(format!("cp - {} to {}", src, dest));
    return copy_file(src, dest).map(|_| ());

    // if info.IsDir() {
    // debugPrint(fmt.Sprintf("Creating directory: %s at %s", info.Name(), dest))
    // return copyDir(src, dest)
    // }
    //
    // debugPrint(fmt.Sprintf("cp - %s %s", src, dest))
    // return copyFile(src, dest)
}

fn copy_file(src: &str, dest: &str) -> Result<u64, utility::Error> {
    ensure_base_dir(dest)?;
    //let d = std::fs::File::create(dest)?;
    fs::copy(src, dest)
        .map_err(|e| utility::Error::Custom(format!("can't copy {} to {} , {:?}", src, dest, e)))
}
fn ensure_base_dir(path: &str) -> Result<(), utility::Error> {
    let path = std::path::Path::new(path);
    let base_path = path.parent().ok_or(utility::Error::Custom(String::from(
        "path must not be empty or root",
    )))?;

    let data = fs::metadata(base_path);
    if let Ok(info) = data {
        if info.is_dir() {
            return Ok(());
        }
    }
    fs::create_dir_all(base_path).map_err(|e| utility::Error::Io(e))
}

fn copy_dir(src: &str, dest: &str) -> Result<(), utility::Error> {
    //   let info = std::fs::metadata(src)?;
    fs::create_dir_all(dest)?;
    let infos = fs::read_dir(src)?;

    //let src = std::path::Path::new(src);
    let dest = std::path::Path::new(dest);
    for info in infos {
        let info = info?;
        //   println!("infos {:?}",info.path().file_name());
        //let src = src.join(info.path());

        let dest = dest.join(info.file_name());
        let dest = dest.to_str().unwrap();
        let src = info.path();
        let src = src.to_str().unwrap();
        copy_files(src, dest)?
    }
    Ok(())
}
fn debug_print(message: String) {
    if let Ok(val) = std::env::var("debug") {
        if val == "true" || val == "1" {
            println!("{}", message);
        }
    }
}
//
// if info.IsDir() {
// debugPrint(fmt.Sprintf("Creating directory: %s at %s", info.Name(), dest))
// return copyDir(src, dest)
// }
//
// debugPrint(fmt.Sprintf("cp - %s %s", src, dest))
// return copyFile(src, dest)
// }
//
// // copyDir will recursively copy a directory to dest
// func copyDir(src, dest string) error {
// info, err := os.Stat(src)
// if err != nil {
// return fmt.Errorf("error reading dest stats: %s", err.Error())
// }
//
// if err := os.MkdirAll(dest, info.Mode()); err != nil {
// return fmt.Errorf("error creating path: %s - %s", dest, err.Error())
// }
//
// infos, err := ioutil.ReadDir(src)
// if err != nil {
// return err
// }
//
// for _, info := range infos {
// if err := CopyFiles(
// filepath.Join(src, info.Name()),
// filepath.Join(dest, info.Name()),
// ); err != nil {
// return err
// }
// }
//
// return nil
// }
//
// // copyFile will copy a file with the same mode as the src file
// func copyFile(src, dest string) error {
// info, err := os.Stat(src)
// if err != nil {
// return fmt.Errorf("error reading src file stats: %s", err.Error())
// }
//
// err = ensureBaseDir(dest)
// if err != nil {
// return fmt.Errorf("error creating dest base directory: %s", err.Error())
// }
//
// f, err := os.Create(dest)
// if err != nil {
// return fmt.Errorf("error creating dest file: %s", err.Error())
// }
// defer f.Close()
//
// if err = os.Chmod(f.Name(), info.Mode()); err != nil {
// return fmt.Errorf("error setting dest file mode: %s", err.Error())
// }
//
// s, err := os.Open(src)
// if err != nil {
// return fmt.Errorf("error opening src file: %s", err.Error())
// }
// defer s.Close()
//
// _, err = io.Copy(f, s)
// if err != nil {
// return fmt.Errorf("Error copying dest file: %s\n" + err.Error())
// }
//
// return nil
// }
//
// // ensureBaseDir creates the base directory of the given file path, if needed.
// // For example, if fpath is 'build/extras/dictionary.txt`, ensureBaseDir will
// // make sure that the directory `buid/extras/` is created.
// func ensureBaseDir(fpath string) error {
// baseDir := path.Dir(fpath)
// info, err := os.Stat(baseDir)
// if err == nil && info.IsDir() {
// return nil
// }
// return os.MkdirAll(baseDir, 0755)
// }
//
// func debugPrint(message string) {
//
// if val, exists := os.LookupEnv("debug"); exists && (val == "1" || val == "true") {
// fmt.Println(message)
// }
// }
