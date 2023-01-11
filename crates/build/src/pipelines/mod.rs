use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use context::PipelineCtx;
use elements_model_import::model_crate::ModelCrate;
use elements_std::{
    asset_cache::AssetCache, asset_url::{AbsAssetUrl, AssetType}
};
use futures::{
    future::{join_all, BoxFuture}, StreamExt
};
use image::ImageFormat;
use itertools::Itertools;
use out_asset::{OutAsset, OutAssetContent, OutAssetPreview};
use serde::{Deserialize, Serialize};

// pub mod audio;
pub mod context;
// pub mod materials;
// pub mod models;
pub mod out_asset;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineConfig {
    // Models(ModelsPipeline),
    ScriptBundles,
    // Materials(MaterialsPipeline),
    // Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct Pipeline {
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<Vec<String>>,
}
impl Pipeline {
    pub async fn process(&self, ctx: PipelineCtx) -> Vec<OutAsset> {
        let mut assets = match &self.pipeline {
            // PipelineConfig::Models(config) => models::pipeline(&ctx, config.clone()).await,
            PipelineConfig::ScriptBundles => {
                ctx.process_files(
                    |f| f.extension() == Some("script_bundle".to_string()),
                    |ctx, file| async move {
                        let bundle = file.download_bytes(&ctx.process_ctx.assets).await.unwrap();
                        let content = ctx.write_file(ctx.root.relative_path(file.path()).with_extension("script_bundle"), bundle).await;

                        Ok(vec![OutAsset {
                            sub_asset: None,
                            type_: AssetType::ScriptBundle,
                            hidden: false,
                            name: file.path().file_name().unwrap().to_string(),
                            tags: Vec::new(),
                            categories: Default::default(),
                            preview: OutAssetPreview::None,
                            content: OutAssetContent::Content(content),
                            source: Some(file.clone()),
                        }])
                    },
                )
                .await
            }
            // PipelineConfig::Materials(config) => materials::pipeline(&ctx, config.clone()).await,
            // PipelineConfig::Audio => audio::pipeline(&ctx).await,
            _ => todo!(),
        };
        let ctx = &ctx;
        for asset in &mut assets {
            asset.tags.extend(self.tags.clone());
            for i in 0..asset.categories.len() {
                if let Some(cat) = self.categories.get(i) {
                    asset.categories[i].extend(cat.iter().cloned().collect::<HashSet<_>>());
                }
            }
        }
        assets
    }
}

pub async fn process_pipelines(ctx: &ProcessCtx) -> Vec<OutAsset> {
    futures::stream::iter(ctx.files.iter())
        .filter_map(|file| async move {
            let pipelines: Vec<Pipeline> = if file.0.path().ends_with("pipeline.toml") {
                file.download_toml(&ctx.assets).await.unwrap()
            } else if file.0.path().ends_with("pipeline.json") {
                file.download_json(&ctx.assets).await.unwrap()
            } else {
                return None;
            };
            Some((file, pipelines))
        })
        .flat_map(|(file, pipelines)| futures::stream::iter(pipelines.into_iter().map(|pipeline| (file.clone(), pipeline))))
        .then(|(file, pipeline)| async move {
            let ctx = PipelineCtx { process_ctx: ctx.clone(), pipeline: Arc::new(pipeline.clone()), root: file };
            pipeline.process(ctx).await
        })
        .flat_map(|out_assets| futures::stream::iter(out_assets.into_iter()))
        .collect::<Vec<_>>()
        .await
}

#[derive(Clone)]
pub struct ProcessCtx {
    pub assets: AssetCache,
    pub files: Arc<Vec<AbsAssetUrl>>,
    pub input_file_filter: Option<String>,
    pub write_file: Arc<dyn Fn(String, Vec<u8>) -> BoxFuture<'static, AbsAssetUrl> + Sync + Send>,
    pub on_status: Arc<dyn Fn(String) -> BoxFuture<'static, ()> + Sync + Send>,
    pub on_error: Arc<dyn Fn(anyhow::Error) -> BoxFuture<'static, ()> + Sync + Send>,
}

pub async fn download_image(assets: &AssetCache, url: &AbsAssetUrl, extension: &Option<String>) -> anyhow::Result<image::DynamicImage> {
    let data = url.download_bytes(assets).await?;
    if let Some(format) = extension.as_ref().and_then(|ext| ImageFormat::from_extension(ext)) {
        Ok(image::load_from_memory_with_format(&data, format).with_context(|| format!("Failed to load image {url}"))?)
    } else {
        Ok(image::load_from_memory(&data).with_context(|| format!("Failed to load image {url}"))?)
    }
}
