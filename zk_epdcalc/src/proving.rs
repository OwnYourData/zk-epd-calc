use anyhow::{Error, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use risc0_zkvm::{ExecutorEnv, ExecutorEnvBuilder, ExternalProver, Prover, ProverOpts, Receipt};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::{JoinHandle, JoinSet},
    time::Instant,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::error::ProvingError;
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use zk_epdcalc_core::VerifiedEpd;

#[derive(Clone)]
pub(crate) enum TaskStatus {
    Submitted,
    InProgress,
    //Cancelled,
    Complete(Result<Arc<dyn ProofResponse>, AppError>),
}

impl TaskStatus {
    pub(crate) fn get_response(self) -> Result<Arc<dyn ProofResponse>, AppError> {
        match self {
            TaskStatus::Submitted | TaskStatus::InProgress => Err(AppError::TaskNotCompleted),
            TaskStatus::Complete(response) => response,
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = match self {
            TaskStatus::Submitted => "Submitted",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Complete(_) => "Complete",
        };
        serializer.serialize_str(repr)
    }
}

pub enum Command {
    Status {
        id: Uuid,
        resp: Sender<Result<TaskStatus, AppError>>,
    },
    Generate {
        id: Uuid,
        config: Box<dyn ProofConfig>,
    },
    Complete {
        id: Uuid,
        zk_epd: Result<Arc<dyn ProofResponse>, ProvingError>,
    },
}

async fn next_cmd(
    cmd_rx: &mut UnboundedReceiver<Command>,
    join_set: &mut JoinSet<Command>,
) -> Option<Command> {
    // Create a stream that yields Some(Command) when a command is received,
    // or None when the channel is closed.
    let cmd_stream = futures::stream::poll_fn(|cx| cmd_rx.poll_recv(cx));

    // Create a stream from the JoinSet, returning the results of the generation tasks.
    let join_stream = futures::stream::poll_fn(|cx| join_set.poll_join_next(cx))
        .map(|result| result.expect("Task in JoinSet failed unexpectedly"));

    // Merge the two streams
    futures::stream::select(cmd_stream, join_stream)
        .next()
        .await
}

async fn proving_service(mut cmd_rx: UnboundedReceiver<Command>) {
    let mut tasks: HashMap<Uuid, TaskStatus> = HashMap::new();
    let mut join_set = JoinSet::new();

    while let Some(cmd) = next_cmd(&mut cmd_rx, &mut join_set).await {
        match cmd {
            Command::Status { id, resp } => {
                let result = tasks.get(&id).ok_or(AppError::UnknownTask(id)).cloned();
                let _ = resp.send(result);
            }
            Command::Generate { id, config } => {
                tasks.insert(id, TaskStatus::InProgress);
                join_set.spawn_blocking(move || {
                    let zk_epd = generate_epd(config);
                    Command::Complete { id, zk_epd }
                });
            }
            Command::Complete { id, zk_epd } => {
                let task = tasks.get_mut(&id).unwrap_or_else(|| {
                    panic!("Failure while adding EPD to unknown Task. Id: {id}")
                });
                let zk_epd = zk_epd.map_err(Into::into);
                *task = TaskStatus::Complete(zk_epd);
            }
        }
    }
}

#[derive(Clone)]
pub struct ProvingService {
    tx: UnboundedSender<Command>,
}

impl ProvingService {
    pub fn new() -> (ProvingService, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let proving_task = tokio::spawn(async {
            proving_service(rx).await;
        });
        (Self { tx }, proving_task)
    }

    #[tracing::instrument(skip(self))]
    pub fn add_task(&self, config: Box<dyn ProofConfig>) -> Uuid {
        let id = Uuid::new_v4();
        let cmd = Command::Generate { id, config };
        self.tx
            .send(cmd)
            .expect("Command channel was irregularly closed");
        id
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_status(&self, id: Uuid) -> Result<TaskStatus, AppError> {
        let (resp, rx) = oneshot::channel();
        let cmd = Command::Status { id, resp };
        self.tx
            .send(cmd)
            .expect("Command channel was irregularly closed");
        rx.await.expect("Response Channel was close unexpectedly")
    }
}

trait WriteConfigExt {
    fn write_config(&mut self, config: &dyn ProofConfig) -> Result<&mut Self, Error>;
}

#[tracing::instrument]
fn generate_proof(config: &dyn ProofConfig) -> Result<Receipt> {
    let env = ExecutorEnv::builder().write_config(config)?.build()?;

    let prover = ExternalProver::new("ipc", "r0vm");

    let opts = if config.snark_p() {
        ProverOpts::groth16()
    } else {
        ProverOpts::default()
    };

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest
    let start = Instant::now();
    let prove_info = prover.prove_with_opts(env, config.guest_elf(), &opts)?;
    let elapsed = start.elapsed();

    println!(
        "Created Proof in {} seconds. Stats:\nUser Cycles: {}, Total Cycles: {}, Segments: {}",
        elapsed.as_secs(),
        prove_info.stats.user_cycles,
        prove_info.stats.total_cycles,
        prove_info.stats.segments
    );

    // Return proof receipt
    Ok(prove_info.receipt)
}

pub fn generate_epd(config: Box<dyn ProofConfig>) -> Result<Arc<dyn ProofResponse>, ProvingError> {
    let receipt = generate_proof(&*config)?;

    let proof_response = config.decode_response(&receipt)?;
    Ok(Arc::from(proof_response))
}

impl<'a> WriteConfigExt for ExecutorEnvBuilder<'a> {
    fn write_config<'b>(&'b mut self, config: &dyn ProofConfig) -> Result<&'b mut Self, Error> {
        config.get_input().write_to_env(self)
    }
}

pub trait ProofInput {
    fn write_to_env<'a, 'b>(
        &self,
        env_builder: &'a mut ExecutorEnvBuilder<'b>,
    ) -> Result<&'a mut ExecutorEnvBuilder<'b>, Error>;
}

impl<T> ProofInput for T
where
    T: Serialize,
{
    fn write_to_env<'a, 'b>(
        &self,
        env_builder: &'a mut ExecutorEnvBuilder<'b>,
    ) -> Result<&'a mut ExecutorEnvBuilder<'b>, Error> {
        env_builder.write(self)
    }
}

pub fn new_config<I, Epd, ZkEpd>(
    snark_p: bool,
    guest_elf: &'static [u8],
    input: I,
) -> Box<dyn ProofConfig>
where
    I: ProofInput + Sync + Send + Debug + 'static,
    Epd: Sync + Send + Debug + DeserializeOwned + 'static,
    ZkEpd: VerifiedEpd<Epd> + Send + Sync + Debug + Serialize + 'static
{
    let config: ProofConfigImpl<I, Epd, ZkEpd> = ProofConfigImpl {
        snark_p,
        guest_elf,
        input,
        phantom_epd: Default::default(),
        phantom_zk_epd: Default::default(),
    };
    Box::new(config)
}

struct ProofConfigImpl<I, Epd, ZkEpd> {
    snark_p: bool,
    guest_elf: &'static [u8],
    input: I,
    phantom_epd: PhantomData<Epd>,
    phantom_zk_epd: PhantomData<ZkEpd>,
}

impl<I, Epd, ZkEpd> Debug for ProofConfigImpl<I, Epd, ZkEpd>
where
    I: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofConfig")
            .field("snark_p", &self.snark_p)
            .field("input:", &self.input)
            .finish()
    }
}

