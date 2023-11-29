use crate::store::{
    filter_store_list, get_target_platform, store_find_function, store_list, DEFAULT_STORE,
    PLATFORM,
};
use crate::store_list::store_render_description;
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use schema::store::v2::store::StoreFunction;

pub(crate) struct StoreDescribe;

impl SubCommandAppend for StoreDescribe {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app = SubCommand::with_name("describe")
            .alias("inspect")
            .about(
                r#"Show details of OpenFaaS function from a store,
	Example: `  faas-cli store describe NodeInfo
  faas-cli store describe NodeInfo --url https://host:port/store.json"#,
            )
            .arg(
                Arg::with_name("FUNCTION-NAME")
                    .help("function name")
                    .index(1)
                    .required(true),
            )
            .arg(
                Arg::with_name("verbose")
                    .help("Verbose output for the field values")
                    .long("verbose")
                    .global(true)
                    .short("v"),
            );
        app
    }
}

impl StoreDescribe {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(d_args) = args.subcommand_matches("describe") {
            let store_address = args.value_of("url").unwrap_or(DEFAULT_STORE);
            let platform_value = d_args.value_of("platform").unwrap_or_default();
            let requested_store_fn = d_args.value_of("FUNCTION-NAME").ok_or(State::Custom(
                "function name must be set at index 0 like 'faas store deploy NAME'".to_string(),
            ))?;

            let verbose = d_args.is_present("verbose");

            colour::green!("platform: {}\n", PLATFORM);
            let target_platform = get_target_platform(platform_value);
            let store_items = store_list(store_address).await?;
            let platform_functions = filter_store_list(store_items, target_platform.as_str());
            let item = store_find_function(requested_store_fn, &platform_functions).ok_or(
                State::Custom(format!(
                    "function '{}' not found for platform '{}'",
                    requested_store_fn, target_platform
                )),
            )?;

            let content = store_render_item(item, target_platform.as_str(), verbose);
            colour::green!("{}", content);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn store_render_item(item: &StoreFunction, platform: &str, verbose: bool) -> String {
    // var b bytes.Buffer
    // w := tabwriter.NewWriter(&b, 0, 0, 1, ' ', 0)
    let width = 25;
    let mini_width = 25;
    let mut fmt = format!("Info for: {}\n\n", item.title);
    let str = format!("{:width$}", "Name", width = mini_width) + item.name.as_str() + "\n\n";
    // let str =  format!("{}\t{}\n", "Name", item.name);
    fmt.push_str(str.as_str());

    //let desc = &item.description;
    //desc := item.Description
    let description = if !verbose {
        store_render_description(item.description.to_string(), verbose)
    } else {
        item.description.to_owned()
    };

    let str = format!("{:width$}", "Description", width = width) + description.as_str() + "\n";
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "Image", width = width)
        + item
            .get_image_name(platform)
            .map(|m| m.as_str())
            .unwrap_or_default()
        + "\n";
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "Process", width = mini_width) + item.fprocess.as_str() + "\n";
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "Repo URL", width = width) + item.repo_url.as_str() + "\n";
    fmt.push_str(str.as_str());

    fmt
}
