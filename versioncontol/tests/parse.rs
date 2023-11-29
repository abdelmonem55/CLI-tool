use versioncontol::parse::{
    is_git_remote, is_pinned_git_remote, parse_panned_remote, PIN_CHARACTER,
};

struct GitInfo {
    pub name: &'static str,
    pub url: &'static str,
}
#[test]
fn test_is_git_remote() {
    let valid_urls = vec![
        GitInfo {
            name: "git protocol without .git suffix",
            url: "git://host.xz/path/to/repo",
        },
        GitInfo {
            name: "git protocol",
            url: "git://host.xz/path/to/repo.git/",
        },
        GitInfo {
            name: "scp style with ip address",
            url: "git@192.168.101.127:user/project.git",
        },
        GitInfo {
            name: "scp style with hostname",
            url: "git@github.com:user/project.git",
        },
        GitInfo {
            name: "http protocol with ip address",
            url: "http://192.168.101.127/user/project.git",
        },
        GitInfo {
            name: "http protocol",
            url: "http://github.com/user/project.git",
        },
        GitInfo {
            name: "http protocol without .git suffix",
            url: "http://github.com/user/project",
        },
        GitInfo {
            name: "https protocol with ip address",
            url: "https://192.168.101.127/user/project.git",
        },
        GitInfo {
            name: "https protocol with hostname",
            url: "https://github.com/user/project.git",
        },
        GitInfo {
            name: "https protocol with basic auth",
            url: "https://username:password@github.com/username/repository.git",
        },
        GitInfo {
            name: "ssh protocol with hostname no port",
            url: "ssh://user@host.xz/path/to/repo.git/",
        },
        GitInfo {
            name: "ssh protocol with hostname and port",
            url: "ssh://user@host.xz:port/path/to/repo.git/",
        },
    ];
    for git in valid_urls {
        assert!(is_git_remote(git.url));
    }
    let invalid_urls = vec![
        GitInfo {
            name: "git protocol with hash",
            url: "git://github.com/openfaas/faas.git#!!ff78lf9h",
        },
        GitInfo {
            name: "local repo file protocol",
            url: "file:///path/to/repo.git/",
        },
        GitInfo {
            name: "ssh missing username and port",
            url: "host.xz:/path/to/repo.git",
        },
        GitInfo {
            name: "ssh username and missing port",
            url: "user@host.xz:path/to/repo.git",
        },
        GitInfo {
            name: "relative local path",
            url: "path/to/repo.git/",
        },
        GitInfo {
            name: "magic relative local",
            url: "~/path/to/repo.git",
        },
    ];
    for git in invalid_urls {
        assert!(!is_git_remote(git.url));
    }
}

