/// Package versioncontrol is a simplified/stripped down version of go/internal/get/vcs that
/// is aimed at the simplier temporary git clone needed for OpenFaaS template fetch.
pub mod core;
pub mod git;
pub mod parse;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
