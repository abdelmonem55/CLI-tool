#![allow(dead_code)]

pub(crate) const OPENFAAS_URL_ENVIRONMENT: &str = "OPENFAAS_URL";
pub(crate) const TEMPLATE_URL_ENVIRONMENT: &str = "OPENFAAS_TEMPLATE_URL";
pub(crate) const TEMPLATE_STORE_URL_ENVIRONMENT: &str = "OPENFAAS_TEMPLATE_STORE_URL";

pub(crate) fn get_gateway_url(
    argument_url: &str,
    default_url: &str,
    yaml_url: &str,
    environment_url: &str,
) -> String {
    let mut gateway_url: String;

    if !argument_url.is_empty() && argument_url != default_url {
        gateway_url = argument_url.into();
    } else if !yaml_url.is_empty() && yaml_url != default_url {
        gateway_url = yaml_url.into();
    } else if !environment_url.is_empty() {
        gateway_url = environment_url.into();
    } else {
        gateway_url = default_url.into();
    }

    gateway_url = gateway_url.trim_end_matches('/').to_ascii_lowercase();
    if !gateway_url.starts_with("http://") {
        format!("http://{}", gateway_url)
    } else {
        gateway_url
    }
}

pub(crate) fn get_template_url(
    argument_url: &str,
    environment_url: &str,
    default_url: &str,
) -> String {
    if !argument_url.is_empty() && argument_url != default_url {
        argument_url.into()
    } else if !argument_url.is_empty() {
        environment_url.into()
    } else {
        default_url.into()
    }
}

pub(crate) fn get_template_store_url(
    argument_url: &str,
    environment_url: &str,
    default_url: &str,
) -> String {
    if argument_url != default_url {
        argument_url.to_string()
    } else if !environment_url.is_empty() {
        environment_url.to_string()
    } else {
        default_url.to_string()
    }
}

pub(crate) fn get_namespace(flag_namespace: &str, stack_namespace: &str) -> String {
    // If the namespace flag is passed use it
    if !flag_namespace.is_empty() {
        flag_namespace.into()
        // https://github.com/openfaas/faas-cli/issues/742#issuecomment-625746405
    } else if !stack_namespace.is_empty() {
        stack_namespace.into()
    } else {
        //return defaultFunctionNamespace
        "".into()
    }
}
