use crate::{
    marshall_plugin::{deserialize, serialize},
    workers::protocol::{FileSystemWorkerMessage, RepositoryCrawlerResponse},
};

use super::fs::list_repositories;
use super::protocol::RepositoryCrawlerRequest;

use anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zellij_tile::prelude::*;

type Result = anyhow::Result<()>;

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct FileSystemWorker {}

impl FileSystemWorker {
    fn parse_request(&mut self, message: String, payload: String) -> Result {
        let message = deserialize::<FileSystemWorkerMessage>(&message)
            .with_context(|| "deserializing inbound message from plugin")?;
        assert!(
            matches!(message, FileSystemWorkerMessage::Crawl),
            "unsupported worker message"
        );

        let request = deserialize::<RepositoryCrawlerRequest>(&payload)
            .with_context(|| "deserializing inbound payload from plugin")?;

        self.crawl(&request.root, request.max_depth)
    }

    fn crawl(&mut self, root: &PathBuf, max_depth: usize) -> Result {
        let repositories = list_repositories(root, max_depth);

        post_message_to_plugin(PluginMessage::new_to_plugin(
            &serialize(&FileSystemWorkerMessage::Crawl)?,
            &serialize(&RepositoryCrawlerResponse { repositories })?,
        ));

        Ok(())
    }
}

impl<'de> ZellijWorker<'de> for FileSystemWorker {
    fn on_message(&mut self, message: String, payload: String) {
        if let Err(error) = self.parse_request(message, payload) {
            // NOTE: if we failed to serialize our response, chances are we're not going to be able
            // to send the error back to the plugin. Fallback to logging the error.
            eprintln!("failed to scan host: {error:?}");
        }
    }
}
