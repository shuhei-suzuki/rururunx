//! Independent retained Git objects and disposable read-only review inputs.
use super::{RuntimeOwner, *};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::{File, OpenOptions}, io::Write, os::unix::fs::{OpenOptionsExt, PermissionsExt}, path::{Path,PathBuf}, sync::Arc};
use tokio::{process::Command, sync::Mutex};
use super::git_io::UnitGit;

pub struct ResultStore {owner:Arc<RuntimeOwner>,gate:Mutex<()>}
#[cfg(test)]mod tests;
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultManifest {
    pub artifact:ArtifactId,pub unit:UnitId,pub revision:String,pub base:String,
    pub object_format:String,pub sources:BTreeMap<String,String>,
}
/// This provenance cannot be manufactured by passing an arbitrary Git directory.
#[derive(Debug,Clone)]
pub struct ResultSnapshot {
    pub artifact:ArtifactId,pub unit:UnitId,pub revision:String,
    source:PathBuf,output:PathBuf,manifest_sha256:String,
}
impl ResultSnapshot {
    pub fn source(&self)->&Path {&self.source}
    pub fn output(&self)->&Path {&self.output}
    pub fn manifest_sha256(&self)->&str {&self.manifest_sha256}
    pub async fn verify(&self)->Result<()> {
        ensure!(text(&git(&self.source,["rev-parse","HEAD"]).await?)?==self.revision,"snapshot HEAD changed");
        ensure!(tracked_digest(&self.source).await?==self.manifest_sha256,"snapshot source changed");
        ensure!(git(&self.source,["status","--porcelain","--untracked-files=all"]).await?.is_empty(),"snapshot became dirty");
        readonly_tree(&self.source,false)?;
        Ok(())
    }
}
impl ResultStore {
    pub fn new(owner:Arc<RuntimeOwner>)->Self {Self {owner,gate:Mutex::new(())}}
    pub async fn capture(&self,authority:&ExecutionAuthority,revision:&str,sources:BTreeMap<String,String>)->Result<ResultArtifact> {
        ensure!(valid_oid(revision) && sources.len()<=128 && sources.iter().all(|(k,v)|!k.is_empty() && k.len()<=128 && v.len()<=256),"invalid result identity/sources");
        let unit=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.validate_execution(authority,false,true)?;
        ensure!(unit.kind==UnitKind::Executor && unit.work==Some(WorkOutcome::Success),"capture requires known successful executor");
        let project=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.project(unit.scope.project_id)?.context("Project missing")?;
        let source=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.task(unit.scope.task_id.context("Task missing")?)?.context("Task missing")?;
        let io=UnitGit::new(self.owner.clone(),&unit,false)?;
        io.ownership(&project,&source).await?;
        let format=io.text(&unit.worktree,["rev-parse","--show-object-format"]).await?;
        ensure!(matches!(format.as_str(),"sha1"|"sha256"),"unsupported Git object format");
        ensure!(revision.len()==if format=="sha1" {40}else{64},"OID format mismatch");
        io.run(&unit.worktree,["merge-base","--is-ancestor",unit.base_sha.as_str(),revision]).await?;
        ensure!(io.text(&unit.worktree,["rev-parse","--verify",&format!("{revision}^{{commit}}")]).await?==revision,"result is not exact commit");
        qualified_content(&unit.worktree,revision).await?;
        let _guard=self.gate.lock().await;
        let repository=self.owner.root.join("projects").join(project.id.to_string()).join("results.git");
        let id=ArtifactId::new();
        let directory=self.owner.root.join("artifacts").join(id.to_string());
        let manifest=directory.join("manifest.json");
        let mut artifact=ResultArtifact {id,scope:unit.scope.clone(),unit_id:unit.id,state:ArtifactState::Staging,
            revision:revision.into(),base_sha:unit.base_sha.clone(),object_format:format.clone(),repository:repository.clone(),manifest,
            manifest_sha256:String::new(),dependencies:sources.clone(),version:1,created_at:crate::domain::now_ms()};
        self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.stage_result(authority,&artifact)?;
        std::fs::create_dir_all(repository.parent().context("result parent missing")?)?;
        ensure!(!repository.starts_with(&unit.worktree) && !unit.worktree.starts_with(&repository),"results overlap executor");
        if !repository.exists() {
            io.run(repository.parent().context("repository parent missing")?,["init","--bare",&format!("--object-format={format}"),repository.to_str().context("repository UTF-8")?]).await?;
        }
        ensure!(!repository.join("objects/info/alternates").exists(),"result alternates forbidden");
        ensure!(io.text(&repository,["rev-parse","--show-object-format"]).await?==format,"repository format changed");
        // fetch copies object graphs over upload-pack. No clone-local hardlinks or alternates.
        io.run(&repository,["-c","fetch.fsckObjects=true","fetch","--no-tags","--no-write-fetch-head","--no-recurse-submodules","--",
            unit.worktree.to_str().context("worktree UTF-8")?,&format!("{revision}:refs/rrx/{id}/commit"),&format!("{}:refs/rrx/{id}/base",unit.base_sha)]).await?;
        io.run(&repository,["fsck","--full","--strict","--no-dangling"]).await?;
        for oid in [revision,unit.base_sha.as_str()] {io.run(&repository,["rev-list","--objects","--missing=error",oid]).await?;}
        std::fs::create_dir_all(directory.parent().context("artifact root missing")?)?;
        std::fs::create_dir(&directory)?;
        let bytes=serde_json::to_vec(&ResultManifest{artifact:id,unit:unit.id,revision:revision.into(),base:unit.base_sha.clone(),object_format:format,sources})?;
        durable_file(&artifact.manifest,&bytes)?;
        sync_tree(&repository)?;File::open(repository.parent().context("repository parent missing")?)?.sync_all()?;
        File::open(directory.parent().context("artifact parent missing")?)?.sync_all()?;
        artifact.manifest_sha256=hex(&bytes);artifact.state=ArtifactState::Ready;artifact.version=2;
        self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.ready_result(&artifact,1)?;
        Ok(artifact)
    }
    pub async fn verify(&self,artifact:&ResultArtifact)->Result<()> {
        ensure!(matches!(artifact.state,ArtifactState::Ready|ArtifactState::Published),"artifact is not usable");
        ensure!(artifact.repository==self.owner.root.join("projects").join(artifact.scope.project_id.to_string()).join("results.git")
            && artifact.manifest==self.owner.root.join("artifacts").join(artifact.id.to_string()).join("manifest.json"),"artifact storage mismatch");
        let bytes=std::fs::read(&artifact.manifest)?;ensure!(bytes.len()<=128*1024 && hex(&bytes)==artifact.manifest_sha256,"artifact manifest corruption");
        let m:ResultManifest=serde_json::from_slice(&bytes)?;
        ensure!(m.artifact==artifact.id && m.unit==artifact.unit_id && m.revision==artifact.revision && m.base==artifact.base_sha
            && m.object_format==artifact.object_format && m.sources==artifact.dependencies,"artifact manifest identity mismatch");
        ensure!(!artifact.repository.join("objects/info/alternates").exists(),"result alternates forbidden");
        for (name,oid) in [("commit",&artifact.revision),("base",&artifact.base_sha)] {
            ensure!(text(&git(&artifact.repository,["rev-parse","--verify",&format!("refs/rrx/{}/{name}",artifact.id)]).await?)?==*oid,"retained reference changed");
            git(&artifact.repository,["rev-list","--objects","--missing=error",oid]).await?;
        }
        git(&artifact.repository,["fsck","--full","--strict","--no-dangling"]).await?;Ok(())
    }
    pub async fn publish(&self,authority:&ExecutionAuthority,artifact:&ResultArtifact,task_version:u64)->Result<ResultArtifact> {
        self.verify(artifact).await?;
        self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.publish_execution_result(authority,artifact.id,task_version)
    }
    pub async fn snapshot(&self,unit:&ExecutionUnit)->Result<ResultSnapshot> {
        ensure!(matches!(unit.kind,UnitKind::Reviewer|UnitKind::Verifier),"snapshot requires reviewer/verifier unit");
        self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.validate_execution(&unit.authority(),true,false)?;
        let artifact=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.result_artifact(unit.artifact_id.context("snapshot artifact missing")?)?;
        ensure!(artifact.scope==unit.scope && artifact.revision==unit.base_sha && artifact.state==ArtifactState::Published,"snapshot artifact binding mismatch");
        self.verify(&artifact).await?;
        let io=UnitGit::new(self.owner.clone(),unit,true)?;
        ensure!(!unit.worktree.exists() && !unit.worktree.symlink_metadata().is_ok(),"snapshot paths are never reused");
        let output=self.owner.root.join("units").join(unit.id.to_string()).join("output");
        std::fs::create_dir_all(&output)?;
        io.run(&output,["clone","--no-local","--no-hardlinks","--no-checkout","--",artifact.repository.to_str().context("repository UTF-8")?,unit.worktree.to_str().context("source UTF-8")?]).await.context("clone independent snapshot")?;
        ensure!(!unit.worktree.join(".git/objects/info/alternates").exists(),"snapshot alternates forbidden");
        // The result repo retains private refs; clone's default head refspec does not select them.
        io.run(&unit.worktree,["fetch","--no-tags","--no-write-fetch-head","--no-recurse-submodules","--",
            artifact.repository.to_str().context("repository UTF-8")?,&format!("{}:refs/rrx/input",artifact.revision),&format!("{}:refs/rrx/base",artifact.base_sha)]).await.context("import exact retained snapshot graph")?;
        io.run(&unit.worktree,["checkout","--detach",artifact.revision.as_str()]).await.context("checkout retained snapshot SHA")?;
        qualified_content(&unit.worktree,&artifact.revision).await?;
        let digest=tracked_digest(&unit.worktree).await?;
        readonly_tree(&unit.worktree,true)?;
        let snapshot=ResultSnapshot {artifact:artifact.id,unit:unit.id,revision:artifact.revision,source:unit.worktree.clone(),output,manifest_sha256:digest};
        snapshot.verify().await?;
        Ok(snapshot)
    }
}