impl<I, Epd, ZkEpd> ProofConfig for ProofConfigImpl<I, Epd, ZkEpd>
where
    I: ProofInput + Send + Debug,
    Epd: DeserializeOwned + Send + Debug,
    ZkEpd: ProofResponse + VerifiedEpd<Epd> + Debug + Send + 'static
{
    fn get_input(&self) -> &dyn ProofInput {
        &self.input
    }

    fn snark_p(&self) -> bool {
        self.snark_p
    }

    fn guest_elf(&self) -> &[u8] {
        self.guest_elf
    }

    fn decode_response(&self, receipt: &Receipt) -> Result<Box<dyn ProofResponse>, ProvingError> {
        let epd: Epd = receipt.journal.decode()?;

        let mut receipt_cbor: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&receipt, &mut receipt_cbor)?;

        let zkp = BASE64_STANDARD.encode(&receipt_cbor).into_boxed_str();

        let zk_epd = ZkEpd::from_result(epd, zkp);
        Ok(Box::new(zk_epd))
    }
}

pub trait ProofConfig: Debug + Send {
    fn get_input(&self) -> &dyn ProofInput;

    fn snark_p(&self) -> bool;

    fn guest_elf(&self) -> &[u8];

    fn decode_response(&self, receipt: &Receipt) -> Result<Box<dyn ProofResponse>, ProvingError>;
}

pub trait ProofResponse: Send + Sync {
    fn into_response(self: Arc<Self>) -> Response;
}

impl<T> ProofResponse for T
where
    T: Serialize + Sync + Send,
{
    fn into_response(self: Arc<Self>) -> Response {
        Json(self).into_response()
    }
}
