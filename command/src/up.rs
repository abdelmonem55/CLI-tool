use crate::build::{build_from_args, generate_build_args};
use crate::deploy::deploy_from_args;
use crate::push::push_from_args;
use crate::{CommandAppend, State};
use clap::{App, Arg, ArgMatches, SubCommand};

pub(crate) struct Up;

impl CommandAppend for Up {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let up = SubCommand::with_name("up")
            .about(
                r#"Build, Push, and Deploy OpenFaaS function containers either via the
supplied YAML config using the "--yaml" flag (which may contain multiple function
definitions), or directly via flags.

The push step may be skipped by setting the --skip-push flag
and the deploy step with --skip-deploy.

Note: All flags from the build, push and deploy flags are valid and can be combined,
see the --help text for those commands for details.`,
	Example: `  faas-cli up -f myfn.yaml
faas-cli up --filter "*gif*" --secret dockerhuborg`"#,
            )
            .args_from_usage(
                "
        --skip-push                   'Skip pushing function to remote registry'
        --skip-deploy                 'Skip function deployment'

         --network [network]               'Name of the network'
          -n ,--namespace [namespace]       'Namespace of the function'
         --replace                          'Remove and re-create existing function(s)'
         --update                           'Perform rolling update on existing function(s)'
         --readonly                         'Force the root container filesystem to be read only'
         --tls-no-verify                      'Disable TLS validation'
         -k ,--token  [token]                     'Pass a JWT token to use instead of basic auth'
         --read-template                       'Read the function's template'
        ",
            )
            .arg(
                Arg::with_name("env")
                    .help("Set one or more environment variables --env e1=v1 ")
                    .long("env")
                    .short("e")
                    .takes_value(true)
                    .global(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("label")
                    .help("Set one or more label (LABEL=VALUE) ")
                    .long("label")
                    //.short("l")
                    .takes_value(true)
                    .global(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("annotation")
                    .help(" Set one or more annotation (ANNOTATION=VALUE)")
                    .long("annotation")
                    .takes_value(true)
                    .global(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("constraint")
                    .help("Apply a constraint to the function")
                    .long("constraint")
                    .takes_value(true)
                    .global(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("secret")
                    .help("Give the function access to a secure secret")
                    .long("secret")
                    .takes_value(true)
                    .global(true)
                    .multiple(true),
            );
        let up = generate_build_args(up);
        let app = app.subcommand(up);
        app
    }
}

impl Up {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(u_args) = args.subcommand_matches("up") {
            build_from_args(u_args).await?;
            println!();

            if !u_args.is_present("skip-push") {
                push_from_args(u_args).await?;
                println!();
            }
            if !u_args.is_present("skip-deploy") {
                deploy_from_args(u_args).await?;
                println!();
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}
