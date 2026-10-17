//! Schema evidence comparison, independent of the hand-authored wire decoder.
//! All traversal is of schema positions; defaults/enum/const remain JSON values.
use crate::execution::codex_wire::{self, CodexWireContract, Ownership};
use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const FILES: &[(&str, &str)] = &[
    ("ClientNotification", "ClientNotification.json"),
    ("ClientRequest_used", "ClientRequest.json"),
    ("ServerNotification", "ServerNotification.json"),
    ("ServerRequest", "ServerRequest.json"),
    ("InitializeResponse", "v1/InitializeResponse.json"),
    ("GetAccountResponse", "v2/GetAccountResponse.json"),
    (
        "GetAccountRateLimitsResponse",
        "v2/GetAccountRateLimitsResponse.json",
    ),
    (
        "EnvironmentStatusResponse",
        "v2/EnvironmentStatusResponse.json",
    ),
    ("ThreadStartResponse", "v2/ThreadStartResponse.json"),
    ("TurnStartResponse", "v2/TurnStartResponse.json"),
    ("ConfigReadResponse", "v2/ConfigReadResponse.json"),
    (
        "ListMcpServerStatusResponse",
        "v2/ListMcpServerStatusResponse.json",
    ),
];
const BASIS: &str =
    include_str!("../../../../../doc/verification/macos-minimum-m0-catalog-0162.json");
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Reason {
    StrictMismatch,
    StrictReach,
    OwnershipKey,
    ServerRequestAdded,
    StrictRemoved,
    AdditionLimit,
    AdditionName,
    Ref,
    Limit,
    File,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mismatch {
    Added,
    Removed,
    Type,
    Required,
    Enum,
    Branch,
    Constraint,
    Closed,
    Default,
}
#[derive(Debug, Serialize)]
pub(crate) struct Receipt {
    pub(crate) pass: bool,
    pub(crate) reason: Option<Reason>,
    pub(crate) kind: Option<Mismatch>,
    pub(crate) positions: Vec<String>,
    pub(crate) tolerated_changes: usize,
    pub(crate) accepted_item_types: usize,
    pub(crate) accepted_notifications: usize,
    pub(crate) additions_sha256: String,
}
pub(crate) struct Gate {
    pub(crate) receipt: Receipt,
    pub(crate) contract: Option<CodexWireContract>,
}
pub(crate) fn file_refused(error: &anyhow::Error) -> Receipt {
    Receipt {
        pass: false,
        reason: Some(
            error
                .downcast_ref::<Refusal>()
                .map_or(Reason::File, |e| e.0),
        ),
        kind: None,
        positions: vec![],
        tolerated_changes: 0,
        accepted_item_types: 0,
        accepted_notifications: 0,
        additions_sha256: String::new(),
    }
}
#[derive(Debug)]
struct Refusal(Reason, Option<Mismatch>, String);
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "schema {:?}", self.0)
    }
}
impl std::error::Error for Refusal {}
fn refuse(reason: Reason, path: &str) -> anyhow::Error {
    Refusal(reason, None, path.into()).into()
}
fn mismatch(kind: Mismatch, path: &str) -> anyhow::Error {
    Refusal(Reason::StrictMismatch, Some(kind), path.into()).into()
}
fn position(path: &str) -> String {
    if path.len() <= 96
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" ._()+/:-".contains(&b))
    {
        path.into()
    } else {
        format!(
            "nonrecordable:{}",
            crate::execution::native_result::digest(path.as_bytes())
        )
    }
}
fn children(s: &Value) -> Result<Vec<(&str, &Value)>> {
    let Some(o) = s.as_object() else {
        ensure!(s.is_boolean(), "invalid schema node");
        return Ok(vec![]);
    };
    let mut out = vec![];
    for (k, v) in o {
        match k.as_str() {
            "description" | "title" | "examples" | "$schema" | "$ref" | "type" | "enum"
            | "const" | "default" | "required" | "format" | "minimum" | "maximum"
            | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf" | "minLength"
            | "maxLength" | "pattern" | "minItems" | "maxItems" | "uniqueItems"
            | "minProperties" | "maxProperties" | "readOnly" | "writeOnly" | "deprecated" => {}
            "properties" | "patternProperties" | "definitions" | "$defs" | "dependentSchemas" => {
                for (n, v) in v.as_object().context("schema map required")? {
                    out.push((n.as_str(), v));
                }
            }
            "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                for v in v.as_array().context("schema list required")? {
                    out.push((k, v));
                }
            }
            "items" if v.is_array() => {
                for v in v.as_array().unwrap() {
                    out.push((k, v));
                }
            }
            "additionalProperties"
            | "additionalItems"
            | "items"
            | "contains"
            | "propertyNames"
            | "not"
            | "if"
            | "then"
            | "else"
            | "unevaluatedProperties"
            | "unevaluatedItems" => out.push((k, v)),
            _ => bail!("unsupported schema keyword"),
        }
    }
    Ok(out)
}
/// Count reference expansion and schema nesting together, independently of memoization.
#[derive(Clone, Copy, Default)]
struct Depth {
    refs: usize,
    json: usize,
}
impl Depth {
    fn check(self) -> Result<()> {
        ensure!(
            self.refs <= 16 && self.json <= 64,
            Refusal(Reason::Limit, None, "traversal depth".into())
        );
        Ok(())
    }
    fn reference(self) -> Self {
        Self {
            refs: self.refs + 1,
            ..self
        }
    }
    fn child(self) -> Self {
        Self {
            json: self.json + 1,
            ..self
        }
    }
}
struct Document {
    root: Value,
    defs: BTreeMap<String, Value>,
}
impl Document {
    fn basis(catalog: &Value, source: &str) -> Self {
        let s = &catalog["schema_contract_catalog"]["source_schemas"][source];
        let defs = s["local_definition_digests"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(n, h)| {
                (
                    n.clone(),
                    catalog["schema_contract_catalog"]["shared_definitions"][h.as_str().unwrap()]
                        .clone(),
                )
            })
            .collect();
        Self {
            root: s["schema"].clone(),
            defs,
        }
    }
    fn live(mut root: Value) -> Result<Self> {
        let defs = root
            .as_object_mut()
            .context("schema root object required")?
            .remove("definitions")
            .unwrap_or_else(|| json!({}));
        Ok(Self {
            root,
            defs: serde_json::from_value(defs)?,
        })
    }
    fn target<'a>(&'a self, s: &Value) -> Result<Option<(&'a str, &'a Value)>> {
        let Some(r) = s.get("$ref") else {
            return Ok(None);
        };
        let name = r
            .as_str()
            .and_then(|r| r.strip_prefix("#/definitions/"))
            .filter(|r| !r.is_empty() && !r.contains(['/', '~']))
            .ok_or_else(|| refuse(Reason::Ref, "ref"))?;
        let (name, target) = self
            .defs
            .get_key_value(name)
            .ok_or_else(|| refuse(Reason::Ref, "ref"))?;
        Ok(Some((name, target)))
    }
    fn bounds(&self) -> Result<()> {
        fn visit(
            d: &Document,
            s: &Value,
            stack: &mut BTreeSet<String>,
            memo: &mut BTreeSet<String>,
            work: &mut usize,
            depth: Depth,
        ) -> Result<()> {
            depth.check()?;
            *work += 1;
            ensure!(*work <= 262144, Refusal(Reason::Limit, None, "work".into()));
            if let Some((name, target)) = d.target(s)? {
                ensure!(
                    !stack.contains(name),
                    Refusal(Reason::Ref, None, "cycle".into())
                );
                if !memo.contains(name) {
                    ensure!(
                        stack.len() < 16 && memo.len() + stack.len() < 512,
                        Refusal(Reason::Limit, None, "ref bound".into())
                    );
                    stack.insert(name.into());
                    visit(d, target, stack, memo, work, depth.reference())?;
                    stack.remove(name);
                    memo.insert(name.into());
                }
            }
            for (_, s) in children(s)? {
                visit(d, s, stack, memo, work, depth.child())?;
            }
            Ok(())
        }
        visit(
            self,
            &self.root,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut 0,
            Depth::default(),
        )
    }
    fn resolve<'a>(&'a self, mut s: &'a Value) -> Result<&'a Value> {
        for _ in 0..=16 {
            match self.target(s)? {
                None => return Ok(s),
                Some((_, t)) => s = t,
            }
        }
        Err(refuse(Reason::Limit, "ref depth"))
    }
}
fn tag(s: &Value, key: &str) -> Option<String> {
    let a = s["properties"][key]["enum"].as_array()?;
    (a.len() == 1)
        .then(|| a[0].as_str().map(str::to_owned))
        .flatten()
}
fn branches(s: &Value, key: &str) -> Result<BTreeMap<String, Value>> {
    let mut out = BTreeMap::new();
    for branch in s["oneOf"]
        .as_array()
        .context("method/variant branches absent")?
    {
        let name = tag(branch, key).context("method/variant is not a singleton enum")?;
        ensure!(
            out.insert(name, branch.clone()).is_none(),
            "duplicate method/variant"
        );
    }
    Ok(out)
}
fn annotation(k: &str) -> bool {
    matches!(k, "description" | "title" | "examples" | "$schema")
}

