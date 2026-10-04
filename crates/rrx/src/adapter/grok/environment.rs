//! Frozen native baseline selection and live value-free Project reference admission.
use super::*;
use crate::{project::{environment_name_forbidden, environment_name_valid, validate_environment_references}, state::EnvironmentAdmission};

pub(super) fn control(key: &str) -> bool {
    baseline_key(key) && (!environment_name_valid(key) || environment_name_forbidden(key)
        || key == "NODE_TLS_REJECT_UNAUTHORIZED"
        || ["HOME","PATH","TMPDIR","BASH_ENV","ENV","SHELL","ZDOTDIR","XDG_CONFIG_HOME","XDG_DATA_HOME","XDG_STATE_HOME","XDG_CACHE_HOME","NODE_OPTIONS","SSL_CERT_FILE","SSL_CERT_DIR","REQUESTS_CA_BUNDLE","CURL_CA_BUNDLE","HTTP_PROXY","HTTPS_PROXY","ALL_PROXY","NO_PROXY","http_proxy","https_proxy","all_proxy","no_proxy"].contains(&key)
        || key.starts_with("LD_") || key.starts_with("DYLD_"))
}
pub(super) fn admission(baseline: &BTreeMap<String,String>, caller: impl IntoIterator<Item=String>) -> AdapterResult<EnvironmentAdmission> {
    EnvironmentAdmission::new(baseline.keys().filter(|key| !control(key)).cloned(), caller).map_err(state_error)
}
pub(super) fn select(adapter: &GrokAdapter, request: &LaunchRequest) -> AdapterResult<(BTreeMap<String,String>, EnvironmentAdmission)> {
    validate_environment_references(&request.project.environment_refs).map_err(|_| failure(ErrorKind::InvalidConfiguration,"owning environment references invalid"))?;
    if request.project.environment_refs.iter().any(|key| control(key)) {
        return Err(failure(ErrorKind::InvalidConfiguration,"native control references are not scoped environment values"));
    }
    let mut selected = adapter.baseline.clone();
    for (key,value) in &request.environment {
        if key.starts_with("GIT_") || key.is_empty() || key.contains(['=','\0']) || value.contains('\0') {
            return Err(failure(ErrorKind::InvalidInput,"invalid/leaking Git environment"));
        }
        if key.starts_with("RRX_") || !environment_name_valid(key) || environment_name_forbidden(key) || control(key)
            || (!ordinary_key(key) && (!baseline_key(key) || adapter.baseline.get(key) != Some(value))) {
            return Err(failure(ErrorKind::InvalidConfiguration,"native environment value cannot replace intentional runtime authority"));
        }
        selected.insert(key.clone(),value.clone());
    }
    let authority = admission(&adapter.baseline,request.environment.keys().cloned())?;
    adapter.store.lock().map_err(|_| failure(ErrorKind::StateFailure,"state store poisoned"))?.check_environment_admission(request.scope.project_id,&authority).map_err(state_error)?;
    Ok((selected,authority))
}
