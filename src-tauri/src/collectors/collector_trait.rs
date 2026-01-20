use crate::database::models::{Channel, StreamStats};
use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait Collector {
    async fn poll_channel(
        &self,
        channel: &Channel,
    ) -> Result<Option<StreamStats>, Box<dyn std::error::Error>>;
    async fn start_collection(&self, channel: &Channel) -> Result<(), Box<dyn std::error::Error>>;
}