#[test]
fn test_is_panned_git_remote() {
    struct GitInfo {
        pub name: &'static str,
        pub url: String,
    }
    let valid_urls = vec![
        GitInfo {
            name: "git protocol without .git suffix",
            url: "git://host.xz/path/to/repo".to_string() + PIN_CHARACTER + "feature-branch",
        },
        GitInfo {
            name: "git protocol",
            url: "git://host.xz/path/to/repo.git/".to_string() + PIN_CHARACTER + "tagname",
        },
        GitInfo {
            name: "scp style with ip address",
            url: "git@192.168.101.127:user/project.git".to_string() + PIN_CHARACTER + "v1.2.3",
        },
        GitInfo {
            name: "scp style with hostname",
            url: "git@github.com:user/project.git".to_string() + PIN_CHARACTER + "feature-branch",
        },
        GitInfo {
            name: "http protocol with ip address",
            url: "http://192.168.101.127/user/project.git".to_string() + PIN_CHARACTER + "tagname",
        },
        GitInfo {
            name: "http protocol",
            url: "http://github.com/user/project.git".to_string() + PIN_CHARACTER + "v1.2.3",
        },
        GitInfo {
            name: "http protocol without .git suffix",
            url: "http://github.com/user/project".to_string() + PIN_CHARACTER + "feature-branch",
        },
        GitInfo {
            name: "https protocol with ip address",
            url: "https://192.168.101.127/user/project.git".to_string() + PIN_CHARACTER + "tagname",
        },
        GitInfo {
            name: "https protocol with hostname",
            url: "https://github.com/user/project.git".to_string() + PIN_CHARACTER + "v1.2.3",
        },
        GitInfo {
            name: "https protocol with basic auth",
            url: "https://username:password@github.com/username/repository.git".to_string()
                + PIN_CHARACTER
                + "feature/branch",
        },
        GitInfo {
            name: "ssh protocol with hostname no port",
            url: "ssh://user@host.xz/path/to/repo.git/".to_string() + PIN_CHARACTER + "v1.2.3",
        },
        GitInfo {
            name: "ssh protocol with hostname and port",
            url: "ssh://user@host.xz:port/path/to/repo.git/".to_string()
                + PIN_CHARACTER
                + "tagname",
        },
    ];
    for url in valid_urls {
        assert!(is_pinned_git_remote(url.url.as_str()));
    }

    let invalid_urls = vec![
        GitInfo {
            name: "ssh protocol with hostname no port without pin",
            url: "ssh://user@host.xz/path/to/repo.git/".to_string(),
        },
        GitInfo {
            name: "ssh protocol with hostname and port without pin",
            url: "ssh://user@host.xz:port/path/to/repo.git/".to_string(),
        },
        GitInfo {
            name: "scp style with ip address without pin",
            url: "git@192.168.101.127:user/project.git".to_string(),
        },
        GitInfo {
            name: "scp style with hostname without pin",
            url: "git@github.com:user/project.git".to_string(),
        },
        GitInfo {
            name: "git protocol without .git suffix and no tag",
            url: "git://host.xz/path/to/repo".to_string(),
        },
        GitInfo {
            name: "git protocol with hash",
            url: "git://github.com/openfaas/faas.git#ff78lf9h@feature/branch".to_string(),
        },
        GitInfo {
            name: "local repo file protocol",
            url: "file:///path/to/repo.git/@feature/branch".to_string(),
        },
        GitInfo {
            name: "ssh missing username and port",
            url: "host.xz:/path/to/repo.git".to_string() + PIN_CHARACTER + "feature-branch",
        },
        GitInfo {
            name: "ssh username and missing port",
            url: "user@host.xz:path/to/repo.git".to_string() + PIN_CHARACTER + "v1.2.3",
        },
        GitInfo {
            name: "relative local path",
            url: "path/to/repo.git/@feature/branch".to_string(),
        },
        GitInfo {
            name: "magic relative local",
            url: "~/path/to/repo.git@feature/branch".to_string(),
        },
    ];

    for url in invalid_urls {
        assert!(!is_pinned_git_remote(url.url.as_str()));
    }
}

#[test]
fn test_parse_pinned_remote() {
    struct Info {
        _name: &'static str,
        url: &'static str,
        ref_name: &'static str,
    }
    let cases = vec![
        Info {
            _name: "git protocol without .git suffix",
            url: "git://host.xz/path/to/repo",
            ref_name: "feature-branch",
        },
        Info {
            _name: "git protocol",
            url: "git://host.xz/path/to/repo.git/",
            ref_name: "tagname",
        },
        Info {
            _name: "scp style with ip address",
            url: "git@192.168.101.127:user/project.git",
            ref_name: "v1.2.3",
        },
        Info {
            _name: "scp style with hostname",
            url: "git@github.com:user/project.git",
            ref_name: "feature/branch",
        },
        Info {
            _name: "http protocol with ip address",
            url: "http://192.168.101.127/user/project.git",
            ref_name: "tagname",
        },
        Info {
            _name: "http protocol",
            url: "http://github.com/user/project.git",
            ref_name: "v1.2.3",
        },
        Info {
            _name: "http protocol without .git suffix",
            url: "http://github.com/user/project",
            ref_name: "feature/branch",
        },
        Info {
            _name: "https protocol with ip address",
            url: "https://192.168.101.127/user/project.git",
            ref_name: "tagname",
        },
        Info {
            _name: "https protocol with hostname",
            url: "https://github.com/user/project.git",
            ref_name: "v1.2.3",
        },
        Info {
            _name: "https protocol with basic auth",
            url: "https://username:password@github.com/username/repository.git",
            ref_name: "feature/branch",
        },
        Info {
            _name: "ssh protocol with hostname no port",
            url: "ssh://user@host.xz/path/to/repo.git/",
            ref_name: "v1.2.3",
        },
        Info {
            _name: "ssh protocol with hostname and port",
            url: "ssh://user@host.xz:port/path/to/repo.git/",
            ref_name: "tagname",
        },
    ];
    for case in cases {
        let (remote, ref_name) =
            parse_panned_remote((case.url.to_string() + PIN_CHARACTER + case.ref_name).as_str());
        assert_eq!(remote.as_str(), case.url);
        assert_eq!(ref_name, case.ref_name);
        let (remote, ref_name) = parse_panned_remote(case.url);
        assert_eq!(remote, case.url);
        assert!(ref_name.is_empty());
    }
}
