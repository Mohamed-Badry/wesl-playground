use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformData {
    resolution: [f32; 2],
    mouse: [f32; 2],
    time: f32,
    _padding: [f32; 3], // Padding to make the size a multiple of 16 bytes
}

impl UniformData {
    pub fn new(height: f32, width: f32) -> Self {
        Self {
            resolution: [width, height],
            mouse: [width / 2.0, height / 2.0],
            time: 0.0,
            _padding: [0.0;3],
        }
    }

    pub fn update_resolution(&mut self, width: f32, height: f32) {
        self.resolution = [width, height];
    }

    pub fn update_mouse(&mut self, x: f32, y: f32) {
        self.mouse = [x, y];
    }

    pub fn update_time(&mut self, delta: f32) {
        self.time += delta;
    }

}

pub struct Uniforms {
    pub data: UniformData,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Uniforms {
    pub fn new(device: &wgpu::Device, height: f32, width: f32) -> Self {
        let data = UniformData::new(height, width);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            data,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.data.update_time(0.016);
        // dbg!("Updating uniforms: {:?}", self.data);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}