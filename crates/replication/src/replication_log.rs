pub mod rpc {
    #![allow(clippy::all)]
    tonic::include_proto!("wal_log");
}

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use crate::primary::frame_stream::FrameStream;
use crate::{LogReadError, ReplicationLogger};

use self::rpc::replication_log_server::ReplicationLog;
use self::rpc::{Frame, HelloRequest, HelloResponse, LogOffset};

pub struct ReplicationLogService {
    logger: Arc<ReplicationLogger>,
    replicas_with_hello: RwLock<HashSet<SocketAddr>>,
}

pub const NO_HELLO_ERROR_MSG: &str = "NO_HELLO";
pub const NEED_SNAPSHOT_ERROR_MSG: &str = "NEED_SNAPSHOT";

impl ReplicationLogService {
    pub fn new(logger: Arc<ReplicationLogger>) -> Self {
        Self {
            logger,
            replicas_with_hello: RwLock::new(HashSet::<SocketAddr>::new()),
        }
    }
}

fn map_frame_stream_output(r: Result<crate::frame::Frame, LogReadError>) -> Result<Frame, Status> {
    match r {
        Ok(frame) => Ok(Frame {
            data: frame.bytes(),
        }),
        Err(LogReadError::SnapshotRequired) => Err(Status::new(
            tonic::Code::FailedPrecondition,
            NEED_SNAPSHOT_ERROR_MSG,
        )),
        Err(LogReadError::Error(e)) => Err(Status::new(tonic::Code::Internal, e.to_string())),
        // this error should be caught before, but we handle it nicely anyways
        Err(LogReadError::Ahead) => Err(Status::new(
            tonic::Code::OutOfRange,
            "frame not yet available",
        )),
    }
}

#[tonic::async_trait]
impl ReplicationLog for ReplicationLogService {
    type LogEntriesStream = BoxStream<'static, Result<Frame, Status>>;
    type SnapshotStream = BoxStream<'static, Result<Frame, Status>>;

    async fn log_entries(
        &self,
        req: tonic::Request<LogOffset>,
    ) -> Result<tonic::Response<Self::LogEntriesStream>, Status> {
        let replica_addr = req
            .remote_addr()
            .ok_or(Status::internal("No remote RPC address"))?;
        {
            let guard = self.replicas_with_hello.read().unwrap();
            if !guard.contains(&replica_addr) {
                return Err(Status::failed_precondition(NO_HELLO_ERROR_MSG));
            }
        }

        let stream = FrameStream::new(self.logger.clone(), req.into_inner().next_offset)
            .map(map_frame_stream_output)
            .boxed();

        Ok(tonic::Response::new(stream))
    }

    async fn hello(
        &self,
        req: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloResponse>, Status> {
        let replica_addr = req
            .remote_addr()
            .ok_or(Status::internal("No remote RPC address"))?;
        {
            let mut guard = self.replicas_with_hello.write().unwrap();
            guard.insert(replica_addr);
        }
        let response = HelloResponse {
            database_id: self.logger.database_id().unwrap().to_string(),
            generation_start_index: self.logger.generation.start_index,
            generation_id: self.logger.generation.id.to_string(),
        };

        Ok(tonic::Response::new(response))
    }

    async fn snapshot(
        &self,
        req: tonic::Request<LogOffset>,
    ) -> Result<tonic::Response<Self::SnapshotStream>, Status> {
        let (sender, receiver) = mpsc::channel(10);
        let logger = self.logger.clone();
        let offset = req.into_inner().next_offset;
        match tokio::task::spawn_blocking(move || logger.get_snapshot_file(offset)).await {
            Ok(Ok(Some(snapshot))) => {
                tokio::task::spawn_blocking(move || {
                    let mut frames = snapshot.frames_iter_from(offset);
                    loop {
                        match frames.next() {
                            Some(Ok(data)) => {
                                let _ = sender.blocking_send(Ok(Frame { data }));
                            }
                            Some(Err(e)) => {
                                let _ = sender.blocking_send(Err(Status::new(
                                    tonic::Code::Internal,
                                    e.to_string(),
                                )));
                                break;
                            }
                            None => {
                                break;
                            }
                        }
                    }
                });

                Ok(tonic::Response::new(ReceiverStream::new(receiver).boxed()))
            }
            Ok(Ok(None)) => Err(Status::new(tonic::Code::Unavailable, "snapshot not found")),
            Err(e) => Err(Status::new(tonic::Code::Internal, e.to_string())),
            Ok(Err(e)) => Err(Status::new(tonic::Code::Internal, e.to_string())),
        }
    }
}

// TODO: original code accepted a config file and a bunch of TLS params, we need that here as well
pub fn configure_rpc(writer_rpc_addr: impl Into<String>) -> anyhow::Result<(tonic::transport::Channel, tonic::transport::Uri)> {
    let writer_rpc_addr = writer_rpc_addr.into();
    let endpoint = tonic::transport::Channel::from_shared(writer_rpc_addr.clone())?;

    let channel = endpoint.connect_lazy();
    let uri = tonic::transport::Uri::from_maybe_shared(writer_rpc_addr)?;

    Ok((channel, uri))
}

