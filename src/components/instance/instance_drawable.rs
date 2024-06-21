use super::{vertex, InstanceData, InstanceEntity};
use hex::{
    anyhow,
    components::{Camera, Trans},
    renderer_manager::Draw,
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
        pipeline::{Pipeline, PipelineBindPoint},
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

impl Drawable<Vec<InstanceEntity>> for InstanceDrawable {
    fn draw(
        &self,
        i: Vec<InstanceEntity>,
        (_, ct, c): (Id, Arc<RwLock<Trans>>, Arc<RwLock<Camera>>),
        context: &Context,
        (_, builder, recreate_swapchain): &mut Draw,
        _: &EntityManager,
        _: &ComponentManager,
    ) -> anyhow::Result<()> {
        if let Some((_, _, instance)) = i.first() {
            let instance = instance.read().unwrap();
            let pipeline = {
                if *recreate_swapchain {
                    instance.recreate_pipeline(context)?;
                }

                let (_, _, pipeline) = &*instance.pipeline;

                pipeline.read().unwrap().clone()
            };

            builder.bind_pipeline_graphics(pipeline.clone())?;

            let c = c.read().unwrap();
            let ct = ct.read().unwrap();
            let instance_data = {
                let instance_data: Vec<_> = i
                    .iter()
                    .map(|(_, t, i)| {
                        let i = i.read().unwrap();
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
                    z: c.calculate_z(instance.layer),
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
                .bind_vertex_buffers(
                    0,
                    (instance.shape.vertices.clone(), instance_buffer.clone()),
                )?
                .draw(
                    instance.shape.vertices.len() as u32,
                    instance_buffer.len() as u32,
                    0,
                    0,
                )?;
        }

        Ok(())
    }
}
