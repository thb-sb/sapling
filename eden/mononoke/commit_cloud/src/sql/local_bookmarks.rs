/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::collections::HashMap;

use ::sql_ext::mononoke_queries;
use async_trait::async_trait;
use edenapi_types::HgId;
use mercurial_types::HgChangesetId;
use serde::Deserialize;
use serde::Serialize;
use sql::Connection;

use crate::sql::ops::Delete;
use crate::sql::ops::Get;
use crate::sql::ops::Insert;
use crate::sql::ops::SqlCommitCloud;
use crate::sql::ops::Update;
use crate::sql::utils::changeset_as_bytes;
use crate::sql::utils::changeset_from_bytes;
use crate::sql::utils::list_as_bytes;
use crate::CommitCloudContext;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceLocalBookmark {
    pub name: String,
    pub commit: HgChangesetId,
}

pub struct DeleteArgs {
    pub removed_bookmarks: Vec<HgChangesetId>,
}

mononoke_queries! {
    read GetLocalBookmarks(reponame: String, workspace: String) -> (String,  Vec<u8>){
        mysql("SELECT `name`, `node` FROM `bookmarks` WHERE `reponame`={reponame} AND `workspace`={workspace}")
        sqlite("SELECT `name`, `commit` FROM `workspacebookmarks` WHERE `reponame`={reponame} AND `workspace`={workspace}")
    }
    write DeleteLocalBookmark(reponame: String, workspace: String, >list removed_bookmarks: Vec<u8>) {
        none,
        mysql("DELETE FROM `bookmarks` WHERE `reponame`={reponame} AND `workspace`={workspace} AND `node` IN {removed_bookmarks}")
        sqlite("DELETE FROM `workspacebookmarks` WHERE `reponame`={reponame} AND `workspace`={workspace} AND `commit` IN {removed_bookmarks}")
    }
    write InsertLocalBookmark(reponame: String, workspace: String, name: String, commit: Vec<u8>) {
        none,
        mysql("INSERT INTO `bookmarks` (`reponame`, `workspace`, `name`, `node`) VALUES ({reponame}, {workspace}, {name}, {commit})")
        sqlite("INSERT INTO `workspacebookmarks` (`reponame`, `workspace`, `name`, `commit`) VALUES ({reponame}, {workspace}, {name}, {commit})")
    }
}

#[async_trait]
impl Get<WorkspaceLocalBookmark> for SqlCommitCloud {
    async fn get(
        &self,
        reponame: String,
        workspace: String,
    ) -> anyhow::Result<Vec<WorkspaceLocalBookmark>> {
        let rows = GetLocalBookmarks::query(
            &self.connections.read_connection,
            &reponame.clone(),
            &workspace,
        )
        .await?;
        rows.into_iter()
            .map(|(name, commit)| {
                Ok(WorkspaceLocalBookmark {
                    name,
                    commit: changeset_from_bytes(&commit, self.uses_mysql)?,
                })
            })
            .collect::<anyhow::Result<Vec<WorkspaceLocalBookmark>>>()
    }
}

#[async_trait]
impl Insert<WorkspaceLocalBookmark> for SqlCommitCloud {
    async fn insert(
        &self,
        reponame: String,
        workspace: String,
        data: WorkspaceLocalBookmark,
    ) -> anyhow::Result<()> {
        InsertLocalBookmark::query(
            &self.connections.write_connection,
            &reponame,
            &workspace,
            &data.name,
            &changeset_as_bytes(&data.commit, self.uses_mysql)?,
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl Update<WorkspaceLocalBookmark> for SqlCommitCloud {
    type UpdateArgs = ();

    async fn update(
        &self,
        _reponame: String,
        _workspace: String,
        _args: Self::UpdateArgs,
    ) -> anyhow::Result<()> {
        //To be implemented among other Update queries
        return Err(anyhow::anyhow!("Not implemented yet"));
    }
}

#[async_trait]
impl Delete<WorkspaceLocalBookmark> for SqlCommitCloud {
    type DeleteArgs = DeleteArgs;
    async fn delete(
        &self,
        reponame: String,
        workspace: String,
        args: Self::DeleteArgs,
    ) -> anyhow::Result<()> {
        DeleteLocalBookmark::query(
            &self.connections.write_connection,
            &reponame,
            &workspace,
            &list_as_bytes(args.removed_bookmarks, self.uses_mysql)?,
        )
        .await?;
        Ok(())
    }
}

pub async fn update_bookmarks(
    sql_commit_cloud: &SqlCommitCloud,
    ctx: CommitCloudContext,
    updated_bookmarks: HashMap<String, HgId>,
    removed_bookmarks: Vec<HgId>,
) -> anyhow::Result<()> {
    if !removed_bookmarks.is_empty() {
        let removed_commits = removed_bookmarks
            .into_iter()
            .map(|id| id.into())
            .collect::<Vec<HgChangesetId>>();
        let delete_args = DeleteArgs {
            removed_bookmarks: removed_commits,
        };

        Delete::<WorkspaceLocalBookmark>::delete(
            sql_commit_cloud,
            ctx.reponame.clone(),
            ctx.workspace.clone(),
            delete_args,
        )
        .await?;
    }

    for (name, book) in updated_bookmarks {
        Insert::<WorkspaceLocalBookmark>::insert(
            sql_commit_cloud,
            ctx.reponame.clone(),
            ctx.workspace.clone(),
            WorkspaceLocalBookmark {
                name,
                commit: book.into(),
            },
        )
        .await?;
    }

    Ok(())
}
