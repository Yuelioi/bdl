pub mod paged;
pub mod uploader;
pub mod video;

use async_trait::async_trait;

use crate::BdlResult;
use crate::input::ClassifiedInput;
use crate::model::NormalizedSourceTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResolveOptions {
    pub fetch_streams: bool,
}

#[async_trait]
pub trait Resolver {
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree>;
}
