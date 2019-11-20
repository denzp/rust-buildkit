use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use failure::{bail, format_err, Error, ResultExt};
use futures::compat::*;
use futures::lock::Mutex;
use log::*;

use tower_grpc::{BoxBody, Request};
use tower_hyper::client::Connection;

use buildkit_proto::google::rpc::Status;
use buildkit_proto::moby::buildkit::v1::frontend::{
    client, result::Result as RefResult, ReadFileRequest, ResolveImageConfigRequest,
    Result as Output, ReturnRequest, SolveRequest,
};

pub use buildkit_llb::ops::source::{ImageSource, ResolveMode};
pub use buildkit_llb::ops::Terminal;
pub use buildkit_proto::moby::buildkit::v1::frontend::FileRange;

use crate::error::ErrorCode;
use crate::oci::ImageSpecification;
use crate::options::common::CacheOptionsEntry;
use crate::utils::OutputRef;

type BridgeConnection = tower_request_modifier::RequestModifier<Connection<BoxBody>, BoxBody>;

#[derive(Clone)]
pub struct Bridge {
    client: Arc<Mutex<client::LlbBridge<BridgeConnection>>>,
}

impl Bridge {
    pub(crate) fn new(client: BridgeConnection) -> Self {
        Self {
            client: Arc::new(Mutex::new(client::LlbBridge::new(client))),
        }
    }

    pub async fn resolve_image_config(
        &self,
        image: &ImageSource,
        log: Option<&str>,
    ) -> Result<(String, ImageSpecification), Error> {
        let request = ResolveImageConfigRequest {
            r#ref: image.canonical_name(),
            platform: None,
            resolve_mode: image.resolve_mode().unwrap_or_default().to_string(),
            log_name: log.unwrap_or_default().into(),
        };

        debug!("requesting to resolve an image: {:?}", request);
        let response = {
            self.client
                .lock()
                .await
                .resolve_image_config(Request::new(request))
                .compat()
                .await
                .unwrap()
                .into_inner()
        };

        Ok((
            response.digest,
            serde_json::from_slice(&response.config)
                .context("Unable to parse image specification")?,
        ))
    }

    pub async fn solve<'a, 'b: 'a>(&'a self, graph: Terminal<'b>) -> Result<OutputRef, Error> {
        self.solve_with_cache(graph, &[]).await
    }

    pub async fn solve_with_cache<'a, 'b: 'a>(
        &'a self,
        graph: Terminal<'b>,
        cache: &[CacheOptionsEntry],
    ) -> Result<OutputRef, Error> {
        debug!("serializing a graph to request");
        let request = SolveRequest {
            definition: Some(graph.into_definition()),
            exporter_attr: vec![],
            allow_result_return: true,
            cache_imports: cache.iter().cloned().map(Into::into).collect(),

            ..Default::default()
        };

        debug!("solving with cache from: {:?}", cache);
        debug!("requesting to solve a graph");
        let response = {
            self.client
                .lock()
                .await
                .solve(Request::new(request))
                .compat()
                .await
                .context("Unable to solve the graph")?
                .into_inner()
                .result
                .ok_or_else(|| format_err!("Unable to extract solve result"))?
        };

        debug!("got response: {:#?}", response);

        let inner = {
            response
                .result
                .ok_or_else(|| format_err!("Unable to extract solve result"))?
        };

        match inner {
            RefResult::Ref(inner) => Ok(OutputRef(inner)),
            other => bail!("Unexpected solve response: {:?}", other),
        }
    }

    pub async fn read_file<'a, 'b: 'a, P>(
        &'a self,
        layer: &'b OutputRef,
        path: P,
        range: Option<FileRange>,
    ) -> Result<Vec<u8>, Error>
    where
        P: Into<PathBuf>,
    {
        let file_path = path.into().display().to_string();
        debug!("requesting a file contents: {:#?}", file_path);

        let request = ReadFileRequest {
            r#ref: layer.0.clone(),
            file_path,
            range,
        };

        let response = {
            self.client
                .lock()
                .await
                .read_file(Request::new(request))
                .compat()
                .await
                .context("Unable to read the file")?
                .into_inner()
                .data
        };

        Ok(response)
    }

    pub(crate) async fn finish_with_success(
        self,
        output: OutputRef,
        config: Option<ImageSpecification>,
    ) -> Result<(), Error> {
        let mut metadata = HashMap::new();

        if let Some(config) = config {
            metadata.insert("containerimage.config".into(), serde_json::to_vec(&config)?);
        }

        let request = ReturnRequest {
            error: None,
            result: Some(Output {
                result: Some(RefResult::Ref(output.0)),
                metadata,
            }),
        };

        self.client
            .lock()
            .await
            .r#return(Request::new(request))
            .compat()
            .await?;

        // TODO: gracefully shutdown the HTTP/2 connection

        Ok(())
    }

    pub(crate) async fn finish_with_error<S>(self, code: ErrorCode, message: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let request = ReturnRequest {
            result: None,
            error: Some(Status {
                code: code as i32,
                message: message.into(),
                details: vec![],
            }),
        };

        debug!("sending an error result: {:#?}", request);
        self.client
            .lock()
            .await
            .r#return(Request::new(request))
            .compat()
            .await?;

        // TODO: gracefully shutdown the HTTP/2 connection

        Ok(())
    }
}
