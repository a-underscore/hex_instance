pub mod fragment;
pub mod instance_data;
pub mod vertex;

pub use instance_data::InstanceData;

use crate::components::Instance;
use hex::{
    anyhow,
    assets::{shape::Vertex2, Shape},
    components::{Camera, Trans},
    ecs::{
        renderer_manager::{Draw, Renderer},
        ComponentManager, Context, EntityManager,
    },
    vulkano::{
        buffer::{
            allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
            BufferUsage,
        },
        buffer::{Buffer, BufferCreateInfo},
        descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
        memory::allocator::AllocationCreateInfo,
        memory::allocator::MemoryTypeFilter,
        padded::Padded,
        pipeline::{
            graphics::{
                color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
                depth_stencil::{DepthState, DepthStencilState},
                input_assembly::{InputAssemblyState, PrimitiveTopology},
                multisample::MultisampleState,
                rasterization::RasterizationState,
                vertex_input::{Vertex, VertexDefinition},
                viewport::{Viewport, ViewportState},
                GraphicsPipelineCreateInfo,
            },
            layout::PipelineDescriptorSetLayoutCreateInfo,
            GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
            PipelineShaderStageCreateInfo,
        },
        render_pass::Subpass,
        shader::EntryPoint,
    },
};
use ordered_float::OrderedFloat;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct InstanceRenderer {
    pub vertex: EntryPoint,
    pub fragment: EntryPoint,
    pub pipeline: Arc<GraphicsPipeline>,
    pub shape: Shape,
}

impl InstanceRenderer {
    pub fn new(context: &Context, shape: Shape) -> anyhow::Result<Self> {
        let vertex = vertex::load(context.device.clone())?
            .entry_point("main")
            .unwrap();
        let fragment = fragment::load(context.device.clone())?
            .entry_point("main")
            .unwrap();
        let pipeline = Self::pipeline(context, vertex.clone(), fragment.clone())?;

        Ok(Self {
            vertex,
            fragment,
            pipeline,
            shape,
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
        let extent = context.images[0].extent();

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
                    viewports: [Viewport {
                        offset: [0.0, 0.0],
                        extent: [extent[0] as f32, extent[1] as f32],
                        depth_range: -1.0..=1.0,
                    }]
                    .into_iter()
                    .collect(),
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

impl Renderer for InstanceRenderer {
    fn draw(
        &mut self,
        Draw(_, builder, recreate_swapchain): &mut Draw,
        context: Arc<RwLock<Context>>,
        em: Arc<RwLock<EntityManager>>,
        cm: Arc<RwLock<ComponentManager>>,
    ) -> anyhow::Result<()> {
        let context = context.read().unwrap();

        if *recreate_swapchain {
            self.pipeline = Self::pipeline(&context, self.vertex.clone(), self.fragment.clone())?;
        }

        builder.bind_pipeline_graphics(self.pipeline.clone())?;

        let em = em.read().unwrap();
        let cm = cm.read().unwrap();

        if let Some((c, ct)) = em.entities().keys().cloned().find_map(|e| {
            Some((
                cm.get_ref::<Camera>(e)
                    .and_then(|c| c.active.then_some(c))?,
                cm.get_ref::<Trans>(e)
                    .and_then(|t| t.active.then_some(t))?,
            ))
        }) {
            let sprites = {
                let sprites = em
                    .entities()
                    .keys()
                    .cloned()
                    .filter_map(|e| {
                        Some((
                            cm.get_ref::<Instance>(e)
                                .and_then(|i| i.active.then_some(i))?,
                            cm.get_ref::<Trans>(e)
                                .and_then(|t| t.active.then_some(t))?,
                        ))
                    })
                    .fold(HashMap::<_, (_, Vec<_>)>::new(), |mut sprites, (i, t)| {
                        let (_, instances) = sprites
                            .entry((Arc::as_ptr(&i.texture), OrderedFloat(i.z)))
                            .or_insert((i.texture.clone(), Vec::new()));

                        instances.push((i.clone(), t.clone()));

                        sprites
                    });

                let mut sprites: Vec<_> = sprites
                    .into_iter()
                    .map(|((_, z), (t, i))| {
                        let instance_data: Vec<_> = i
                            .into_iter()
                            .map(|(s, t)| {
                                let t: [[f32; 3]; 3] = t.matrix().into();

                                InstanceData {
                                    z: s.z,
                                    color: s.color,
                                    transform_x: t[0],
                                    transform_y: t[1],
                                    transform_z: t[2],
                                }
                            })
                            .collect();

                        (z, instance_data, t)
                    })
                    .collect();

                sprites.sort_by_key(|(z, _, _)| *z);

                sprites
            };

            for (_, i, t) in sprites {
                let view = {
                    let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
                    let subbuffer_allocator = SubbufferAllocator::new(
                        context.memory_allocator.clone(),
                        SubbufferAllocatorCreateInfo {
                            buffer_usage: BufferUsage::UNIFORM_BUFFER,
                            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                            ..Default::default()
                        },
                    );
                    let subbuffer = subbuffer_allocator.allocate_sized()?;

                    *subbuffer.write()? = vertex::View {
                        camera_transform: <[[f32; 3]; 3]>::from(ct.matrix()).map(Padded),
                        camera_proj: c.proj().into(),
                    };

                    PersistentDescriptorSet::new(
                        &context.descriptor_set_allocator,
                        layout.clone(),
                        [WriteDescriptorSet::buffer(0, subbuffer)],
                        [],
                    )?
                };
                let texture = {
                    let layout = self.pipeline.layout().set_layouts().get(1).unwrap();

                    PersistentDescriptorSet::new(
                        &context.descriptor_set_allocator,
                        layout.clone(),
                        [
                            WriteDescriptorSet::sampler(0, t.sampler.clone()),
                            WriteDescriptorSet::image_view(1, t.image.clone()),
                        ],
                        [],
                    )?
                };
                let instance_buffer = Buffer::from_iter(
                    context.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::VERTEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    i,
                )?;

                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        0,
                        view.clone(),
                    )?
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        1,
                        texture.clone(),
                    )?
                    .bind_vertex_buffers(0, (self.shape.vertices.clone(), instance_buffer.clone()))?
                    .draw(
                        self.shape.vertices.len() as u32,
                        instance_buffer.len() as u32,
                        0,
                        0,
                    )?;
            }
        }

        Ok(())
    }
}
