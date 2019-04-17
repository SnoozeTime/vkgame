use crate::renderer::AttachmentType;
use std::default::Default;

#[derive(Debug, Clone, Copy, Default)]
pub struct GameConfig {
    pub renderer_config: RenderOptions,
}

// -----------------------------------------

/// RenderOptions will enable/disable rendering features. It can be to adjust to
/// weaker computers or to add some debugging information
#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub display_outlines: bool,
    pub show_shadowmap: bool,
    pub show_shadowmap_color: bool,
    pub attachment_to_show: Option<AttachmentType>,
}

impl Default for RenderOptions {
    fn default() -> RenderOptions {
        RenderOptions {
            display_outlines: true,
            show_shadowmap: false,
            show_shadowmap_color: false,
            attachment_to_show: None,
        }
    }
}
