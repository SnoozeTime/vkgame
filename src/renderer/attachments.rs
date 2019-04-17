/// Type for an attachment. This is mainly used for debugging currently
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AttachmentType {
    /// Color of fragment
    GBufferDiffuse,

    /// Normal of fragment
    GBufferNormal,

    /// World position of fragment
    GBufferPosition,

    /// Shadow map for light i
    ShadowMap(u8),

    /// Diffuse from pov of light i
    LightDiffuse(u8),
}

pub const DEBUG_ATTACHMENTS: [AttachmentType; 5] = [
    AttachmentType::GBufferDiffuse,
    AttachmentType::GBufferNormal,
    AttachmentType::GBufferPosition,
    AttachmentType::ShadowMap(0),
    AttachmentType::LightDiffuse(0),
];
