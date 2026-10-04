//! Value-free Project reference admission. No filesystem or native configuration reads.
use std::collections::BTreeSet;

use anyhow::{Result, bail};
use rusqlite::{Connection, OptionalExtension};

use super::{StateGuardError, Store};
use crate::{
    domain::ProjectId,
    project::{environment_name_forbidden, environment_name_valid},
};

/// Names only, classified by the native provider before construction. Values never
/// cross this boundary. Bounds reject unsupported runtime configurations, not truncate.
pub(crate) struct EnvironmentAdmission {
    baseline: BTreeSet<String>,
    caller: BTreeSet<String>,
}
impl EnvironmentAdmission {
    pub(crate) fn new(
        baseline: impl IntoIterator<Item = String>,
        caller: impl IntoIterator<Item = String>,
    ) -> Result<Self> {
        fn bounded(
            names: impl IntoIterator<Item = String>,
            limit: usize,
            budget: usize,
        ) -> Result<BTreeSet<String>> {
            let mut selected = BTreeSet::new();
            let mut bytes = 0usize;
            for name in names {
                if name.len() > 256
                    || !environment_name_valid(&name)
                    || environment_name_forbidden(&name)
                {
                    bail!(StateGuardError::EnvironmentAuthority);
                }
                if selected.insert(name.clone()) {
                    bytes += name.len();
                    if selected.len() > limit || bytes > budget {
                        bail!(StateGuardError::EnvironmentAuthority);
                    }
                }
            }
            Ok(selected)
        }
        let baseline = bounded(baseline, 512, 65_536)?;
        let caller = bounded(caller, 128, 128 * 256)?;
        Ok(Self { baseline, caller })
    }
    pub(crate) fn candidates(&self, names: &[String]) -> BTreeSet<String> {
        names
            .iter()
            .filter(|name| self.baseline.contains(*name))
            .cloned()
            .collect()
    }
}

/// Extract only the JSON reference field; unrelated malformed foreign fields and
/// lifecycle are intentionally irrelevant. Missing/non-array/non-string authority
/// fails opaquely. Every valid string is considered; no inventory-count truncation.
fn refs(
    connection: &Connection,
    owner: ProjectId,
    own: bool,
    mut accept: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let comparison = if own { "=" } else { "<>" };
    let (count, invalid): (u64, u64) = connection.query_row(&format!(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN json_valid(body) THEN CASE WHEN json_type(body,'$.environment_refs')='array' THEN 0 ELSE 1 END ELSE 1 END),0) FROM projects WHERE id {comparison} ?1"
    ), [owner.to_string()], |row| Ok((row.get(0)?,row.get(1)?)))?;
    if invalid != 0 || (own && count != 1) {
        bail!(StateGuardError::EnvironmentAuthority);
    }
    let mut statement = connection.prepare(&format!(
        "SELECT entry.type, entry.value FROM projects AS project, json_each(CASE WHEN json_valid(project.body) THEN CASE WHEN json_type(project.body,'$.environment_refs')='array' THEN json_extract(project.body,'$.environment_refs') ELSE '[]' END ELSE '[]' END) AS entry WHERE project.id {comparison} ?1"
    ))?;
    let rows = statement.query_map([owner.to_string()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    for row in rows {
        let (kind, name) = row.map_err(|_| StateGuardError::EnvironmentAuthority)?;
        if kind != "text" {
            bail!(StateGuardError::EnvironmentAuthority);
        }
        accept(
            name.as_deref()
                .ok_or(StateGuardError::EnvironmentAuthority)?,
        )?;
    }
    Ok(())
}
fn own_refs(connection: &Connection, owner: ProjectId) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    refs(connection, owner, true, |name| {
        if !environment_name_valid(name)
            || environment_name_forbidden(name)
            || !names.insert(name.to_owned())
        {
            bail!(StateGuardError::EnvironmentAuthority);
        }
        Ok(())
    })?;
    Ok(names)
}

pub(super) fn evaluate(
    connection: &Connection,
    owner: ProjectId,
    admission: &EnvironmentAdmission,
) -> Result<()> {
    let own = own_refs(connection, owner)?;
    if !admission.caller.is_subset(&own) {
        bail!(StateGuardError::EnvironmentAuthority);
    }
    refs(connection, owner, false, |name| {
        if environment_name_valid(name) && admission.baseline.contains(name) && !own.contains(name)
        {
            bail!(StateGuardError::EnvironmentAuthority);
        }
        Ok(())
    })
}
impl Store {
    /// Initial selection and final transactional admission use the identical policy.
    pub(crate) fn check_environment_admission(
        &self,
        owner: ProjectId,
        admission: &EnvironmentAdmission,
    ) -> Result<()> {
        evaluate(&self.connection, owner, admission)
    }
    /// Explicit owning operator check only; no foreign query/activity/source validation.
    pub(crate) fn environment_candidates(
        &self,
        owner: ProjectId,
        admission: &EnvironmentAdmission,
    ) -> Result<BTreeSet<String>> {
        let exists: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM projects WHERE id=?1",
                [owner.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            bail!(StateGuardError::EnvironmentAuthority);
        }
        Ok(admission.candidates(
            &own_refs(&self.connection, owner)?
                .into_iter()
                .collect::<Vec<_>>(),
        ))
    }
}