/// Remove only tolerated positions; field values inside strict keywords stay intact.
fn project(s: &Value, name: &str) -> Value {
    let Some(o) = s.as_object() else {
        return s.clone();
    };
    let mut p = o.clone();
    let keep: Option<&[&str]> = match name {
        "Thread" => Some(&["id", "cwd", "modelProvider", "status", "turns"]),
        // Reserved refinement keys must remain visible so a new declaration is refused.
        "Config" => Some(&["web_search", "features", "mcp_servers"]),
        "McpServerStatus" => Some(&["name", "tools", "toolsError"]),
        _ => None,
    };
    if let Some(keep) = keep {
        if let Some(v) = p.get_mut("properties").and_then(Value::as_object_mut) {
            v.retain(|k, _| keep.contains(&k.as_str()));
        }
        if let Some(v) = p.get_mut("required").and_then(Value::as_array_mut) {
            v.retain(|k| keep.contains(&k.as_str().unwrap_or("")));
        }
    }
    match name {
        "TurnError" => {
            if let Some(v) = p.get_mut("properties").and_then(Value::as_object_mut) {
                v.remove("misalignment");
            }
        }
        "ConfigReadResponse" => {
            if p["properties"].get("origins").is_some() {
                p["properties"]["origins"]["additionalProperties"] = json!(true);
            }
            if p["properties"].get("layers").is_some() {
                p["properties"]["layers"]["items"] = json!(true);
            }
        }
        "McpServerStatus" => {
            if p["properties"].get("tools").is_some() {
                p["properties"]["tools"]["additionalProperties"] = json!(true);
            }
        }
        _ => {}
    }
    Value::Object(p)
}
struct Compare<'a> {
    basis: &'a Document,
    live: &'a Document,
    names: BTreeMap<String, String>, // live definition -> basis definition
    visited: BTreeSet<(String, String)>,
    strict: BTreeSet<String>,
    work: usize,
    additions: &'a mut CodexWireContract,
}
fn ref_siblings(target: Value, reference: &Value) -> Value {
    let mut siblings = reference.as_object().unwrap().clone();
    siblings.retain(|k, _| k != "$ref" && !annotation(k));
    if siblings.is_empty() {
        return target;
    }
    if let Some(mut merged) = target.as_object().cloned()
        && siblings
            .iter()
            .all(|(k, v)| merged.get(k).is_none_or(|t| t == v))
    {
        merged.extend(siblings);
        Value::Object(merged)
    } else {
        // Both constraints apply when a keyword occurs on the target and the reference.
        json!({"allOf":[target, Value::Object(siblings)]})
    }
}
impl Compare<'_> {
    fn node(&mut self, a: &Value, b: &Value, name: &str, path: &str) -> Result<()> {
        self.node_at(a, b, name, path, Depth::default())
    }
    fn node_at(
        &mut self,
        a: &Value,
        b: &Value,
        name: &str,
        path: &str,
        depth: Depth,
    ) -> Result<()> {
        depth.check()?;
        self.work += 1;
        ensure!(
            self.work <= 262144,
            Refusal(Reason::Limit, None, path.into())
        );
        let ar = self.basis.target(a)?;
        let br = self.live.target(b)?;
        if ar.is_some() || br.is_some() {
            // `$ref` is a schema keyword, not permission to discard sibling constraints/defaults.
            let (an, av) = ar.unwrap_or((name, a));
            let (bn, bv) = br.unwrap_or((name, b));
            if ar.is_some() {
                self.strict.insert(an.into());
            }
            if ar.is_some() && br.is_some() {
                self.names.insert(bn.into(), an.into());
                if !self.visited.insert((an.into(), bn.into()))
                    && [a, b].iter().all(|s| {
                        s.as_object()
                            .unwrap()
                            .keys()
                            .all(|k| k == "$ref" || annotation(k))
                    })
                {
                    return Ok(());
                }
            }
            return self.node_at(
                &if ar.is_some() {
                    ref_siblings(project(av, an), a)
                } else {
                    project(av, an)
                },
                &if br.is_some() {
                    ref_siblings(project(bv, an), b)
                } else {
                    project(bv, an)
                },
                an,
                &format!("{path}/{an}"),
                depth.reference(),
            );
        }
        if name == "Config" {
            ensure!(
                b["properties"].get("features").is_none()
                    && b["properties"].get("mcp_servers").is_none(),
                Refusal(Reason::StrictMismatch, Some(Mismatch::Added), path.into())
            );
        }
        if name == "ThreadItem" && a.get("oneOf").is_some() {
            let remainder = |v: &Value| {
                let mut o = v.as_object().unwrap().clone();
                o.remove("oneOf");
                Value::Object(o)
            };
            self.node_at(&remainder(a), &remainder(b), "", path, depth.child())?;
            let aa = branches(a, "type")?;
            let bb = branches(b, "type")?;
            for (kind, av) in &aa {
                let bv = bb
                    .get(kind)
                    .ok_or_else(|| mismatch(Mismatch::Removed, path))?;
                if ["agentMessage", "commandExecution", "fileChange"].contains(&kind.as_str()) {
                    self.node_at(av, bv, "", &format!("{path}/{kind}"), depth.child())?;
                } else {
                    self.node_at(
                        &item_keys(av),
                        &item_keys(bv),
                        "",
                        &format!("{path}/{kind}"),
                        depth.child(),
                    )?;
                }
            }
            for kind in bb.keys().filter(|k| !aa.contains_key(*k)) {
                ensure!(
                    valid_name(kind, 64, false),
                    Refusal(Reason::AdditionName, None, path.into())
                );
                let branch = &bb[kind];
                let keys = item_keys(branch);
                ensure!(
                    keys["type"] == "object"
                        && keys["properties"]["id"] == json!({"type":"string"})
                        && keys["properties"]["type"] == json!({"type":"string","enum":[kind]})
                        && keys["required"] == json!(["id", "type"]),
                    Refusal(Reason::OwnershipKey, None, path.into())
                );
                if !self.additions.items.contains(kind) {
                    self.additions.items.push(kind.clone());
                }
                ensure!(
                    self.additions.items.len() <= 32,
                    Refusal(Reason::AdditionLimit, None, path.into())
                );
            }
            return Ok(());
        }
        let (Some(ao), Some(bo)) = (a.as_object(), b.as_object()) else {
            ensure!(
                a == b,
                Refusal(Reason::StrictMismatch, Some(Mismatch::Type), path.into())
            );
            return Ok(());
        };
        let ak = ao
            .keys()
            .filter(|k| !annotation(k))
            .collect::<BTreeSet<_>>();
        let bk = bo
            .keys()
            .filter(|k| !annotation(k))
            .collect::<BTreeSet<_>>();
        if ak != bk {
            return Err(mismatch(
                if bk.is_superset(&ak) {
                    Mismatch::Added
                } else {
                    Mismatch::Removed
                },
                path,
            ));
        }
        for k in ak {
            let av = &ao[k];
            let bv = &bo[k];
            let p = format!("{path}/{k}");
            match k.as_str() {
                "properties" | "patternProperties" | "dependentSchemas" => {
                    let am = av.as_object().context("schema map")?;
                    let bm = bv.as_object().context("schema map")?;
                    if am.keys().collect::<Vec<_>>() != bm.keys().collect::<Vec<_>>() {
                        return Err(mismatch(
                            if bm.len() > am.len() {
                                Mismatch::Added
                            } else {
                                Mismatch::Removed
                            },
                            &p,
                        ));
                    }
                    for (key, a) in am {
                        self.node_at(a, &bm[key], "", &format!("{p}/{key}"), depth.child())?;
                    }
                }
                "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                    let aa = av.as_array().context("schema branches")?;
                    let bb = bv.as_array().context("schema branches")?;
                    if aa.len() != bb.len() {
                        return Err(mismatch(Mismatch::Branch, &p));
                    }
                    // Tags identify branches independently of declaration order.
                    let mut used = BTreeSet::new();
                    for a in aa {
                        let mut found = None;
                        for (i, b) in bb.iter().enumerate().filter(|(i, _)| !used.contains(i)) {
                            let saved = (
                                self.names.clone(),
                                self.visited.clone(),
                                self.strict.clone(),
                            );
                            match self.node_at(a, b, "", &p, depth.child()) {
                                Ok(()) => {
                                    found = Some(i);
                                    break;
                                }
                                Err(e)
                                    if e.downcast_ref::<Refusal>()
                                        .is_some_and(|r| r.0 == Reason::Limit) =>
                                {
                                    return Err(e);
                                }
                                Err(_) => {}
                            }
                            (self.names, self.visited, self.strict) = saved;
                        }
                        used.insert(found.ok_or_else(|| mismatch(Mismatch::Branch, &p))?);
                    }
                }
                "additionalProperties" if av.is_boolean() && bv.is_boolean() && av != bv => {
                    return Err(mismatch(Mismatch::Closed, &p));
                }
                "additionalProperties"
                | "additionalItems"
                | "items"
                | "contains"
                | "propertyNames"
                | "not"
                | "if"
                | "then"
                | "else"
                | "unevaluatedProperties"
                | "unevaluatedItems" => self.node_at(av, bv, "", &p, depth.child())?,
                _ => {
                    let equal = if matches!(k.as_str(), "enum" | "required" | "type")
                        && av.is_array()
                        && bv.is_array()
                    {
                        value_set(av) == value_set(bv)
                    } else {
                        av == bv
                    };
                    if !equal {
                        return Err(mismatch(
                            match k.as_str() {
                                "type" | "format" => Mismatch::Type,
                                "required" => Mismatch::Required,
                                "enum" | "const" => Mismatch::Enum,
                                "default" => Mismatch::Default,
                                "additionalProperties" => Mismatch::Closed,
                                _ => Mismatch::Constraint,
                            },
                            &p,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
fn value_set(v: &Value) -> BTreeSet<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect()
}
fn valid_name(s: &str, limit: usize, method: bool) -> bool {
    !s.is_empty()
        && s.len() <= limit
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || method && b"/.-".contains(&b))
}
fn item_keys(s: &Value) -> Value {
    let mut p = s.clone();
    if let Some(m) = p.get_mut("properties").and_then(Value::as_object_mut) {
        m.retain(|k, _| matches!(k.as_str(), "id" | "type"));
    }
    if let Some(a) = p.get_mut("required").and_then(Value::as_array_mut) {
        a.retain(|v| v == "id" || v == "type");
        a.sort_by_key(|v| v.as_str().unwrap_or("").to_owned());
    }
    p
}

fn owner_schema(d: &Document, branch: &Value) -> Result<(Ownership, Value)> {
    let p = d.resolve(&branch["properties"]["params"])?;
    let required = p["required"].as_array().cloned().unwrap_or_default();
    let mut fields = serde_json::Map::new();
    let mut req = vec![];
    for key in ["threadId", "turnId"] {
        if let Some(s) = p["properties"].get(key) {
            let s = d.resolve(s)?;
            owner_nullable(d, s)?;
            fields.insert(key.into(), s.clone());
            if required.contains(&json!(key)) {
                req.push(json!(key));
            }
        }
    }
    // Known nested ownership is compared through its strict payload instead.
    let method = tag(branch, "method").context("method absent")?;
    let owner = if let Some(known) = codex_wire::notification_owner(&method) {
        known
    } else {
        Ownership::Added
    };
    Ok((owner, json!({"properties":fields,"required":req})))
}
fn owner_nullable(d: &Document, s: &Value) -> Result<bool> {
    fn domain(d: &Document, s: &Value, depth: Depth) -> Result<u8> {
        depth.check()?;
        if let Some((_, target)) = d.target(s)? {
            return domain(d, target, depth.reference());
        }
        if let Some(t) = s.get("type") {
            let mut bits = 0;
            for ty in t.as_array().map_or_else(|| vec![t], |a| a.iter().collect()) {
                bits |= match ty.as_str() {
                    Some("string") => 1,
                    Some("null") => 2,
                    _ => return Err(refuse(Reason::OwnershipKey, "ownership type")),
                };
            }
            return Ok(bits);
        }
        for key in ["anyOf", "oneOf", "allOf"] {
            if let Some(a) = s.get(key).and_then(Value::as_array) {
                ensure!(
                    !a.is_empty(),
                    Refusal(Reason::OwnershipKey, None, "ownership union".into())
                );
                let mut bits = if key == "allOf" { 3 } else { 0 };
                for branch in a {
                    let b = domain(d, branch, depth.child())?;
                    if key == "allOf" {
                        bits &= b;
                    } else {
                        bits |= b;
                    }
                }
                return Ok(bits);
            }
        }
        Err(refuse(Reason::OwnershipKey, "ownership type"))
    }
    match domain(d, s, Depth::default())? {
        1 => Ok(false),
        3 => Ok(true),
        _ => Err(refuse(Reason::OwnershipKey, "ownership type")),
    }
}
fn anchors(
    d: &Document,
    names: &BTreeMap<String, String>,
    strict: &BTreeSet<String>,
    notifications: bool,
) -> Result<BTreeMap<String, Value>> {
    let mut out = BTreeMap::new();
    if notifications {
        for (method, branch) in branches(&d.root, "method")? {
            if codex_wire::STRICT_NOTIFICATIONS.contains(&method.as_str()) {
                continue;
            }
            let mut p = branch["properties"]["params"].clone();
            if let Some(o) = p.get_mut("properties").and_then(Value::as_object_mut) {
                o.remove("threadId");
                o.remove("turnId");
            }
            out.insert(method, p);
        }
    }
    for (local, def) in &d.defs {
        let name = names.get(local).map_or(local.as_str(), String::as_str);
        if !strict.contains(name) {
            continue;
        }
        match name {
            "ThreadItem" => {
                for (kind, mut branch) in branches(def, "type")? {
                    if ["agentMessage", "commandExecution", "fileChange"].contains(&kind.as_str()) {
                        continue;
                    }
                    if let Some(o) = branch.get_mut("properties").and_then(Value::as_object_mut) {
                        o.remove("id");
                        o.remove("type");
                    }
                    out.insert(format!("ThreadItem[{kind}]"), branch);
                }
            }
            "Thread" | "Config" | "TurnError" | "McpServerStatus" => {
                let kept = project(def, name);
                for (key, v) in def["properties"]
                    .as_object()
                    .context("mixed properties absent")?
                {
                    if kept["properties"].get(key).is_none() {
                        out.insert(format!("{name}.{key}"), v.clone());
                    }
                }
                if name == "McpServerStatus" {
                    out.insert(
                        "McpServerStatus.tools".into(),
                        def["properties"]["tools"]["additionalProperties"].clone(),
                    );
                }
            }
            _ => {}
        }
    }
    if d.root["properties"].get("origins").is_some() && d.root["properties"].get("config").is_some()
    {
        out.insert(
            "ConfigReadResponse.origins".into(),
            d.root["properties"]["origins"].clone(),
        );
        out.insert(
            "ConfigReadResponse.layers".into(),
            d.root["properties"]["layers"]["items"].clone(),
        );
    }
    Ok(out)
}
fn reaches(
    d: &Document,
    anchors: &BTreeMap<String, Value>,
    names: &BTreeMap<String, String>,
    strict: &BTreeSet<String>,
) -> Result<BTreeSet<(String, String)>> {
    fn walk(
        d: &Document,
        s: &Value,
        policy: (&BTreeMap<String, String>, &BTreeSet<String>),
        seen: &mut BTreeSet<String>,
        found: &mut BTreeSet<String>,
        work: &mut usize,
        depth: Depth,
    ) -> Result<()> {
        let (names, strict) = policy;
        depth.check()?;
        *work += 1;
        ensure!(
            *work <= 262144,
            Refusal(Reason::Limit, None, "reach work".into())
        );
        if let Some((n, t)) = d.target(s)? {
            let mapped = names.get(n).map_or(n, String::as_str);
            if strict.contains(mapped) {
                found.insert(mapped.into());
            }
            if seen.insert(n.into()) {
                walk(d, t, policy, seen, found, work, depth.reference())?;
            }
        }
        for (_, s) in children(s)? {
            walk(d, s, policy, seen, found, work, depth.child())?;
        }
        Ok(())
    }
    let mut out = BTreeSet::new();
    let mut work = 0;
    for (anchor, s) in anchors {
        let mut found = BTreeSet::new();
        walk(
            d,
            s,
            (names, strict),
            &mut BTreeSet::new(),
            &mut found,
            &mut work,
            Depth::default(),
        )?;
        for n in found {
            out.insert((anchor.clone(), n));
        }
    }
    Ok(out)
}
fn structured(d: &Document, s: &Value, seen: &mut BTreeSet<String>) -> Result<bool> {
    structured_at(d, s, seen, Depth::default())
}
fn structured_at(
    d: &Document,
    s: &Value,
    seen: &mut BTreeSet<String>,
    depth: Depth,
) -> Result<bool> {
    depth.check()?;
    if let Some((n, t)) = d.target(s)?
        && seen.insert(n.into())
    {
        return structured_at(d, t, seen, depth.reference());
    }
    if s.as_object().is_some_and(|o| {
        [
            "$ref",
            "properties",
            "patternProperties",
            "items",
            "prefixItems",
        ]
        .iter()
        .any(|k| o.contains_key(*k))
            || o.get("additionalProperties").is_some_and(|v| v.is_object())
    }) {
        return Ok(true);
    }
    if s["type"] == "object"
        || s["type"] == "array"
        || s["type"]
            .as_array()
            .is_some_and(|a| a.contains(&json!("object")) || a.contains(&json!("array")))
    {
        return Ok(true);
    }
    for (_, s) in children(s)? {
        if structured_at(d, s, seen, depth.child())? {
            return Ok(true);
        }
    }
    Ok(false)
}
fn compare_source(
    source: &str,
    basis: &Document,
    live: &Document,
    contract: &mut CodexWireContract,
    receipt: &mut Receipt,
) -> Result<()> {
    basis.bounds()?;
    live.bounds()?;
    let mut cmp = Compare {
        basis,
        live,
        names: BTreeMap::new(),
        visited: BTreeSet::new(),
        strict: BTreeSet::new(),
        work: 0,
        additions: contract,
    };
    match source {
        "ClientRequest_used" | "ServerNotification" | "ServerRequest" => {
            let aa = branches(&basis.root, "method")?;
            let bb = branches(&live.root, "method")?;
            // Common root declarations (not method branches), including emittedAtMs.
            let mut a = basis.root.clone();
            a.as_object_mut().unwrap().remove("oneOf");
            let mut b = live.root.clone();
            b.as_object_mut().unwrap().remove("oneOf");
            cmp.node(&a, &b, "", source)?;
            for (method, a) in &aa {
                let strict = source != "ServerNotification"
                    || codex_wire::STRICT_NOTIFICATIONS.contains(&method.as_str());
                let Some(b) = bb.get(method) else {
                    ensure!(!strict, Refusal(Reason::StrictRemoved, None, method.into()));
                    continue;
                };
                if strict {
                    cmp.node(a, b, "", method)?;
                } else {
                    let (_, ao) = owner_schema(basis, a)?;
                    let (_, bo) = owner_schema(live, b)?;
                    cmp.node(&ao, &bo, "", method)
                        .map_err(|_| refuse(Reason::OwnershipKey, method))?;
                    // The envelope remains strict while params content is tolerated.
                    let mut a = a.clone();
                    a["properties"]["params"] = json!(true);
                    let mut b = b.clone();
                    b["properties"]["params"] = json!(true);
                    cmp.node(&a, &b, "", method)?;
                }
            }
            if source != "ClientRequest_used" {
                for (method, b) in bb.iter().filter(|(m, _)| !aa.contains_key(*m)) {
                    ensure!(
                        source == "ServerNotification",
                        Refusal(Reason::ServerRequestAdded, None, method.clone())
                    );
                    ensure!(
                        valid_name(method, 128, true) && !codex_wire::reserved_method(method),
                        Refusal(Reason::AdditionName, None, method.clone())
                    );
                    ensure!(
                        b["required"].is_array()
                            && value_set(&b["required"]) == value_set(&json!(["method", "params"]))
                            && b["properties"].get("id").is_none(),
                        Refusal(Reason::OwnershipKey, None, method.clone())
                    );
                    let (_, keys) = owner_schema(live, b)?;
                    let key = |name: &str| {
                        keys["properties"].get(name).map(|v| codex_wire::Key {
                            required: keys["required"].as_array().unwrap().contains(&json!(name)),
                            nullable: owner_nullable(live, v).unwrap(),
                        })
                    };
                    cmp.additions.notifications.insert(
                        method.clone(),
                        codex_wire::AddedNotification {
                            thread: key("threadId"),
                            turn: key("turnId"),
                        },
                    );
                    ensure!(
                        cmp.additions.notifications.len() <= 64,
                        Refusal(Reason::AdditionLimit, None, method.clone())
                    );
                }
            }
        }
        _ => cmp.node(
            &project(&basis.root, source),
            &project(&live.root, source),
            source,
            source,
        )?,
    }
    let ba = anchors(
        basis,
        &BTreeMap::new(),
        &cmp.strict,
        source == "ServerNotification",
    )?;
    let la = anchors(
        live,
        &cmp.names,
        &cmp.strict,
        source == "ServerNotification",
    )?;
    let br = reaches(basis, &ba, &BTreeMap::new(), &cmp.strict)?;
    let lr = reaches(live, &la, &cmp.names, &cmp.strict)?;
    for (anchor, name) in lr.difference(&br) {
        if structured(basis, &basis.defs[name], &mut BTreeSet::new())? {
            return Err(refuse(
                Reason::StrictReach,
                &format!("{source}/{anchor}/{name}"),
            ));
        }
    }
    receipt.tolerated_changes += ba
        .keys()
        .chain(la.keys())
        .collect::<BTreeSet<_>>()
        .iter()
        .filter(|k| ba.get(**k) != la.get(**k))
        .count();
    Ok(())
}
pub(crate) fn compare(files: &BTreeMap<String, Value>) -> Gate {
    let mut receipt = Receipt {
        pass: false,
        reason: None,
        kind: None,
        positions: vec![],
        tolerated_changes: 0,
        accepted_item_types: 0,
        accepted_notifications: 0,
        additions_sha256: String::new(),
    };
    let mut contract = CodexWireContract::default();
    let result = (|| -> Result<()> {
        let catalog: Value = serde_json::from_str(BASIS)?;
        for (source, _) in FILES {
            let basis = Document::basis(&catalog, source);
            let mut live = Document::live(
                files
                    .get(*source)
                    .ok_or_else(|| refuse(Reason::File, source))?
                    .clone(),
            )?;
            if *source == "ClientRequest_used" {
                let used = branches(&basis.root, "method")?;
                let selected = branches(&live.root, "method")?
                    .into_iter()
                    .filter(|(m, _)| used.contains_key(m))
                    .map(|(_, b)| b)
                    .collect::<Vec<_>>();
                live.root["oneOf"] = json!(selected);
            }
            compare_source(source, &basis, &live, &mut contract, &mut receipt)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        if let Some(Refusal(reason, kind, path)) = e.downcast_ref::<Refusal>() {
            receipt.reason = Some(*reason);
            receipt.kind = *kind;
            receipt.positions.push(position(path));
        } else {
            receipt.reason = Some(Reason::File);
        }
        return Gate {
            receipt,
            contract: None,
        };
    }
    receipt.pass = true;
    receipt.accepted_item_types = contract.items.len();
    receipt.accepted_notifications = contract.notifications.len();
    receipt.additions_sha256 = crate::execution::native_result::digest(
        &serde_json::to_vec(&(&contract.items, &contract.notifications)).unwrap(),
    );
    Gate {
        receipt,
        contract: Some(contract),
    }
}

/// Inspect every generated entry for the file-sink budget, then read only the manifest.
pub(crate) fn read_directory(root: &std::fs::File) -> Result<BTreeMap<String, Value>> {
    use rustix::fs::{Mode, OFlags, openat};
    use std::{fs::File, io::Read, os::unix::fs::MetadataExt};
    const FLAGS: OFlags = OFlags::RDONLY
        .union(OFlags::NOFOLLOW)
        .union(OFlags::NONBLOCK)
        .union(OFlags::CLOEXEC);
    fn inspect(dir: &File, depth: usize, count: &mut usize, bytes: &mut u64) -> Result<()> {
        ensure!(
            depth <= 4,
            Refusal(Reason::Limit, None, "directory depth".into())
        );
        for entry in rustix::fs::Dir::read_from(dir)? {
            let e = entry?;
            let name = e.file_name();
            if matches!(name.to_bytes(), b"." | b"..") {
                continue;
            }
            *count += 1;
            ensure!(
                *count <= 1024,
                Refusal(Reason::Limit, None, "entries".into())
            );
            let f = File::from(openat(dir, name, FLAGS, Mode::empty())?);
            let m = f.metadata()?;
            ensure!(
                m.uid() == rustix::process::getuid().as_raw(),
                "schema owner mismatch"
            );
            if m.is_dir() {
                inspect(&f, depth + 1, count, bytes)?;
            } else {
                ensure!(m.is_file() && m.nlink() == 1, "schema unsafe entry");
                *bytes = bytes.checked_add(m.len()).context("schema size overflow")?;
                ensure!(
                    *bytes <= 16 * 1024 * 1024,
                    Refusal(Reason::Limit, None, "file bytes".into())
                );
            }
        }
        Ok(())
    }
    inspect(root, 0, &mut 0, &mut 0)?;
    let mut out = BTreeMap::new();
    for (source, path) in FILES {
        let mut dir = root.try_clone()?;
        let components = path.split('/').collect::<Vec<_>>();
        for component in &components[..components.len() - 1] {
            dir = File::from(openat(
                &dir,
                *component,
                FLAGS | OFlags::DIRECTORY,
                Mode::empty(),
            )?);
        }
        let mut file = File::from(openat(
            &dir,
            *components.last().unwrap(),
            FLAGS,
            Mode::empty(),
        )?);
        let before = file.metadata()?;
        ensure!(
            before.is_file() && before.nlink() == 1,
            "schema unsafe file"
        );
        ensure!(
            before.len() <= 1024 * 1024,
            Refusal(Reason::Limit, None, "file bytes".into())
        );
        let mut bytes = vec![];
        file.by_ref()
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        let after = file.metadata()?;
        ensure!(
            bytes.len() as u64 == before.len()
                && (
                    before.len(),
                    before.mtime(),
                    before.mtime_nsec(),
                    before.ctime(),
                    before.ctime_nsec()
                ) == (
                    after.len(),
                    after.mtime(),
                    after.mtime_nsec(),
                    after.ctime(),
                    after.ctime_nsec()
                ),
            "schema file changed"
        );
        let value = crate::execution::strict_json::decode_schema(&bytes).map_err(|e| {
            use crate::execution::strict_json::Error;
            refuse(
                if matches!(
                    e,
                    Error::FrameBytes
                        | Error::Depth
                        | Error::Nodes
                        | Error::StringBytes
                        | Error::TotalStringBytes
                        | Error::ObjectEntries
                        | Error::ArrayEntries
                ) {
                    Reason::Limit
                } else {
                    Reason::File
                },
                source,
            )
        })?;
        out.insert((*source).into(), value);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> BTreeMap<String, Value> {
        serde_json::from_value(
            serde_json::from_str::<Value>(include_str!(
                "../../../tests/fixtures/codex-contract/basis-pair.json"
            ))
            .unwrap()["files"]
                .clone(),
        )
        .unwrap()
    }
    fn definition<'a>(
        files: &'a mut BTreeMap<String, Value>,
        source: &str,
        name: &str,
    ) -> &'a mut Value {
        &mut files.get_mut(source).unwrap()["definitions"][name]
    }
    /// Memoized preflight targets cannot bypass traversal depth refusals.
    #[test]
    fn u4r1a_memo_prefilled_chain_refuses_limit() {
        let mut defs = BTreeMap::new();
        let mut prefill = serde_json::Map::new();
        for i in 0..511 {
            let name = format!("D{i:03}");
            let mut target = if i == 0 {
                json!({"type":"string"})
            } else {
                json!({"$ref":format!("#/definitions/D{:03}", i - 1)})
            };
            for _ in 0..24 {
                target = json!({"items":target});
            }
            defs.insert(name.clone(), target);
            prefill.insert(
                name.clone(),
                json!({"$ref":format!("#/definitions/{name}")}),
            );
        }
        let notification = |method: &str, params: Value| {
            json!({
                "type":"object", "properties":{"method":{"type":"string","enum":[method]},"params":params},
                "required":["method","params"]
            })
        };
        let basis = Document {
            root: json!({"oneOf":[]}),
            defs: BTreeMap::new(),
        };
        let live = Document {
            root: json!({"oneOf":[
                notification("aaa/prefill", json!({"properties":prefill})),
                notification("zzz/chain", json!({"$ref":"#/definitions/D510"}))
            ]}),
            defs,
        };
        // The reverse chain is already memoized before the final anchor is visited.
        live.bounds().unwrap();
        let mut receipt = compare(&fixture()).receipt;
        let error = compare_source(
            "ServerNotification",
            &basis,
            &live,
            &mut CodexWireContract::default(),
            &mut receipt,
        )
        .unwrap_err();
        assert_eq!(error.downcast_ref::<Refusal>().unwrap().0, Reason::Limit);
    }
    /// Every recursive schema consumer enforces both reference and JSON depth.
    #[test]
    fn u4r1a_recursive_consumers_are_bounded() {
        for (refs, nesting) in [(17, 0), (1, 65)] {
            let mut defs = BTreeMap::new();
            for i in 0..refs {
                let mut v = if i + 1 == refs {
                    json!({"type":"string"})
                } else {
                    json!({"$ref":format!("#/definitions/D{}", i + 1)})
                };
                for _ in 0..nesting {
                    v = json!({"allOf":[v]});
                }
                defs.insert(format!("D{i}"), v);
            }
            let d = Document {
                root: json!({"$ref":"#/definitions/D0"}),
                defs,
            };
            let mut contract = CodexWireContract::default();
            let mut cmp = Compare {
                basis: &d,
                live: &d,
                names: BTreeMap::new(),
                visited: BTreeSet::new(),
                strict: BTreeSet::new(),
                work: 0,
                additions: &mut contract,
            };
            for error in [
                cmp.node(&d.root, &d.root, "", "test").unwrap_err(),
                structured(&d, &d.root, &mut BTreeSet::new()).unwrap_err(),
                owner_nullable(&d, &d.root).unwrap_err(),
            ] {
                assert_eq!(error.downcast_ref::<Refusal>().unwrap().0, Reason::Limit);
            }
        }
    }
    /// Direct params references count as new structured strict reaches.
    #[test]
    fn u4r1a_direct_params_strict_reach() {
        for target in ["ThreadStatus", "CommandAction"] {
            for known in [false, true] {
                let mut files = fixture();
                let root = files.get_mut("ServerNotification").unwrap();
                let branches = root["oneOf"].as_array_mut().unwrap();
                if known {
                    let branch = branches
                        .iter_mut()
                        .find(|b| tag(b, "method").as_deref() == Some("skills/changed"))
                        .unwrap();
                    branch["properties"]["params"] =
                        json!({"$ref":format!("#/definitions/{target}")});
                } else {
                    branches.push(json!({"type":"object","properties":{"method":{"type":"string","enum":["new/direct"]},"params":{"$ref":format!("#/definitions/{target}")}},"required":["method","params"]}));
                }
                let gate = compare(&files);
                assert_eq!(
                    gate.receipt.reason,
                    Some(Reason::StrictReach),
                    "{target}/{known}: {:?}",
                    gate.receipt
                );
            }
        }
    }
    /// ThreadItem permits new variants while preserving every root keyword.
    #[test]
    fn u4r1a_thread_item_root_is_strict() {
        for (key, value) in [
            ("properties", json!({"new":{"type":"string"}})),
            ("allOf", json!([{"type":"object"}])),
            ("default", json!({})),
            ("required", json!(["new"])),
        ] {
            let mut files = fixture();
            definition(&mut files, "ServerNotification", "ThreadItem")[key] = value;
            assert_eq!(
                compare(&files).receipt.reason,
                Some(Reason::StrictMismatch),
                "{key}"
            );
        }
        let mut files = fixture();
        definition(&mut files, "ServerNotification", "ThreadItem")["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"id":{"type":"string"},"type":{"type":"string","enum":["newKind"]}},"required":["id","type"]}));
        assert!(compare(&files).receipt.pass);
    }
    /// Ref rewrites preserve sibling constraints and defaults on either side.
    #[test]
    fn u4r1a_ref_siblings_survive_rewrites() {
        for key in ["default", "const"] {
            for (ar, br) in [(false, true), (true, false), (true, true)] {
                for different in [false, true] {
                    let document = |reference, value| Document {
                        root: if reference {
                            json!({"$ref":"#/definitions/Leaf", key:value})
                        } else {
                            json!({"type":"string", key:value})
                        },
                        defs: BTreeMap::from([("Leaf".into(), json!({"type":"string", key:"a"}))]),
                    };
                    let a = document(ar, "a");
                    let b = document(br, if different { "b" } else { "a" });
                    let mut contract = CodexWireContract::default();
                    let mut cmp = Compare {
                        basis: &a,
                        live: &b,
                        names: BTreeMap::new(),
                        visited: BTreeSet::new(),
                        strict: BTreeSet::new(),
                        work: 0,
                        additions: &mut contract,
                    };
                    let result = cmp.node(&a.root, &b.root, "", "test");
                    if different {
                        assert_eq!(
                            result.unwrap_err().downcast_ref::<Refusal>().unwrap().0,
                            Reason::StrictMismatch
                        );
                    } else {
                        result.unwrap();
                    }
                }
            }
        }
    }
    /// CX-7 unchanged tag evidence passes with the approved reach-pair set and no structured reach.
    #[test]
    fn u4_basis_and_reach_pairs() {
        let files = fixture();
        let gate = compare(&files);
        assert!(gate.receipt.pass, "{:?}", gate.receipt);
        assert_eq!(gate.receipt.tolerated_changes, 0);
        assert_eq!(gate.receipt.reason, None);
        let catalog: Value = serde_json::from_str(BASIS).unwrap();
        let mut pairs = vec![];
        for (source, _) in FILES {
            let b = Document::basis(&catalog, source);
            let l = Document::live(files[*source].clone()).unwrap();
            let mut additions = CodexWireContract::default();
            let mut cmp = Compare {
                basis: &b,
                live: &l,
                names: BTreeMap::new(),
                visited: BTreeSet::new(),
                strict: BTreeSet::new(),
                work: 0,
                additions: &mut additions,
            };
            if *source == "ServerNotification" {
                for (m, s) in branches(&b.root, "method").unwrap() {
                    if codex_wire::STRICT_NOTIFICATIONS.contains(&m.as_str()) {
                        cmp.node(&s, &s, "", &m).unwrap();
                    }
                }
            } else {
                cmp.node(
                    &project(&b.root, source),
                    &project(&l.root, source),
                    source,
                    source,
                )
                .unwrap();
            }
            let a = anchors(
                &b,
                &BTreeMap::new(),
                &cmp.strict,
                *source == "ServerNotification",
            )
            .unwrap();
            for (anchor, n) in reaches(&b, &a, &BTreeMap::new(), &cmp.strict).unwrap() {
                assert!(
                    !structured(&b, &b.defs[&n], &mut BTreeSet::new()).unwrap(),
                    "structured basis reach"
                );
                pairs.push(format!("{source}:{anchor}:{n}"));
            }
        }
        pairs.sort();
        println!("REACH_PAIRS={}", serde_json::to_string(&pairs).unwrap());
        assert_eq!(pairs.len(), 20, "basis reach set changed");
        let manifest: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/codex-contract/manifest.json"
        ))
        .unwrap();
        assert_eq!(json!(pairs), manifest["reach_pairs"]);
    }
    /// CX-1 exact method branches, rather than substrings or unrelated properties, establish the contract.
    #[test]
    fn u4_method_branches_are_unique_and_complete() {
        for scenario in ["delete", "duplicate", "partial", "unrelated"] {
            let mut f = fixture();
            let s = f.get_mut("ClientRequest_used").unwrap();
            let a = s["oneOf"].as_array_mut().unwrap();
            match scenario {
                "delete" => {
                    a.remove(0);
                }
                "duplicate" => a.push(a[0].clone()),
                "partial" => a[0]["properties"]["method"]["enum"] = json!(["initializeExtra"]),
                _ => {
                    a.remove(0);
                    s["properties"] = json!({"initialize":{"type":"string"}});
                }
            }
            assert!(!compare(&f).receipt.pass, "{scenario}");
        }
    }
    /// CX-7 strict declarations and value-internal annotation keys cannot drift.
    #[test]
    fn u4_strict_schema_changes_refuse() {
        for scenario in [
            "property",
            "enum",
            "default",
            "value-internal",
            "closed",
            "required",
            "config-reserved",
        ] {
            let mut f = fixture();
            match scenario {
                "property" => {
                    definition(&mut f, "TurnStartResponse", "Turn")["properties"]["optionalNew"] =
                        json!({"type":["string","null"]});
                }
                "enum" => {
                    definition(&mut f, "TurnStartResponse", "TurnStatus")["enum"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("new"));
                }
                "default" => {
                    definition(&mut f, "ClientRequest_used", "InitializeCapabilities")["properties"]
                        ["experimentalApi"]["default"] = json!(true);
                }
                "value-internal" => {
                    definition(&mut f, "ClientRequest_used", "InitializeCapabilities")["properties"]
                        ["experimentalApi"]["default"] =
                        json!({"description":"changed","$ref":"not a schema ref"});
                }
                "closed" => {
                    definition(&mut f, "ServerRequest", "RequestPermissionProfile")["additionalProperties"] =
                        json!(true);
                }
                "config-reserved" => {
                    definition(&mut f, "ConfigReadResponse", "Config")["properties"]["features"] =
                        json!({"type":"object"});
                }
                _ => {
                    definition(&mut f, "TurnStartResponse", "Turn")["required"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("rootTurnId"));
                }
            }
            let gate = compare(&f);
            assert!(!gate.receipt.pass, "{scenario}");
            assert_eq!(
                gate.receipt.reason,
                Some(Reason::StrictMismatch),
                "{scenario}: {:?}",
                gate.receipt
            );
        }
    }
    /// CX-7 only a new structured strict reach is refused; scalar-leaf additions are permitted.
    #[test]
    fn u4_tolerated_growth_and_new_reach() {
        for method in ["initialized", "turn/start", "item/tool/call"] {
            let mut f = fixture();
            f.get_mut("ServerNotification").unwrap()["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"method":{"type":"string","enum":[method]},"params":{"type":"object"}},"required":["method","params"]}));
            assert_eq!(
                compare(&f).receipt.reason,
                Some(Reason::AdditionName),
                "RPC method became a tolerated notification"
            );
        }
        for scenario in [
            "notification-turn",
            "thread-turn",
            "user-command",
            "new-item",
            "new-notification",
            "scalar-notification",
            "tolerated-change",
        ] {
            let mut f = fixture();
            match scenario {
                "notification-turn" => {
                    definition(&mut f, "ServerNotification", "HookStartedNotification")["properties"]
                        ["new"] = json!({"$ref":"#/definitions/Turn"});
                }
                "thread-turn" => {
                    definition(&mut f, "ServerNotification", "Thread")["properties"]["new"] =
                        json!({"$ref":"#/definitions/Turn"});
                }
                "user-command" => {
                    definition(&mut f, "ServerNotification", "ThreadItem")["oneOf"][0]["properties"]
                        ["new"] = json!({"$ref":"#/definitions/CommandAction"});
                }
                "new-item" => {
                    definition(&mut f,"ServerNotification","ThreadItem")["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"id":{"type":"string"},"type":{"type":"string","enum":["newItem"]},"new":{"type":"boolean"}},"required":["id","type","new"]}));
                }
                "tolerated-change" => {
                    definition(&mut f, "ServerNotification", "HookStartedNotification")["properties"]
                        ["new"] = json!({"type":["string","null"]});
                }
                _ => {
                    let target = if scenario == "new-notification" {
                        "ThreadItem"
                    } else {
                        "AbsolutePathBuf"
                    };
                    f.get_mut("ServerNotification").unwrap()["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"method":{"type":"string","enum":["new/notification"]},"params":{"type":"object","properties":{"new":{"$ref":format!("#/definitions/{target}")}}}},"required":["method","params"]}));
                }
            }
            let gate = compare(&f);
            if matches!(
                scenario,
                "new-item" | "scalar-notification" | "tolerated-change"
            ) {
                assert!(gate.receipt.pass, "{scenario}: {:?}", gate.receipt);
            } else {
                assert_eq!(
                    gate.receipt.reason,
                    Some(Reason::StrictReach),
                    "{scenario}: {:?}",
                    gate.receipt
                );
            }
        }
    }
    /// CX-7 each tolerated-addition ceiling admits its limit and refuses one more without a contract.
    #[test]
    fn u4r1b_addition_limits_refuse_one_over_each_ceiling() {
        for (items, notifications) in [(32, 0), (33, 0), (0, 64), (0, 65)] {
            let mut f = fixture();
            let notification = f.get_mut("ServerNotification").unwrap();
            for index in 0..items {
                notification["definitions"]["ThreadItem"]["oneOf"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"type":"object","properties":{"id":{"type":"string"},"type":{"type":"string","enum":[format!("extraItem{index}")]}},"required":["id","type"]}));
            }
            for index in 0..notifications {
                notification["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"method":{"type":"string","enum":[format!("extra/notification{index}")]},"params":{"type":"object"}},"required":["method","params"]}));
            }
            let gate = compare(&f);
            if items <= 32 && notifications <= 64 {
                assert!(
                    gate.receipt.pass,
                    "{items}/{notifications}: {:?}",
                    gate.receipt
                );
                assert_eq!(
                    (
                        gate.receipt.accepted_item_types,
                        gate.receipt.accepted_notifications
                    ),
                    (items, notifications)
                );
                assert!(gate.contract.is_some());
            } else {
                assert!(!gate.receipt.pass, "{items}/{notifications}");
                assert_eq!(
                    gate.receipt.reason,
                    Some(Reason::AdditionLimit),
                    "{items}/{notifications}: {:?}",
                    gate.receipt
                );
                assert!(gate.contract.is_none(), "{items}/{notifications}");
            }
        }
    }
    /// CX-7 added item kinds and notification methods admit their name-length limit and refuse one more byte.
    #[test]
    fn u4r1b_addition_names_refuse_one_over_each_length() {
        for (kind, method) in [
            ("k".repeat(64), None),
            ("k".repeat(65), None),
            (String::new(), Some(format!("m/{}", "n".repeat(126)))),
            (String::new(), Some(format!("m/{}", "n".repeat(127)))),
        ] {
            let mut f = fixture();
            let notification = f.get_mut("ServerNotification").unwrap();
            let length = if let Some(method) = &method {
                notification["oneOf"].as_array_mut().unwrap().push(json!({"type":"object","properties":{"method":{"type":"string","enum":[method]},"params":{"type":"object"}},"required":["method","params"]}));
                (method.len(), 128)
            } else {
                notification["definitions"]["ThreadItem"]["oneOf"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"type":"object","properties":{"id":{"type":"string"},"type":{"type":"string","enum":[kind]}},"required":["id","type"]}));
                (kind.len(), 64)
            };
            let gate = compare(&f);
            if length.0 <= length.1 {
                assert!(gate.receipt.pass, "{length:?}: {:?}", gate.receipt);
            } else {
                assert_eq!(
                    gate.receipt.reason,
                    Some(Reason::AdditionName),
                    "{length:?}: {:?}",
                    gate.receipt
                );
                assert!(gate.contract.is_none(), "{length:?}");
            }
        }
    }
    /// CX-2 reference depth, distinct memo targets, cycle detection and work are separately bounded.
    #[test]
    fn u4_ref_bounds_and_value_positions() {
        for count in [16, 17] {
            let defs = (0..count)
                .map(|i| {
                    (
                        format!("D{i}"),
                        if i + 1 == count {
                            json!({"type":"string"})
                        } else {
                            json!({"$ref":format!("#/definitions/D{}",i+1)})
                        },
                    )
                })
                .collect();
            let d = Document {
                root: json!({"$ref":"#/definitions/D0"}),
                defs,
            };
            assert_eq!(d.bounds().is_ok(), count == 16);
        }
        for count in [512, 513] {
            let defs = (0..count)
                .map(|i| (format!("D{i}"), json!({"type":"string"})))
                .collect();
            let refs = (0..count)
                .map(|i| json!({"$ref":format!("#/definitions/D{i}")}))
                .collect::<Vec<_>>();
            assert_eq!(
                Document {
                    root: json!({"allOf":refs}),
                    defs
                }
                .bounds()
                .is_ok(),
                count == 512
            );
        }
        for r in [
            "#/definitions/Missing",
            "https://example.invalid/schema",
            "#/definitions/Cycle",
        ] {
            let d = Document {
                root: json!({"$ref":r}),
                defs: BTreeMap::from([("Cycle".into(), json!({"$ref":"#/definitions/Cycle"}))]),
            };
            assert!(d.bounds().is_err());
            let mut files = fixture();
            definition(&mut files, "InitializeResponse", "AbsolutePathBuf")["$ref"] = json!(r);
            assert!(
                !compare(&files).receipt.pass,
                "live schema reference bypass"
            );
        }
        let d = Document {
            root: json!({"default":{"$ref":"external","description":"value"}}),
            defs: BTreeMap::new(),
        };
        assert!(d.bounds().is_ok());
        for count in [262143, 262144] {
            let d = Document {
                root: json!({"allOf":vec![json!(true);count]}),
                defs: BTreeMap::new(),
            };
            assert_eq!(d.bounds().is_ok(), count == 262143);
        }
    }
    /// CX-2 schema JSON has its own exact limits without widening the wire profile.
    #[test]
    fn u4_schema_json_limits() {
        use crate::execution::strict_json::{Limits, decode, decode_schema};
        for depth in [64, 65] {
            let s = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
            assert_eq!(decode_schema(s.as_bytes()).is_ok(), depth == 64);
            assert!(decode(s.as_bytes(), Limits::CEILINGS).is_err());
        }
        for nodes in [262144, 262145] {
            let mut rows = vec![json!(vec![0; 1023]); 256];
            if nodes == 262144 {
                rows[0].as_array_mut().unwrap().pop();
            }
            assert_eq!(
                decode_schema(&serde_json::to_vec(&rows).unwrap()).is_ok(),
                nodes == 262144
            );
        }
        for n in [65536, 65537] {
            assert_eq!(
                decode_schema(&serde_json::to_vec(&"s".repeat(n)).unwrap()).is_ok(),
                n == 65536
            );
        }
    }
    /// Schema files obey independent aggregate, fixed-file, entry and depth budgets without following symlinks.
    #[test]
    fn u4_schema_directory_limits() {
        for scenario in [
            "baseline",
            "file",
            "file-plus-one",
            "aggregate",
            "aggregate-plus-one",
            "entries",
            "entries-plus-one",
            "depth",
            "depth-plus-one",
            "symlink",
        ] {
            let temp = tempfile::tempdir().unwrap();
            for (_, p) in FILES {
                let p = temp.path().join(p);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, b"{}").unwrap();
            }
            match scenario {
                "file" | "file-plus-one" => {
                    let mut data = b"{}".to_vec();
                    data.resize(1024 * 1024 + usize::from(scenario == "file-plus-one"), b' ');
                    std::fs::write(temp.path().join(FILES[0].1), data).unwrap();
                }
                "aggregate" | "aggregate-plus-one" => {
                    let f = std::fs::File::create(temp.path().join("extra")).unwrap();
                    f.set_len(
                        16 * 1024 * 1024 - (FILES.len() * 2) as u64
                            + u64::from(scenario == "aggregate-plus-one"),
                    )
                    .unwrap();
                }
                "entries" | "entries-plus-one" => {
                    for i in
                        0..(1024 - FILES.len() - 2 + usize::from(scenario == "entries-plus-one"))
                    {
                        std::fs::write(temp.path().join(format!("extra-{i}")), b"").unwrap();
                    }
                }
                "depth" | "depth-plus-one" => {
                    let mut p = temp.path().to_path_buf();
                    for _ in 0..(4 + usize::from(scenario == "depth-plus-one")) {
                        p = p.join("d");
                        std::fs::create_dir(&p).unwrap();
                    }
                }
                "symlink" => std::os::unix::fs::symlink("/", temp.path().join("external")).unwrap(),
                _ => {}
            }
            let success = read_directory(&std::fs::File::open(temp.path()).unwrap()).is_ok();
            assert_eq!(
                success,
                !scenario.ends_with("plus-one") && scenario != "symlink",
                "{scenario}"
            );
        }
    }
}