pub(crate) fn git_command(root:&Path)->Result<Command> {
    let mut c=Command::new(super::resources::resolve_program("git")?);c.arg("-C").arg(root).args(["-c","gc.auto=0","-c","maintenance.auto=false","-c","core.fsmonitor=false"]);
    c.env("GIT_OPTIONAL_LOCKS","0").env_remove("GIT_DIR").env_remove("GIT_WORK_TREE").env_remove("GIT_INDEX_FILE").env_remove("GIT_OBJECT_DIRECTORY").env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES");Ok(c)
}
pub(crate) async fn git<I,S>(root:&Path,args:I)->Result<Vec<u8>> where I:IntoIterator<Item=S>,S:AsRef<std::ffi::OsStr> {
    process::capture(git_command(root)?.args(args)).await
}
pub(crate) fn text(bytes:&[u8])->Result<String>{Ok(std::str::from_utf8(bytes)?.trim().to_owned())}
pub(crate) fn hex(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
pub(crate) fn durable_file(path:&Path,bytes:&[u8])->Result<()> {
    let temp=path.with_extension(format!("tmp-{}",uuid::Uuid::new_v4()));
    let mut file=OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temp)?;
    file.write_all(bytes)?;file.sync_all()?;
    std::fs::rename(&temp,path)?;File::open(path.parent().context("file parent missing")?)?.sync_all()?;Ok(())
}
fn sync_tree(path:&Path)->Result<()> {
    let metadata=std::fs::symlink_metadata(path)?;ensure!(!metadata.file_type().is_symlink(),"symlink in result storage");
    if metadata.is_dir(){for e in std::fs::read_dir(path)?{sync_tree(&e?.path())?;}}
    File::open(path)?.sync_all()?;Ok(())
}
fn readonly_tree(path:&Path,apply:bool)->Result<()> {
    let m=std::fs::symlink_metadata(path)?;
    ensure!(!m.file_type().is_symlink(),"source symlinks require a qualified profile");
    ensure!(m.is_dir()||m.is_file(),"unsupported snapshot file type");
    if m.is_dir(){for entry in std::fs::read_dir(path)? {readonly_tree(&entry?.path(),apply)?;}}
    if apply {std::fs::set_permissions(path,std::fs::Permissions::from_mode(m.permissions().mode() & !0o222))?;}
    else {ensure!(m.permissions().mode() & 0o222==0,"snapshot regained write permissions");}
    Ok(())
}
pub(crate) async fn qualified_content(root:&Path,revision:&str)->Result<()> {
    let tree=git(root,["ls-tree","-r","-z",revision]).await?;
    for line in tree.split(|b|*b==0).filter(|s|!s.is_empty()) {
        ensure!(!line.starts_with(b"160000 "),"submodules require a qualified profile");
        ensure!(!line.starts_with(b"120000 "),"source symlinks require a qualified profile");
    }
    // LFS pointers are ordinary Git blobs, not their required external content.
    let mut command=git_command(root)?;command.args(["grep","-l","-I","-e","version https://git-lfs.github.com/spec/v1",revision,"--"]);
    let observed=process::capture_observed(&mut command).await?;
    ensure!(observed.receipt.status.code()==Some(1) || (observed.receipt.status.success() && observed.stdout.is_empty()),"LFS pointer scan found unsupported content or failed");
    let attrs=git(root,["ls-tree","-r","--name-only",revision]).await?;
    for name in std::str::from_utf8(&attrs)?.lines().filter(|n|n.ends_with(".gitattributes")) {
        let bytes=git(root,["show",&format!("{revision}:{name}")]).await?;
        ensure!(!bytes.windows(b"filter=lfs".len()).any(|w|w==b"filter=lfs"),"LFS attributes require a qualified profile");
    }
    Ok(())
}
async fn tracked_digest(root:&Path)->Result<String> {
    let list=git(root,["ls-files","-z"]).await?;
    let mut hash=Sha256::new();
    for name in list.split(|b|*b==0).filter(|s|!s.is_empty()) {
        let name=std::str::from_utf8(name)?;
        ensure!(!Path::new(name).is_absolute() && !Path::new(name).components().any(|c|matches!(c,std::path::Component::ParentDir)),"invalid tracked path");
        let path=root.join(name);let m=std::fs::symlink_metadata(&path)?;
        ensure!(m.is_file() && !m.file_type().is_symlink(),"snapshot source must be a regular file");
        hash.update(name.as_bytes());hash.update([0]);hash.update((m.permissions().mode() & 0o111).to_le_bytes());
        let mut file=File::open(path)?;std::io::copy(&mut file,&mut HashWriter(&mut hash))?;hash.update([0]);
    }
    Ok(format!("{:x}",hash.finalize()))
}
struct HashWriter<'a>(&'a mut Sha256);
impl Write for HashWriter<'_> {fn write(&mut self,buf:&[u8])->std::io::Result<usize>{self.0.update(buf);Ok(buf.len())}fn flush(&mut self)->std::io::Result<()>{Ok(())}}
