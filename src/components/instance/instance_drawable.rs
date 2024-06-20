use super::{fragment, vertex, Instance, InstanceData, InstanceEntity};
use hex::{
    anyhow,
    assets::{shape::Vertex2, Shape},
    components::{Camera, Sprite, Trans},
    renderer_manager::{Draw, Renderer},
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
                viewport::ViewportState,
                GraphicsPipelineCreateInfo,
            },
            layout::PipelineDescriptorSetLayoutCreateInfo,
            GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
            PipelineShaderStageCreateInfo,
        },
        render_pass::Subpass,
        shader::EntryPoint,
    },
    ComponentManager, Context, Drawable, EntityManager, Id,
};
use std::sync::{Arc, RwLock};

pub struct InstanceDrawable;

impl InstanceDrawable {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Drawable<(i32, Shape, Vec<InstanceEntity>)> for InstanceDrawable {
    fn draw(
        &self,
        (l, s, i): (i32, Shape, Vec<InstanceEntity>),
        (_, ct, c): (Id, Arc<RwLock<Trans>>, Arc<RwLock<Camera>>),
        context: &Context,
        (_, builder, recreate_swapchain): &mut Draw,
        _: &EntityManager,
        _: &ComponentManager,
    ) -> anyhow::Result<()> {
        if !i.is_empty() {
            let (_, _, instance) = i.first().unwrap();
            let instance = instance.read().unwrap();
            let pipeline = {
                if *recreate_swapchain {
                    let (vertex, fragment) = &*instance.shaders;

                    *instance.pipeline.write().unwrap() =
                        Instance::pipeline(context, vertex.clone(), fragment.clone())?;
                }

                instance.pipeline.read().unwrap()
            };
            let c = c.read().unwrap();
            let ct = ct.read().unwrap();
            let z = c.calculate_z(l);
            let instance_data = {
                let instance_data: Vec<_> = i
                    .iter()
                    .map(|(_, t, i)| {
                        let i = i.write().unwrap();
                        let t = t.read().unwrap();
                        let t: [[f32; 3]; 3] = t.matrix().into();

                        InstanceData {
                            color: i.color.into(),
                            transform_x: t[0],
                            transform_y: t[1],
                            transform_z: t[2],
                        }
                    })
                    .collect();

                instance_data
            };
            let view = {
                let layout = pipeline.layout().set_layouts().first().unwrap();
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
                    z,
                };

                PersistentDescriptorSet::new(
                    &context.descriptor_set_allocator,
                    layout.clone(),
                    [WriteDescriptorSet::buffer(0, subbuffer)],
                    [],
                )?
            };
            let texture = {
                let layout = pipeline.layout().set_layouts().get(1).unwrap();

                PersistentDescriptorSet::new(
                    &context.descriptor_set_allocator,
                    layout.clone(),
                    [
                        WriteDescriptorSet::sampler(0, instance.texture.sampler.clone()),
                        WriteDescriptorSet::image_view(1, instance.texture.image.clone()),
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
                instance_data,
            )?;

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    view.clone(),
                )?
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    1,
                    texture.clone(),
                )?
                .bind_vertex_buffers(0, (s.vertices.clone(), instance_buffer.clone()))?
                .draw(s.vertices.len() as u32, instance_buffer.len() as u32, 0, 0)?;
        }

        Ok(())
    }
}
