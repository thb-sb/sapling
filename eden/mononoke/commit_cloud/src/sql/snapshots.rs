/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use ::sql_ext::mononoke_queries;
use async_trait::async_trait;
use edenapi_types::HgId;
use mercurial_types::HgChangesetId;

use crate::sql::ops::Delete;
use crate::sql::ops::Get;
use crate::sql::ops::Insert;
use crate::sql::ops::SqlCommitCloud;
use crate::sql::ops::Update;
use crate::sql::utils::changeset_as_bytes;
use crate::sql::utils::changeset_from_bytes;
use crate::sql::utils::list_as_bytes;
use crate::CommitCloudContext;

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceSnapshot {
    pub commit: HgChangesetId,
}

pub struct DeleteArgs {
    pub removed_commits: Vec<HgChangesetId>,
}

mononoke_queries! {
    read GetSnapshots(reponame: String, workspace: String) -> (String, Vec<u8>){
        mysql("SELECT `reponame`, `node` FROM snapshots WHERE `reponame`={reponame} AND `workspace`={workspace} ORDER BY `seq`")
        sqlite("SELECT `reponame`, `commit` FROM snapshots WHERE `reponame`={reponame} AND `workspace`={workspace} ORDER BY `seq`")
    }

    write DeleteSnapshot(reponame: String, workspace: String, >list commits: Vec<u8>) {
        none,
        mysql("DELETE FROM `snapshots` WHERE `reponame`={reponame} AND `workspace`={workspace} AND `node` IN {commits}")
        sqlite("DELETE FROM `snapshots` WHERE `reponame`={reponame} AND `workspace`={workspace} AND `commit` IN {commits}")
    }

    write InsertSnapshot(reponame: String, workspace: String, commit: Vec<u8>) {
        none,
        mysql("INSERT INTO `snapshots` (`reponame`, `workspace`, `node`) VALUES ({reponame}, {workspace}, {commit})")
        sqlite("INSERT INTO `snapshots` (`reponame`, `workspace`, `commit`) VALUES ({reponame}, {workspace}, {commit})")
    }
}

#[async_trait]
impl Get<WorkspaceSnapshot> for SqlCommitCloud {
    async fn get(
        &self,
        reponame: String,
        workspace: String,
    ) -> anyhow::Result<Vec<WorkspaceSnapshot>> {
        let rows =
            GetSnapshots::query(&self.connections.read_connection, &reponame, &workspace).await?;
        rows.into_iter()
            .map(|(_reponame, commit)| {
                Ok(WorkspaceSnapshot {
                    commit: changeset_from_bytes(&commit, self.uses_mysql)?,
                })
            })
            .collect::<anyhow::Result<Vec<WorkspaceSnapshot>>>()
    }
}
#[async_trait]
impl Insert<WorkspaceSnapshot> for SqlCommitCloud {
    async fn insert(
        &self,
        reponame: String,
        workspace: String,
        data: WorkspaceSnapshot,
    ) -> anyhow::Result<()> {
        InsertSnapshot::query(
            &self.connections.write_connection,
            &reponame,
            &workspace,
            &changeset_as_bytes(&data.commit, self.uses_mysql)?,
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl Update<WorkspaceSnapshot> for SqlCommitCloud {
    type UpdateArgs = ();
    async fn update(
        &self,
        _reponame: String,
        _workspace: String,
        _extra_arg: Self::UpdateArgs,
    ) -> anyhow::Result<()> {
        //To be implemented among other Update queries
        return Err(anyhow::anyhow!("Not implemented yet"));
    }
}

#[async_trait]
impl Delete<WorkspaceSnapshot> for SqlCommitCloud {
    type DeleteArgs = DeleteArgs;
    async fn delete(
        &self,
        reponame: String,
        workspace: String,
        args: Self::DeleteArgs,
    ) -> anyhow::Result<()> {
        DeleteSnapshot::query(
            &self.connections.write_connection,
            &reponame,
            &workspace,
            &list_as_bytes(args.removed_commits, self.uses_mysql)?,
        )
        .await?;
        Ok(())
    }
}

pub async fn update_snapshots(
    sql_commit_cloud: &SqlCommitCloud,
    ctx: CommitCloudContext,
    new_snapshots: Vec<HgId>,
    removed_snapshots: Vec<HgId>,
) -> anyhow::Result<()> {
    if !removed_snapshots.is_empty() {
        let delete_args = DeleteArgs {
            removed_commits: removed_snapshots
                .into_iter()
                .map(|id| id.into())
                .collect::<Vec<HgChangesetId>>(),
        };

        Delete::<WorkspaceSnapshot>::delete(
            sql_commit_cloud,
            ctx.reponame.clone(),
            ctx.workspace.clone(),
            delete_args,
        )
        .await?;
    }

    for snapshot in new_snapshots {
        Insert::<WorkspaceSnapshot>::insert(
            sql_commit_cloud,
            ctx.reponame.clone(),
            ctx.workspace.clone(),
            WorkspaceSnapshot {
                commit: snapshot.into(),
            },
        )
        .await?;
    }

    Ok(())
}
