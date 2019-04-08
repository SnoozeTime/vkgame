use std::sync::Arc;
use vk_sys;
use vulkano::device::Device;
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::framebuffer::AttachmentDescription;
use vulkano::framebuffer::PassDependencyDescription;
use vulkano::framebuffer::PassDescription;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::RenderPassDesc;
use vulkano::framebuffer::RenderPassDescClearValues;
use vulkano::framebuffer::{LoadOp, StoreOp};
use vulkano::image::ImageLayout;
use vulkano::sync::AccessFlagBits;
use vulkano::sync::PipelineStages;

/// Will build the two render pass of the renderer
pub fn build_render_pass(
    device: Arc<Device>,
    final_output_format: Format,
) -> (
    Arc<RenderPassAbstract + Send + Sync>,
    Arc<RenderPassAbstract + Send + Sync>,
    Arc<RenderPassAbstract + Send + Sync>,
) {
    (
        Arc::new(
            ShadowRenderPassDesc::new((Format::D16Unorm, 1), (Format::R16G16B16A16Sfloat, 1))
                .build_render_pass(device.clone())
                .unwrap(),
        ),
        Arc::new(
            OffscreenRenderPassDesc::new(
                (final_output_format, 1),
                (Format::A2B10G10R10UnormPack32, 1),
                (Format::R16G16B16A16Sfloat, 1),
                (Format::R16G16B16A16Sfloat, 1),
                (Format::D16Unorm, 1),
            )
            .build_render_pass(device.clone())
            .unwrap(),
        ),
        Arc::new(
            OnscreenRenderPassDesc::new((final_output_format, 1))
                .build_render_pass(device.clone())
                .unwrap(),
        ),
    )
}

/// Render shadowmaps
struct ShadowRenderPassDesc {
    attachments: Vec<AttachmentDescription>,
    passes: Vec<PassDescription>,
    dependencies: Vec<PassDependencyDescription>,
}

