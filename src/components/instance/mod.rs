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
    components::Trans,
    nalgebra::Vector4,
    parking_lot::RwLock,
    vulkano::{
        pipeline::{
            graphics::{
                color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
                depth_stencil::{CompareOp, DepthState, DepthStencilState},
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
use std::sync::Arc;

pub type InstanceEntity = (Id, Arc<RwLock<Instance>>, Arc<RwLock<Trans>>);
pub type InstancePipeline = (RwLock<Arc<GraphicsPipeline>>, EntryPoint, EntryPoint);

#[derive(Clone)]
pub struct Instance {
    pub shape: Arc<Shape>,
    pub texture: Arc<Texture>,
    pub color: Vector4<f32>,
    pub layer: i32,
    pub pipeline: Arc<InstancePipeline>,
    pub drawable: Arc<dyn Drawable<Vec<InstanceEntity>>>,
}

impl Instance {
    pub fn new(
        context: &Context,
        shape: Arc<Shape>,
        texture: Arc<Texture>,
        color: Vector4<f32>,
        layer: i32,
    ) -> anyhow::Result<Arc<RwLock<Self>>> {
        let vertex = vertex::load(context.device.clone())?
            .entry_point("main")
            .unwrap();
        let fragment = fragment::load(context.device.clone())?
            .entry_point("main")
            .unwrap();

        Ok(Arc::new(RwLock::new(Self {
            shape,
            texture,
            color,
            layer,
            pipeline: Arc::new((
                RwLock::new(Self::pipeline(context, vertex.clone(), fragment.clone())?),
                vertex,
                fragment,
            )),
            drawable: InstanceDrawable::new(),
        })))
    }

    pub fn recreate_pipeline(&self, context: &Context) -> anyhow::Result<()> {
        let (ref pipeline, ref vertex, ref fragment) = &*self.pipeline;

        *pipeline.write() = Self::pipeline(context, vertex.clone(), fragment.clone())?;

        Ok(())
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
                    depth: Some(DepthState {
                        write_enable: true,
                        compare_op: CompareOp::LessOrEqual,
                    }),
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
