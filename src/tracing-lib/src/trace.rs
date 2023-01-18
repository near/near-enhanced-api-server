use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::Project;
use std::sync::Arc;

use opentelemetry::runtime::Tokio;
use opentelemetry::sdk::trace::{Config, Sampler, TracerProvider};

use opentelemetry_stackdriver::{Authorizer, Error, LogContext, MonitoredResource, StackDriverExporter};
use tokio::task::JoinHandle;
use tonic::metadata::MetadataValue;
use tonic::Request;

use async_trait::async_trait;
use opentelemetry::sdk::trace;
use tracing_subscriber::layer::SubscriberExt;

use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::util::SubscriberInitExt;

struct TraceAuthorizer {
    project_id: String,
    ts: Arc<dyn TokenSource>,
}

impl TraceAuthorizer {
    async fn new(project: Project) -> Self {
        let ts = google_cloud_auth::create_token_source_from_project(
            &project,
            google_cloud_auth::Config {
                audience: None,
                scopes: Some(&[
                    "https://www.googleapis.com/auth/trace.append",
                    "https://www.googleapis.com/auth/logging.write",
                ]),
            },
        )
        .await
        .unwrap();
        TraceAuthorizer {
            project_id: project.project_id().unwrap().to_string(),
            ts: Arc::from(ts),
        }
    }
}

#[async_trait]
impl Authorizer for TraceAuthorizer {
    type Error = opentelemetry_stackdriver::Error;

    fn project_id(&self) -> &str {
        self.project_id.as_str()
    }

    async fn authorize<T: Send + Sync>(&self, req: &mut Request<T>, _scopes: &[&str]) -> Result<(), Self::Error> {
        let token = self
            .ts
            .token()
            .await
            .map_err(|e| Error::Authorizer(e.into()))?
            .access_token;
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::from_str(&format!("Bearer {}", token.as_str())).unwrap(),
        );
        Ok(())
    }
}

pub struct Tracer {
    j: Option<JoinHandle<()>>,
    pub provider: TracerProvider,
    pub project: Option<Project>,
}

impl Tracer {
    fn create_provider(builder: trace::Builder) -> TracerProvider {
        builder
            .with_config(Config {
                sampler: Box::new(Sampler::AlwaysOn),
                ..Default::default()
            })
            .build()
    }

    pub async fn done(&mut self) {
        if let Some(j) = &mut self.j {
            let _ = j.await;
        }
    }

    pub async fn default() -> Self {
        let project = google_cloud_auth::project().await;
        let mut builder = TracerProvider::builder();
        let tracer = if let Ok(project) = project {
            let log_context = LogContext {
                log_id: "cloud-trace-example".into(),
                resource: MonitoredResource::Global {
                    project_id: project.project_id().unwrap().to_string(),
                },
            };
            let auth = TraceAuthorizer::new(project.clone()).await;
            let (exporter, driver) = StackDriverExporter::builder()
                .log_context(log_context)
                .num_concurrent_requests(2)
                .build(auth)
                .await
                .unwrap();
            builder = builder.with_batch_exporter(exporter, Tokio);
            let j = tokio::spawn(driver);
            Self {
                j: Some(j),
                provider: Self::create_provider(builder),
                project: Some(project),
            }
        } else {
            Self {
                j: None,
                provider: Self::create_provider(builder),
                project: None,
            }
        };

        tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer.provider.tracer("tracing")))
            .with(tracing_stackdriver::Stackdriver::new())
            .with(tracing_subscriber::filter::EnvFilter::from_default_env())
            .init();

        tracer
    }
}