#[allow(unsafe_code)]
unsafe impl RenderPassDesc for ShadowRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<AttachmentDescription> {
        self.attachments
            .get(id)
            .map(|attachment| attachment.clone())
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        self.passes.len()
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<PassDescription> {
        self.passes.get(id).map(|pass| pass.clone())
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        self.dependencies.len()
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<PassDependencyDescription> {
        self.dependencies.get(id).map(|dep| dep.clone())
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for ShadowRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        Box::new(values.into_iter())
    }
}

impl ShadowRenderPassDesc {
    pub fn new(shadow_depth: (Format, u32), debug_color: (Format, u32)) -> Self {
        let mut attachments = Vec::new();
        // This is the shadow map attachment.
        attachments.push(AttachmentDescription {
            format: shadow_depth.0,
            samples: shadow_depth.1,
            load: LoadOp::Clear,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::DepthStencilAttachmentOptimal,
        });
        attachments.push(AttachmentDescription {
            format: debug_color.0,
            samples: debug_color.1,
            load: LoadOp::Clear,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        let mut passes = Vec::new();
        // draw the shadow map first.
        passes.push(PassDescription {
            color_attachments: vec![(1, ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: Some((0, ImageLayout::DepthStencilAttachmentOptimal)),
            input_attachments: vec![],
            resolve_attachments: vec![],
            preserve_attachments: vec![],
        });

        let mut dependencies = Vec::new();

        dependencies.push(PassDependencyDescription {
            source_subpass: vk_sys::SUBPASS_EXTERNAL as usize,
            destination_subpass: 0,
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            source_access: AccessFlagBits::all(), // TODO: correct values
            destination_access: AccessFlagBits::all(), // TODO: correct values
            by_region: true,                      // TODO: correct values
        });

        ShadowRenderPassDesc {
            attachments,
            passes,
            dependencies,
        }
    }
}

/// This render pass will render the scene to g-buffer.
/// G-Buffer is a number of color attachments that will store
/// - diffuse color
/// - fragment position in our referencial coordinates
/// - normals
/// It will also contains the depth attachment.
///
/// There is a dependency between this pass and the other pass.
/// That is specified in PassDependencyDescription.
struct OffscreenRenderPassDesc {
    attachments: Vec<AttachmentDescription>,
    passes: Vec<PassDescription>,
    dependencies: Vec<PassDependencyDescription>,
}

#[allow(unsafe_code)]
unsafe impl RenderPassDesc for OffscreenRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<AttachmentDescription> {
        self.attachments
            .get(id)
            .map(|attachment| attachment.clone())
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        self.passes.len()
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<PassDescription> {
        self.passes.get(id).map(|pass| pass.clone())
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        self.dependencies.len()
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<PassDependencyDescription> {
        self.dependencies.get(id).map(|dep| dep.clone())
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for OffscreenRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        Box::new(values.into_iter())
    }
}

impl OffscreenRenderPassDesc {
    pub fn new(
        final_color: (Format, u32),
        diffuse: (Format, u32),
        normals: (Format, u32),
        fragment_pos: (Format, u32),
        depth: (Format, u32),
    ) -> Self {
        let mut attachments = Vec::new();
        attachments.push(AttachmentDescription {
            format: final_color.0,
            samples: final_color.1,
            load: LoadOp::Clear,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        attachments.push(AttachmentDescription {
            format: diffuse.0,
            samples: diffuse.1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        attachments.push(AttachmentDescription {
            format: normals.0,
            samples: normals.1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        attachments.push(AttachmentDescription {
            format: fragment_pos.0,
            samples: fragment_pos.1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        attachments.push(AttachmentDescription {
            format: depth.0,
            samples: depth.1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::DepthStencilAttachmentOptimal,
        });
        let mut passes = Vec::new();

        // Only one subpass to render the scene to g-buffer
        passes.push(PassDescription {
            color_attachments: vec![
                (1, ImageLayout::ColorAttachmentOptimal),
                (2, ImageLayout::ColorAttachmentOptimal),
                (3, ImageLayout::ColorAttachmentOptimal),
            ],
            depth_stencil: Some((4, ImageLayout::DepthStencilAttachmentOptimal)),
            input_attachments: vec![],
            resolve_attachments: vec![],
            preserve_attachments: vec![],
        });

        // Lighting pass
        passes.push(PassDescription {
            color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: None,
            input_attachments: vec![
                (1, ImageLayout::ShaderReadOnlyOptimal),
                (2, ImageLayout::ShaderReadOnlyOptimal),
                (3, ImageLayout::ShaderReadOnlyOptimal),
                (4, ImageLayout::ShaderReadOnlyOptimal),
            ],
            resolve_attachments: vec![],
            preserve_attachments: vec![],
        });

        // Skybox pass
        passes.push(PassDescription {
            color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: Some((4, ImageLayout::DepthStencilAttachmentOptimal)),
            input_attachments: vec![],
            resolve_attachments: vec![],
            preserve_attachments: vec![],
        });

        let mut dependencies = Vec::new();

        dependencies.push(PassDependencyDescription {
            source_subpass: vk_sys::SUBPASS_EXTERNAL as usize,
            destination_subpass: 0,
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            source_access: AccessFlagBits::all(), // TODO: correct values
            destination_access: AccessFlagBits::all(), // TODO: correct values
            by_region: true,                      // TODO: correct values
        });

        dependencies.push(PassDependencyDescription {
            source_subpass: 0,
            destination_subpass: 1,
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            source_access: AccessFlagBits::all(), // TODO: correct values
            destination_access: AccessFlagBits::all(), // TODO: correct values
            by_region: true,                      // TODO: correct values
        });

        dependencies.push(PassDependencyDescription {
            source_subpass: 1,
            destination_subpass: 2,
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            source_access: AccessFlagBits::all(), // TODO: correct values
            destination_access: AccessFlagBits::all(), // TODO: correct values
            by_region: true,                      // TODO: correct values
        });

        dependencies.push(PassDependencyDescription {
            source_subpass: 2,
            // outside of render pass
            destination_subpass: vk_sys::SUBPASS_EXTERNAL as usize,

            // TODO Correct values
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            },
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            },
            source_access: AccessFlagBits::all(),
            destination_access: AccessFlagBits::all(),
            by_region: true,
        });

        OffscreenRenderPassDesc {
            attachments,
            passes,
            dependencies,
        }
    }
}

/// This render pass will use the g-buffer and the lights to
/// render the scene to the screen. It also has subpasses for
/// post processing and GUI overlay
pub struct OnscreenRenderPassDesc {
    attachments: Vec<AttachmentDescription>,
    passes: Vec<PassDescription>,
    dependencies: Vec<PassDependencyDescription>,
}

#[allow(unsafe_code)]
unsafe impl RenderPassDesc for OnscreenRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<AttachmentDescription> {
        self.attachments
            .get(id)
            .map(|attachment| attachment.clone())
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        self.passes.len()
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<PassDescription> {
        self.passes.get(id).map(|pass| pass.clone())
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        self.dependencies.len()
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<PassDependencyDescription> {
        self.dependencies.get(id).map(|dep| dep.clone())
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for OnscreenRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        Box::new(values.into_iter())
    }
}

impl OnscreenRenderPassDesc {
    pub fn new(final_color: (Format, u32)) -> Self {
        let mut attachments = Vec::new();
        attachments.push(AttachmentDescription {
            format: final_color.0,
            samples: final_color.1,
            load: LoadOp::DontCare,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::Store,
            initial_layout: ImageLayout::ColorAttachmentOptimal,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        let mut passes = Vec::new();

        // GUI pass
        passes.push(PassDescription {
            color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: None,
            input_attachments: vec![],
            resolve_attachments: vec![],
            preserve_attachments: vec![],
        });

        let mut dependencies = Vec::new();

        dependencies.push(PassDependencyDescription {
            source_subpass: vk_sys::SUBPASS_EXTERNAL as usize,
            destination_subpass: 0,
            source_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            destination_stages: PipelineStages {
                all_graphics: true,
                ..PipelineStages::none()
            }, // TODO: correct values
            source_access: AccessFlagBits::all(), // TODO: correct values
            destination_access: AccessFlagBits::all(), // TODO: correct values
            by_region: true,                      // TODO: correct values
        });

        OnscreenRenderPassDesc {
            attachments,
            passes,
            dependencies,
        }
    }
}
