use crate::store::{
    filter_store_list, get_store_platforms, get_target_platform, store_list, DEFAULT_STORE,
    MAX_DESCRIPTION_LEN,
};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use schema::store::v2::store::StoreFunction;

pub(crate) struct StoreList;

impl SubCommandAppend for StoreList {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("list")
                .alias("ls")
                .about(r#"List all secrets`,
	Example: `faas-cli secret list
faas-cli secret list --gateway=http://127.0.0.1:8080`,"#)
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .global(true)
                        .help("Enable verbose output to see the full description of each function in the store")
                );

        app
    }
}

impl StoreList {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("list") {
            let store_address = l_args.value_of("url").unwrap_or(DEFAULT_STORE);
            let platform = l_args.value_of("platform").unwrap_or_default();
            let verbose = l_args.is_present("verbose");

            //todo check platform value
            let target_platform = get_target_platform(platform);
            let store_list = store_list(store_address).await?;
            let available_platforms = get_store_platforms(&store_list);

            let filtered_functions = filter_store_list(store_list, target_platform.as_str());
            if filtered_functions.is_empty() {
                colour::blue!("No functions found in the store for platform '{}', try one of the following: {}\n", target_platform
                         , available_platforms.join(", "));
            }

            colour::green!("{}", store_render_items(filtered_functions, verbose));

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn store_render_items(items: Vec<StoreFunction>, verbose: bool) -> String {
    let width = 45;
    let mut fmt = format!("{:width$}", "FUNCTION", width = width) + "DESCRIPTION\n";

    for item in items {
        let str = format!("{:width$}", format!("{}", item.title), width = width)
            + format!("{}\n", store_render_description(item.description, verbose)).as_str();
        // let str = format!(
        //     "{}\t{}\n",
        //     item.title,
        //     store_render_description(item.description, verbose)
        // );
        fmt.push_str(str.as_str());
    }

    fmt
}

pub(crate) fn store_render_description(descr: String, verbose: bool) -> String {
    if !verbose && descr.len() > MAX_DESCRIPTION_LEN {
        let (left, _) = descr.split_at(MAX_DESCRIPTION_LEN - 3);
        left.to_string() + "..."
    } else {
        descr
    }
}
