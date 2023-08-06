use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use bevy::asset::LoadedAsset;

use super::BuildingTemplate;

pub struct BuildingTemplateLoader;

impl bevy::asset::AssetLoader for BuildingTemplateLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();

            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(bytes),
            );
            let tilemap = loader
                .load_tmx_map(path)
                .map_err(|e| anyhow::anyhow!("Could not load TMX map: {e}"))?;

            let building_type = path
                .file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".building.tmx"))
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow::anyhow!("unknown building {}", path.display()))?;

            let template = BuildingTemplate::from_tilemap(building_type, tilemap)?;

            load_context.set_default_asset(LoadedAsset::new(template));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["building.tmx"];
        EXTENSIONS
    }
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        // In this case, the path is ignored because the byte data is already provided.
        Ok(Cursor::new(self.bytes.clone()))
    }
}
