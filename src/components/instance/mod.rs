pub mod fragment;
pub mod instance_data;
pub mod instance_drawable;
pub mod vertex;

pub use instance_data::InstanceData;
pub use instance_drawable::InstanceDrawable;

use hex::{
    anyhow,
    assets::{
        shape::{Shape, Vertex2},
        Texture,
    },
    component_manager::Component,
    components::Trans,
    nalgebra::Vector4,
    vulkano::{
        pipeline::{
            graphics::{
                color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
                depth_stencil::{DepthState, DepthStencilState},
                input_assembly::{InputAssemblyState, PrimitiveTopology},
                multisample::MultisampleState,
                rasterization::RasterizationState,
                vertex_input::{Vertex, VertexDefinition},
                viewport::ViewportState,
                GraphicsPipelineCreateInfo,
            },
            layout::PipelineDescriptorSetLayoutCreateInfo,
            GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        },
        render_pass::Subpass,
        shader::EntryPoint,
    },
    Context, Drawable, Id,
};
use std::sync::{Arc, RwLock};

pub type InstanceEntity = (Id, Arc<RwLock<Trans>>, Arc<RwLock<Instance>>);

#[derive(Clone)]
pub struct Instance {
    pub texture: Arc<Texture>,
    pub pipeline: Arc<RwLock<Arc<GraphicsPipeline>>>,
    pub drawable: Arc<dyn Drawable<(f32, Shape, Vec<InstanceEntity>)>>,
    pub shaders: Arc<(EntryPoint, EntryPoint)>,
    pub color: Vector4<f32>,
    pub layer: i32,
    pub active: bool,
}

impl Instance {
    pub fn new(
        context: &Context,
        texture: Arc<Texture>,
        color: Vector4<f32>,
        layer: i32,
        active: bool,
    ) -> anyhow::Result<Self> {
        let vertex = vertex::load(context.device.clone())?
            .entry_point("main")
            .unwrap();
        let fragment = fragment::load(context.device.clone())?
            .entry_point("main")
            .unwrap();

        Ok(Self {
            texture,
            pipeline: Arc::new(RwLock::new(Self::pipeline(
                context,
                vertex.clone(),
                fragment.clone(),
            )?)),
            shaders: Arc::new((vertex, fragment)),
            drawable: InstanceDrawable::new(),
            color,
            layer,
            active,
        })
    }

    pub fn pipeline(
        context: &Context,
        vertex: EntryPoint,
        fragment: EntryPoint,
    ) -> anyhow::Result<Arc<GraphicsPipeline>> {
        let vertex_input_state = [Vertex2::per_vertex(), InstanceData::per_instance()]
            .definition(&vertex.info().input_interface)?;
        let stages = [
            PipelineShaderStageCreateInfo::new(vertex),
            PipelineShaderStageCreateInfo::new(fragment),
        ];
        let layout = PipelineLayout::new(
            context.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(context.device.clone())?,
        )?;
        let subpass = Subpass::from(context.render_pass.clone(), 0).unwrap();

        Ok(GraphicsPipeline::new(
            context.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::TriangleFan,
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState {
                    viewports: [context.viewport.clone()].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend::alpha()),
                        ..Default::default()
                    },
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )?)
    }
}

impl Component for Instance {}
