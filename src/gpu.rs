extern crate winapi;

use crate::dx12;
use crate::glyphs::load_raw_glyphs;
use crate::scene;
use crate::window;
use std::path::{Path, PathBuf};
use std::{mem, ptr};
use winapi::shared::{dxgi, dxgi1_2, dxgiformat, dxgitype, minwindef, winerror};
use winapi::um::{d3d12, d3dcommon};
use winapi::Interface;

const FRAME_COUNT: u32 = 2;
pub type VertexCoordinates = [f32; 3];
pub type VertexColor = [f32; 4];
pub type Vertex = VertexCoordinates;

unsafe fn store_u32_in_256_bytes(x: u32) -> [u8; 256] {
    let mut result: [u8; 256] = [0; 256];
    let x_in_bytes: [u8; 4] = mem::transmute(x);

    for n in 0..4 {
        result[n] = x_in_bytes[n];
    }

    result
}

pub struct Quad {
    ox: f32,
    oy: f32,
    width: f32,
    height: f32,
}

impl Quad {
    fn new(ox: f32, oy: f32, width: f32, height: f32) -> Quad {
        Quad {
            ox,
            oy,
            width,
            height,
        }
    }

    fn as_vertices(&self) -> [Vertex; 4] {
        [
            [self.ox, self.oy, 0.0],
            [self.ox, self.oy + self.height, 0.0],
            [self.ox + self.width, self.oy, 0.0],
            [self.ox + self.width, self.oy + self.height, 0.0],
        ]
    }
}

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

fn materialize_per_tile_command_list_kernel_code(
    ptcl_num_tiles_per_tg_x: u32,
    ptcl_num_tiles_per_tg_y: u32,
    shader_template_path: &Path,
    shader_path: &Path,
) {
    let step0 = std::fs::read_to_string(shader_template_path)
        .expect("could not read data from provided shader template path");

    let step1 = step0.replace("~PTCL_X~", &format!("{}", ptcl_num_tiles_per_tg_x));
    let step2 = step1.replace("~PTCL_Y~", &format!("{}", ptcl_num_tiles_per_tg_y));

    std::fs::write(shader_path, step2).expect("shader template could not be materialized");
}

fn materialize_paint_kernel_code(
    paint_num_pixels_per_tg_x: u32,
    paint_num_pixels_per_tg_y: u32,
    shader_template_path: &Path,
    shader_path: &Path,
) {
    let step0 = std::fs::read_to_string(shader_template_path)
        .expect("could not read data from provided shader template path");

    let step1 = step0.replace("~P_X~", &format!("{}", paint_num_pixels_per_tg_x));
    let step2 = step1.replace("~P_Y~", &format!("{}", paint_num_pixels_per_tg_y));

    std::fs::write(shader_path, step2).expect("shader template could not be materialized");
}

enum TimingQueryPoints {
    BeginCmd,
    PtclInitComplete,
    PtclDispatch,
    PtclBufferSync,
    PaintInitComplete,
    PaintDispatch,
    CanvasBufferSync,
    DrawInitComplete,
    Draw,
    EndCmd,
    Count,
}

struct TimingData {
    begin_cmd_tps: Vec<f64>,
    ptcl_init_complete_ts: Vec<f64>,
    ptcl_dispatch_ts: Vec<f64>,
    ptcl_buf_sync_ts: Vec<f64>,
    paint_init_complete_ts: Vec<f64>,
    paint_dispatch_ts: Vec<f64>,
    canvas_buf_sync_ts: Vec<f64>,
    draw_init_complete_ts: Vec<f64>,
    draw_ts: Vec<f64>,
    end_cmd_ts: Vec<f64>,
}

fn interpret_timing_data_in_ms(
    num_renders: usize,
    tick_period_in_seconds: f64,
    raw_timing_data: Vec<u64>,
) -> TimingData {
    let tick0 = raw_timing_data[0];
    let tick_period_in_ms = tick_period_in_seconds * 1000.0;
    let timing_data_in_ms = raw_timing_data
        .iter()
        .map(|ticks| (*ticks as f64) * tick_period_in_ms)
        .collect::<Vec<f64>>();

    let mut begin_cmd_tps = Vec::<f64>::new();
    let mut ptcl_init_complete_ts = Vec::<f64>::new();
    let mut ptcl_dispatch_ts = Vec::<f64>::new();
    let mut ptcl_buf_sync_ts = Vec::<f64>::new();
    let mut paint_init_complete_ts = Vec::<f64>::new();
    let mut paint_dispatch_ts = Vec::<f64>::new();
    let mut canvas_buf_sync_ts = Vec::<f64>::new();
    let mut draw_init_complete_ts = Vec::<f64>::new();
    let mut draw_ts = Vec::<f64>::new();
    let mut end_cmd_ts = Vec::<f64>::new();

    let tp_count = TimingQueryPoints::Count as usize;
    let ptcl_init_complete_offset = TimingQueryPoints::PtclInitComplete as usize;
    let ptcl_dispatch_offset = TimingQueryPoints::PtclDispatch as usize;
    let ptcl_buf_sync_offset = TimingQueryPoints::PtclBufferSync as usize;
    let paint_init_complete_offset = TimingQueryPoints::PaintInitComplete as usize;
    let paint_dispatch_offset = TimingQueryPoints::PaintDispatch as usize;
    let canvas_buf_sync_offset = TimingQueryPoints::CanvasBufferSync as usize;
    let draw_init_complete_offset = TimingQueryPoints::DrawInitComplete as usize;
    let draw_offset = TimingQueryPoints::Draw as usize;
    let end_cmd_offset = TimingQueryPoints::EndCmd as usize;

    for i in 0..num_renders {
        let ix = i * tp_count;

        let (
            begin_cmd_tp,
            ptcl_init_complete_tp,
            ptcl_dispatch_tp,
            ptcl_buf_sync_tp,
            paint_init_complete_tp,
            paint_dispatch_tp,
            canvas_buf_sync_tp,
            draw_init_complete_tp,
            draw_tp,
            end_cmd_tp,
        ) = (
            timing_data_in_ms[ix],
            timing_data_in_ms[ix + ptcl_init_complete_offset],
            timing_data_in_ms[ix + ptcl_dispatch_offset],
            timing_data_in_ms[ix + ptcl_buf_sync_offset],
            timing_data_in_ms[ix + paint_init_complete_offset],
            timing_data_in_ms[ix + paint_dispatch_offset],
            timing_data_in_ms[ix + canvas_buf_sync_offset],
            timing_data_in_ms[ix + draw_init_complete_offset],
            timing_data_in_ms[ix + draw_offset],
            timing_data_in_ms[ix + end_cmd_offset],
        );

        begin_cmd_tps.push(begin_cmd_tp);
        ptcl_init_complete_ts.push(ptcl_init_complete_tp - begin_cmd_tp);
        ptcl_dispatch_ts.push(ptcl_dispatch_tp - ptcl_init_complete_tp);
        ptcl_buf_sync_ts.push(ptcl_buf_sync_tp - ptcl_dispatch_tp);
        paint_init_complete_ts.push(paint_init_complete_tp - ptcl_buf_sync_tp);
        paint_dispatch_ts.push(paint_dispatch_tp - paint_init_complete_tp);
        canvas_buf_sync_ts.push(canvas_buf_sync_tp - paint_dispatch_tp);
        draw_init_complete_ts.push(draw_init_complete_tp - canvas_buf_sync_tp);
        draw_ts.push(draw_tp - draw_init_complete_tp);
        end_cmd_ts.push(end_cmd_tp - draw_tp);
    }

    TimingData {
        begin_cmd_tps,
        ptcl_init_complete_ts,
        ptcl_dispatch_ts,
        ptcl_buf_sync_ts,
        paint_init_complete_ts,
        paint_dispatch_ts,
        canvas_buf_sync_ts,
        draw_init_complete_ts,
        draw_ts,
        end_cmd_ts,
    }
}

fn average_f64s(count: usize, input_data: &[f64]) -> f64 {
    let mut sum: f64 = 0.0;
    let num_elements: f64 = count as f64;

    for i in 0..count {
        sum += input_data[i];
    }

    return sum / num_elements;
}

pub struct GpuState {
    width: u32,
    height: u32,

    // pipeline stuff
    device: dx12::Device,
    command_allocators: Vec<dx12::CommandAllocator>,
    command_queue: dx12::CommandQueue,
    command_list: dx12::GraphicsCommandList,

    viewport: d3d12::D3D12_VIEWPORT,
    scissor_rect: d3d12::D3D12_RECT,
    swapchain: dx12::SwapChain3,
    vertex_buffer: dx12::Resource,
    vertex_buffer_view: d3d12::D3D12_VERTEX_BUFFER_VIEW,
    rtv_descriptor_heap: dx12::DescriptorHeap,
    render_targets: Vec<dx12::Resource>,
    graphics_root_signature: dx12::RootSignature,
    graphics_pipeline_state: dx12::PipelineState,

    num_tiles_x: u32,
    num_tiles_y: u32,
    num_ptcl_tg_x: u32,
    num_ptcl_tg_y: u32,

    compute_descriptor_heap: dx12::DescriptorHeap,
    constants_buffer: dx12::Resource,
    object_data_buffer: dx12::Resource,
    per_tile_command_lists_buffer: dx12::Resource,
    glyph_textures: Vec<d12::Resource>,
    canvas_texture: dx12::Resource,
    per_tile_command_lists_root_signature: dx12::RootSignature,
    paint_root_signature: dx12::RootSignature,
    per_tile_command_lists_pipeline_state: dx12::PipelineState,
    paint_pipeline_state: dx12::PipelineState,

    // synchronizers
    frame_index: usize,
    fence_event: dx12::Event,
    fence: dx12::Fence,
    fence_values: Vec<u64>,

    query_heap: dx12::QueryHeap,
    timing_query_buffer: dx12::Resource,
    num_renders: u32,
}

impl GpuState {
    pub unsafe fn new(
        wnd: &window::Window,
        per_tile_command_lists_entry: String,
        paint_entry: String,
        vertex_entry: String,
        fragment_entry: String,
        tile_side_length_in_pixels: u32,
        per_tile_command_lists_num_tiles_per_tg_x: u32,
        per_tile_command_lists_num_tiles_per_tg_y: u32,
        paint_num_tiles_per_tg_x: u32,
        paint_num_tiles_per_tg_y: u32,
        num_renders: u32,
    ) -> GpuState {
        let raw_glyphs = crate::glyphs::load_raw_glyphs();
        let width = wnd.get_width();
        let height = wnd.get_height();
        let num_objects = 1000;
        let (object_size, object_data) =
            scene::create_random_scene(width, height, num_objects, &raw_glyphs);
        //        let num_objects = 1;
        //        let (bbox_data, color_data) = scene::create_constant_scene();

        let f_tile_side_length_in_pixels = tile_side_length_in_pixels as f32;
        let f_width = width as f32;
        let f_height = height as f32;
        let canvas_quad_width =
            (f_width / f_tile_side_length_in_pixels).ceil() * f_tile_side_length_in_pixels;
        let canvas_quad_height =
            (f_height / f_tile_side_length_in_pixels).ceil() * f_tile_side_length_in_pixels;
        let num_tiles_x = {
            let min_ntx = (canvas_quad_width / f_tile_side_length_in_pixels) as u32;
            let remainder = min_ntx % per_tile_command_lists_num_tiles_per_tg_x;

            if remainder == 0 {
                min_ntx
            } else {
                min_ntx + (per_tile_command_lists_num_tiles_per_tg_x - remainder)
            }
        };
        let num_tiles_y = {
            let min_nty = (canvas_quad_height / f_tile_side_length_in_pixels) as u32;
            let remainder = min_nty % per_tile_command_lists_num_tiles_per_tg_y;

            if remainder == 0 {
                min_nty
            } else {
                min_nty + (per_tile_command_lists_num_tiles_per_tg_y - remainder)
            }
        };
        let canvas_quad_width = (num_tiles_x * tile_side_length_in_pixels) as f32;
        let canvas_quad_height = (num_tiles_y * tile_side_length_in_pixels) as f32;
        let num_ptcl_tg_x = num_tiles_x / per_tile_command_lists_num_tiles_per_tg_x;
        let num_ptcl_tg_y = num_tiles_y / per_tile_command_lists_num_tiles_per_tg_y;
        let paint_num_pixels_per_tg_x = paint_num_tiles_per_tg_x * tile_side_length_in_pixels;
        let paint_num_pixels_per_tg_y = paint_num_tiles_per_tg_y * tile_side_length_in_pixels;

        let canvas_quad = Quad::new(
            -1.0 * (canvas_quad_width / 2.0),
            -1.0 * (canvas_quad_height / 2.0),
            canvas_quad_width,
            canvas_quad_height,
        );

        let shader_folder = Path::new("shaders");

        let ptcl_kernel_template_path = shader_folder.join(Path::new("ptcl_kernel_template.hlsl"));
        let ptcl_kernel_path = shader_folder.join(Path::new("ptcl_kernel.hlsl"));
        materialize_per_tile_command_list_kernel_code(
            per_tile_command_lists_num_tiles_per_tg_x,
            per_tile_command_lists_num_tiles_per_tg_y,
            &ptcl_kernel_template_path,
            &ptcl_kernel_path,
        );

        let paint_kernel_template_path =
            shader_folder.join(Path::new("paint_kernel_template.hlsl"));
        let paint_kernel_path = shader_folder.join(Path::new("paint_kernel.hlsl"));
        materialize_paint_kernel_code(
            paint_num_pixels_per_tg_x,
            paint_num_pixels_per_tg_y,
            &paint_kernel_template_path,
            &paint_kernel_path,
        );

        let vertex_shader_path = shader_folder.join(Path::new("vertex_shader.hlsl"));
        let fragment_shader_path = shader_folder.join(Path::new("fragment_shader.hlsl"));

        println!("width: {}", width);
        println!("height: {}", height);
        println!("tile_side_length_in_pixels: {}", tile_side_length_in_pixels);
        println!(
            "per_tile_command_lists_num_tiles_per_tg_x: {}",
            per_tile_command_lists_num_tiles_per_tg_x
        );
        println!(
            "per_tile_command_lists_num_tiles_per_tg_y: {}",
            per_tile_command_lists_num_tiles_per_tg_y
        );
        println!("num_tiles_x: {}", num_tiles_x);
        println!("num_tiles_y: {}", num_tiles_y);
        println!("canvas_quad_width: {}", canvas_quad_width);
        println!("canvas_quad_height: {}", canvas_quad_height);
        println!("slop_x: {}", (canvas_quad_width / (width as f32)) - 1.0);
        println!("slop_y: {}", (canvas_quad_height / (height as f32)) - 1.0);
        println!("num_ptcl_tg_x: {}", num_ptcl_tg_x);
        println!("num_ptcl_tg_x: {}", num_ptcl_tg_y);
        //panic!("stop");

        let viewport = d3d12::D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0 as f32,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 0.0,
        };

        let scissor_rect = d3d12::D3D12_RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };

        #[cfg(debug_assertions)]
        dx12::enable_debug_layer();

        let (factory4, device, command_allocators, command_queue, fence, fence_event) =
            GpuState::create_shared_pipeline_dependencies();

        let (swapchain, rtv_descriptor_heap, render_targets) =
            GpuState::create_graphics_pipeline_dependencies(
                device.clone(),
                width,
                height,
                wnd,
                factory4,
                command_queue.clone(),
            );

        let (
            compute_descriptor_heap,
            constants_buffer,
            object_data_buffer,
            per_tile_command_lists_buffer,
            glyph_textures,
            canvas_texture,
        ) = GpuState::create_compute_pipeline_dependencies(
            device.clone(),
            width,
            height,
            num_objects,
            object_size,
            num_tiles_x,
            num_tiles_y,
            tile_side_length_in_pixels,
            object_data,
            raw_glyphs,
        );

        let (
            per_tile_command_lists_root_signature,
            per_tile_command_lists_pipeline_state,
            paint_root_signature,
            paint_pipeline_state,
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
            command_list,
        ) = GpuState::create_pipeline_states(
            device.clone(),
            &ptcl_kernel_path,
            &paint_kernel_path,
            &vertex_shader_path,
            &fragment_shader_path,
            per_tile_command_lists_entry,
            paint_entry,
            vertex_entry,
            fragment_entry,
            &command_allocators,
            canvas_quad,
        );

        let query_heap = device.create_query_heap(
            d3d12::D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
            num_renders * (TimingQueryPoints::Count as u32),
        );
        let timing_query_buffer = GpuState::create_timing_query_buffer(device.clone(), num_renders);

        let mut gpu_state = GpuState {
            width,
            height,

            // pipeline stuff
            device,
            command_allocators,
            command_queue,
            command_list,

            viewport,
            scissor_rect,
            swapchain,
            vertex_buffer,
            vertex_buffer_view,
            rtv_descriptor_heap,
            render_targets,
            graphics_root_signature,
            graphics_pipeline_state,

            num_tiles_x,
            num_tiles_y,
            num_ptcl_tg_x,
            num_ptcl_tg_y,

            compute_descriptor_heap,
            constants_buffer,
            object_data_buffer,
            per_tile_command_lists_buffer,
            glyph_textures,
            canvas_texture,
            per_tile_command_lists_root_signature,
            paint_root_signature,
            per_tile_command_lists_pipeline_state,
            paint_pipeline_state,

            // synchronizers
            frame_index: 0,
            fence_event,
            fence,
            fence_values: (0..FRAME_COUNT).into_iter().map(|_| 1).collect(),

            query_heap,
            timing_query_buffer,
            num_renders,
        };

        // wait for upload of any resources to gpu
        gpu_state.wait_for_gpu();

        gpu_state
    }

    unsafe fn populate_command_list(&mut self, render_index: u32) {
        let offset = render_index * (TimingQueryPoints::Count as u32);

        self.command_allocators[self.frame_index].reset();

        // per tile command list generation call
        self.command_list.reset(
            self.command_allocators[self.frame_index].clone(),
            self.per_tile_command_lists_pipeline_state.clone(),
        );

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::BeginCmd as u32 + offset,
        );

        self.command_list
            .set_compute_root_signature(self.per_tile_command_lists_root_signature.clone());
        self.command_list
            .set_descriptor_heaps(vec![self.compute_descriptor_heap.clone()]);
        self.command_list.set_compute_root_descriptor_table(
            0,
            self.compute_descriptor_heap
                .get_gpu_descriptor_handle_at_offset(0),
        );

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::PtclInitComplete as u32 + offset,
        );

        //panic!("stop");

        self.command_list
            .dispatch(self.num_ptcl_tg_x, self.num_ptcl_tg_y, 1);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::PtclDispatch as u32 + offset,
        );

        // need to ensure all writes to per_tile_command_lists are complete before any reads are done
        let synchronize_wrt_per_tile_command_lists =
            dx12::create_uav_resource_barrier(self.per_tile_command_lists_buffer.0.as_raw());
        self.command_list
            .set_resource_barrier(vec![synchronize_wrt_per_tile_command_lists]);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::PtclBufferSync as u32 + offset,
        );

        // paint call
        self.command_list
            .set_pipeline_state(self.paint_pipeline_state.clone());
        self.command_list
            .set_compute_root_signature(self.paint_root_signature.clone());
        self.command_list
            .set_descriptor_heaps(vec![self.compute_descriptor_heap.clone()]);
        self.command_list.set_compute_root_descriptor_table(
            0,
            self.compute_descriptor_heap
                .get_gpu_descriptor_handle_at_offset(0),
        );

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::PaintInitComplete as u32 + offset,
        );

        self.command_list
            .dispatch(self.num_tiles_x, self.num_tiles_y, 1);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::PaintDispatch as u32 + offset,
        );

        // need to ensure all writes to intermediate are complete before any reads are done
        let synchronize_wrt_canvas =
            dx12::create_uav_resource_barrier(self.canvas_texture.0.as_raw());
        self.command_list
            .set_resource_barrier(vec![synchronize_wrt_canvas]);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::CanvasBufferSync as u32 + offset,
        );

        // graphics pipeline call
        self.command_list
            .set_pipeline_state(self.graphics_pipeline_state.clone());
        self.command_list
            .set_graphics_root_signature(self.graphics_root_signature.clone());
        self.command_list
            .set_descriptor_heaps(vec![self.compute_descriptor_heap.clone()]);
        self.command_list.set_graphics_root_descriptor_table(
            0,
            self.compute_descriptor_heap
                .get_gpu_descriptor_handle_at_offset(4),
        );
        self.command_list.set_viewport(&self.viewport);
        self.command_list.set_scissor_rect(&self.scissor_rect);

        let transition_render_target_from_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
        );
        self.command_list
            .set_resource_barrier(vec![transition_render_target_from_present]);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::DrawInitComplete as u32 + offset,
        );

        let mut rt_descriptor = self
            .rtv_descriptor_heap
            .get_cpu_descriptor_handle_at_offset(self.frame_index as u32);
        self.command_list.set_render_target(rt_descriptor);

        // Record drawing commands.
        self.command_list
            .clear_render_target_view(rt_descriptor, &CLEAR_COLOR);
        self.command_list
            .set_primitive_topology(d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
        self.command_list
            .set_vertex_buffer(0, 1, &self.vertex_buffer_view);
        self.command_list.draw_instanced(4, 1, 0, 0);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::Draw as u32 + offset,
        );

        let transition_render_target_to_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
        );
        self.command_list
            .set_resource_barrier(vec![transition_render_target_to_present]);

        self.command_list.end_timing_query(
            self.query_heap.clone(),
            TimingQueryPoints::EndCmd as u32 + offset,
        );

        if (render_index == (self.num_renders - 1)) {
            self.command_list.resolve_timing_query_data(
                self.query_heap.clone(),
                0,
                self.num_renders * (TimingQueryPoints::Count as u32),
                self.timing_query_buffer.clone(),
                0,
            );
            //            let transition_timing_query_heap_to_read = dx12::create_transition_resource_barrier(
            //                self.timing_query_buffer.0.as_raw(),
            //                d3d12::D3D12_RESOURCE_STATE_COPY_DEST,
            //                d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            //            );
            //            self.command_list
            //                .set_resource_barrier(vec![transition_timing_query_heap_to_read]);
        }

        self.command_list.close();
    }

    unsafe fn execute_command_list(&mut self) {
        self.command_queue
            .execute_command_lists(1, &[self.command_list.as_raw_list()]);
    }

    pub unsafe fn render(&mut self, render_index: u32) {
        self.populate_command_list(render_index);

        self.execute_command_list();

        // TODO: what should the present flags be?
        self.swapchain.present(1, 0);

        self.move_to_next_frame();
    }

    unsafe fn wait_for_gpu(&mut self) {
        self.command_queue
            .signal(self.fence.clone(), self.fence_values[self.frame_index]);

        //TODO: handle return value
        self.fence.set_event_on_completion(
            self.fence_event.clone(),
            self.fence_values[self.frame_index],
        );
        self.fence_event
            .wait_ex(winapi::um::winbase::INFINITE, false);

        self.fence_values[self.frame_index] = self.fence_values[self.frame_index] + 1;

        self.fence_values[self.frame_index] = self.fence_values[self.frame_index] + 1;
    }

    unsafe fn move_to_next_frame(&mut self) {
        let current_fence_value = self.fence_values[self.frame_index];
        self.command_queue
            .signal(self.fence.clone(), current_fence_value);

        self.frame_index = self.swapchain.get_current_back_buffer_index() as usize;

        if self.fence.get_value() < self.fence_values[self.frame_index] {
            self.fence.set_event_on_completion(
                self.fence_event.clone(),
                self.fence_values[self.frame_index],
            );
            //TODO: handle return value
            self.fence_event
                .wait_ex(winapi::um::winbase::INFINITE, false);
        }

        self.fence_values[self.frame_index] = current_fence_value + 1;
    }

    pub unsafe fn destroy(&mut self) {
        self.wait_for_gpu();

        if winapi::um::handleapi::CloseHandle(self.fence_event.0) == 0 {
            panic!("could not close fence event properly")
        }
    }

    unsafe fn create_shared_pipeline_dependencies() -> (
        dx12::Factory4,
        dx12::Device,
        Vec<dx12::CommandAllocator>,
        dx12::CommandQueue,
        dx12::Fence,
        dx12::Event,
    ) {
        // create factory4
        let factory4 = dx12::Factory4::create(0);

        // create device
        let device = match dx12::Device::create_device(&factory4) {
            Ok(device) => device,
            Err(hr) => {
                if hr[0] == winerror::DXGI_ERROR_NOT_FOUND {
                    panic!("could not find adapter");
                } else {
                    panic!("could not find dx12 capable device");
                }
            }
        };

        let list_type = d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT;
        let command_queue =
            device.create_command_queue(list_type, 0, d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE, 0);

        let mut command_allocators: Vec<dx12::CommandAllocator> = (0..FRAME_COUNT)
            .into_iter()
            .map(|_| device.create_command_allocator(list_type))
            .collect();

        let fence = device.create_fence(0);
        let fence_event = dx12::Event::create(false, false);

        (
            factory4,
            device,
            command_allocators,
            command_queue,
            fence,
            fence_event,
        )
    }

    unsafe fn create_graphics_pipeline_dependencies(
        device: dx12::Device,
        width: u32,
        height: u32,
        wnd: &window::Window,
        factory4: dx12::Factory4,
        command_queue: dx12::CommandQueue,
    ) -> (dx12::SwapChain3, dx12::DescriptorHeap, Vec<dx12::Resource>) {
        // create swapchain
        let swapchain_desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_IGNORE,
            BufferCount: FRAME_COUNT,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Flags: 0,
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: dxgi1_2::DXGI_SCALING_STRETCH,
            Stereo: winapi::shared::minwindef::FALSE,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
        };

        // disable full screen transitions
        // winapi does not have DXGI_MWA_NO_ALT_ENTER?
        factory4.0.MakeWindowAssociation(wnd.hwnd, 1);

        let swap_chain3 = factory4.create_swapchain_for_hwnd(
            command_queue.clone(),
            wnd.hwnd.clone(),
            swapchain_desc,
        );

        // create graphics descriptor heap
        let rtv_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: FRAME_COUNT,
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_descriptor_heap = device.create_descriptor_heap(&rtv_descriptor_heap_desc);

        let mut render_targets: Vec<dx12::Resource> = Vec::new();
        for ix in 0..FRAME_COUNT {
            let render_target_resource = swap_chain3.get_buffer(ix);

            device.create_render_target_view(
                render_target_resource.clone(),
                ptr::null(),
                rtv_descriptor_heap.get_cpu_descriptor_handle_at_offset(ix),
            );

            render_targets.push(render_target_resource.clone());
        }

        (swap_chain3, rtv_descriptor_heap, render_targets)
    }

    unsafe fn create_constants_buffer(
        constants: &[u32],
        device: dx12::Device,
        descriptor_heap: dx12::DescriptorHeap,
        descriptor_index: u32,
    ) -> dx12::Resource {
        let constants = [
            num_objects,
            object_size,
            tile_side_length_in_pixels,
            num_tiles_x,
            num_tiles_y,
        ];
        let constant_buffer_stride = mem::size_of::<u32>();
        let constant_buffer_size = constant_buffer_stride * constants.len();
        // https://github.com/microsoft/DirectX-Graphics-Samples/blob/cce992eb853e7cfd6235a10d23d58a8f2334aad5/Samples/Desktop/D3D12HelloWorld/src/HelloConstBuffers/D3D12HelloConstBuffers.cpp#L284
        let padded_size_in_bytes: u64 = 256;
        let constants_buffer_resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: padded_size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };
        let constants_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let constants_buffer = device.create_committed_resource(
            &constants_buffer_heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &constants_buffer_resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
        );
        constants_buffer.upload_data_to_resource(256, constants.as_ptr());
        device.create_constant_buffer_view(
            constants_buffer.clone(),
            descriptor_heap.get_cpu_descriptor_handle_at_offset(descriptor_index),
            padded_size_in_bytes as u32,
        );

        constants_buffer
    }

    unsafe fn create_object_data_buffer(
        object_data: Vec<u8>,
        device: dx12::Device,
        descriptor_heap: dx12::DescriptorHeap,
        descriptor_index: u32,
    ) -> dx12::Resource {
        let object_data_buffer_size_in_bytes = object_data.len();
        let object_data_buffer_size_in_u32s =
            (object_data_buffer_size_in_bytes / mem::size_of::<u32>()) as u32;
        let object_data_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let object_data_buffer_resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: object_data_buffer_size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };
        let object_data_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let object_data_buffer = device.create_committed_resource(
            &object_data_buffer_heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &object_data_buffer_resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
        );
        object_data_buffer.upload_data_to_resource(object_data.len(), object_data.as_ptr());
        device.create_byte_addressed_buffer_shader_resource_view(
            object_data_buffer.clone(),
            descriptor_heap.get_cpu_descriptor_handle_at_offset(descriptor_index),
            0,
            object_data_buffer_size_in_u32s,
        );

        object_data_buffer
    }

    unsafe fn create_per_tile_command_lists_buffer(
        device: dx12::Device,
        descriptor_heap: dx12::DescriptorHeap,
        descriptor_index: u32,
    ) -> dx12::Resource {
        //TODO: consider flag D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS?
        let per_tile_command_list_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let per_tile_command_list_buffer_size_in_u32s = num_objects * num_tiles_x * num_tiles_y;
        let per_tile_command_list_buffer_size =
            (mem::size_of::<u32>() as u64) * (per_tile_command_list_buffer_size_in_u32s as u64);
        assert!(
            per_tile_command_list_buffer_size < (std::u32::MAX as u64),
            "per_tile_command_list_buffer_size >= std::u32::MAX!"
        );
        let per_tile_command_list_resource_desc = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: per_tile_command_list_buffer_size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            //essentially we're letting the adapter decide the layout
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            ..mem::zeroed()
        };
        let mut per_tile_command_lists = device.create_committed_resource(
            &per_tile_command_list_heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &per_tile_command_list_resource_desc,
            d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ptr::null(),
        );
        println!("creating per tile command lists buffer...");
        //TODO: if per_tile_command_list_buffer_size > std::u32::MAX, then we need to have more views, with first element being std::u32::MAX?
        device.create_byte_addressed_buffer_unordered_access_view(
            per_tile_command_lists.clone(),
            descriptor_heap.get_cpu_descriptor_handle_at_offset(descriptor_index),
            0,
            per_tile_command_list_buffer_size_in_u32s,
        );

        per_tile_command_lists
    }

    unsafe fn create_glyph_texture(
        raw_glyph: crate::glyphs::RawGlyph,
        device: dx12::Device,
        descriptor_heap: dx12::DescriptorHeap,
        descriptor_index: u32,
    ) -> dx12::Resource {
        let glyph_size_in_bytes = raw_glyph.bytes.len();
        let glyph_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let glyph_resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: raw_glyph.width as u64,
            Height: raw_glyph.height as u32,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };
        let glyph_texture = device.create_committed_resource(
            &object_data_buffer_heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &object_data_buffer_resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
        );
        glyph_texture.upload_data_to_resource(raw_glyph.bytes.len(), raw_glyph.bytes.as_ptr());
        device.create_texture2D_shader_resource_view(
            glyph_texture.clone(),
            dxgiformat::DXGI_FORMAT_R8_UNORM,
            compute_descriptor_heap.get_cpu_descriptor_handle_at_offset(descriptor_index),
        );

        glyph_texture
    }

    unsafe fn create_canvas_texture(
        device: dx12::Device,
        descriptor_heap: dx12::DescriptorHeap,
        descriptor_index: u32,
    ) -> dx12::Resource {
        //TODO: consider flag D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS?
        let canvas_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let canvas_resource_desc = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            //TODO: what alignment should be chosen?
            Alignment: 0,
            Width: (num_tiles_x * tile_side_length_in_pixels) as u64,
            Height: num_tiles_y * tile_side_length_in_pixels,
            DepthOrArraySize: 1,
            //TODO: what should MipLevels be?
            MipLevels: 1,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            //essentially we're letting the adapter decide the layout
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: d3d12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        };
        let mut clear_value: d3d12::D3D12_CLEAR_VALUE = mem::zeroed();
        *clear_value.u.Color_mut() = [0.0, 0.0, 0.0, 0.0];
        let mut canvas_texture = device.create_committed_resource(
            &canvas_heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &canvas_resource_desc,
            d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ptr::null(),
        );
        device.create_unordered_access_view(
            canvas_texture.clone(),
            compute_descriptor_heap.get_cpu_descriptor_handle_at_offset(descriptor_index),
        );

        canvas_texture
    }

    unsafe fn create_compute_pipeline_dependencies(
        device: dx12::Device,
        width: u32,
        height: u32,
        num_objects: u32,
        object_size: u32,
        num_tiles_x: u32,
        num_tiles_y: u32,
        tile_side_length_in_pixels: u32,
        object_data: Vec<u8>,
        raw_glyphs: Vec<crate::glyphs::RawGlyph>,
    ) -> (
        dx12::DescriptorHeap,
        dx12::Resource,
        dx12::Resource,
        dx12::Resource,
        Vec<dx12::Resource>,
        dx12::Resource,
    ) {
        // create compute resource descriptor heap
        let compute_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: 4 + (raw_glyphs.len() as u32),
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };
        let compute_descriptor_heap = device.create_descriptor_heap(&compute_descriptor_heap_desc);
        let mut descriptor_index: u32 = 0;

        // create constants buffer
        let constants_buffer = {
            let constants = [
                num_objects,
                object_size,
                tile_side_length_in_pixels,
                num_tiles_x,
                num_tiles_y,
            ];
            GpuState::create_constants_buffer(
                &constants,
                device.clone(),
                compute_descriptor_heap.clone(),
                descriptor_index,
            )
        };
        descriptor_index += 1;

        // create object data buffer
        let object_data_buffer = GpuState::create_object_data_buffer(
            object_data,
            device.clone(),
            compute_descriptor_heap.clone(),
            descriptor_index,
        );
        descriptor_index += 1;

        // create per tile command list resource
        let ptcl_buffer = GpuState::create_per_tile_command_lists_buffer(
            device.clone(),
            compute_descriptor_heap.clone(),
            descriptor_index,
        );
        descriptor_index += 1;

        // create intermediate target resource
        let canvas_texture = GpuState::create_canvas_texture(
            device.clone(),
            compute_descriptor_heap.clone(),
            descriptor_index,
        );
        descriptor_index += 1;

        // create glyph textures
        let mut glyph_textures = Vec::<dx12::Resource>::new();
        for raw_glyph in raw_glyphs.into_iter() {
            let glyph_texture = GpuState::create_glyph_texture(
                raw_glyph,
                device.clone(),
                compute_descriptor_heap.clone(),
                descriptor_index,
            );
            glyph_textures.push(glyph_texture);
            descriptor_index += 1;
        }

        (
            compute_descriptor_heap,
            constants_buffer,
            object_data_buffer,
            ptcl_buffer,
            glyph_textures,
            canvas_texture,
        )
    }

    unsafe fn create_compute_pipeline_states(
        device: dx12::Device,
        ptcl_kernel_path: &Path,
        paint_kernel_path: &Path,
        shader_compile_flags: minwindef::DWORD,
        per_tile_command_lists_entry: String,
        paint_entry: String,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::RootSignature,
        dx12::PipelineState,
    ) {
        // descriptor_ranges
        let constants_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
            NumDescriptors: 1,
            OffsetInDescriptorsFromTableStart: 0,
            ..mem::zeroed()
        };
        let objects_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: 2,
            OffsetInDescriptorsFromTableStart: 1,
            ..mem::zeroed()
        };
        let per_tile_command_lists_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 1,
            OffsetInDescriptorsFromTableStart: 3,
            BaseShaderRegister: 0,
            ..mem::zeroed()
        };
        // intermediate target is in a separate descriptor range from per_tile_command_lists
        // thus OffsetInDescriptorsFromTableStart should be the same?
        let canvas_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 1,
            OffsetInDescriptorsFromTableStart: 4,
            BaseShaderRegister: 1,
            ..mem::zeroed()
        };

        let per_tile_command_lists_descriptor_ranges = [
            constants_descriptor_range,
            objects_descriptor_range,
            per_tile_command_lists_descriptor_range,
        ];
        let per_tile_command_lists_descriptor_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: per_tile_command_lists_descriptor_ranges.len() as u32,
            pDescriptorRanges: per_tile_command_lists_descriptor_ranges.as_ptr() as *const _,
        };
        let mut per_tile_command_lists_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_ALL,
            ..mem::zeroed()
        };
        *per_tile_command_lists_root_parameter
            .u
            .DescriptorTable_mut() = per_tile_command_lists_descriptor_table;
        let per_tile_command_lists_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 1,
            pParameters: &per_tile_command_lists_root_parameter as *const _,
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
        };
        let blob = dx12::RootSignature::serialize_description(
            &per_tile_command_lists_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let per_tile_command_lists_root_signature = device.create_root_signature(0, blob);

        let paint_descriptor_ranges = [
            constants_descriptor_range,
            objects_descriptor_range,
            per_tile_command_lists_descriptor_range,
            canvas_descriptor_range,
        ];
        let paint_descriptor_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: paint_descriptor_ranges.len() as u32,
            pDescriptorRanges: paint_descriptor_ranges.as_ptr() as *const _,
        };
        let mut paint_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_ALL,
            ..mem::zeroed()
        };
        *paint_root_parameter.u.DescriptorTable_mut() = paint_descriptor_table;
        let paint_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 1,
            pParameters: &paint_root_parameter as *const _,
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
        };
        let blob = dx12::RootSignature::serialize_description(
            &paint_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let paint_root_signature = device.create_root_signature(0, blob);

        // load compute shaders
        let compute_target = String::from("cs_5_1");

        println!("compiling per tile command lists shader...");
        let per_tile_command_lists_shader_blob = dx12::ShaderByteCode::compile_from_file(
            ptcl_kernel_path,
            compute_target.clone(),
            per_tile_command_lists_entry,
            shader_compile_flags,
        );
        let per_tile_command_lists_shader_bytecode =
            dx12::ShaderByteCode::from_blob(per_tile_command_lists_shader_blob);

        println!("compiling paint shader...");
        let paint_shader_blob = dx12::ShaderByteCode::compile_from_file(
            paint_kernel_path,
            compute_target.clone(),
            paint_entry,
            shader_compile_flags,
        );
        let paint_shader_bytecode = dx12::ShaderByteCode::from_blob(paint_shader_blob);

        // create compute pipeline states
        let per_tile_command_lists_ps_desc = d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: per_tile_command_lists_root_signature.0.as_raw(),
            CS: per_tile_command_lists_shader_bytecode.bytecode,
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let per_tile_command_lists_pipeline_state =
            device.create_compute_pipeline_state(&per_tile_command_lists_ps_desc);

        let paint_ps_desc = d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: paint_root_signature.0.as_raw(),
            CS: paint_shader_bytecode.bytecode,
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let paint_pipeline_state = device.create_compute_pipeline_state(&paint_ps_desc);

        (
            per_tile_command_lists_root_signature,
            per_tile_command_lists_pipeline_state,
            paint_root_signature,
            paint_pipeline_state,
        )
    }

    unsafe fn create_graphics_pipeline_state(
        device: dx12::Device,
        vertex_shader_path: &Path,
        fragment_shader_path: &Path,
        shader_compile_flags: minwindef::DWORD,
        vertex_entry: String,
        fragment_entry: String,
        canvas_quad: Quad,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::Resource,
        d3d12::D3D12_VERTEX_BUFFER_VIEW,
    ) {
        // create graphics root signature
        let frag_shader_uav_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 1,
            OffsetInDescriptorsFromTableStart: 0,
            BaseShaderRegister: 1,
            ..mem::zeroed()
        };
        let frag_shader_srv_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: 1,
            pDescriptorRanges: &frag_shader_uav_descriptor_range as *const _,
        };
        let mut graphics_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_PIXEL,
            ..mem::zeroed()
        };
        *graphics_root_parameter.u.DescriptorTable_mut() = frag_shader_srv_table;
        let graphics_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 1,
            pParameters: &graphics_root_parameter as *const _,
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };
        // serialize root signature description and create graphics root signature
        let blob = dx12::RootSignature::serialize_description(
            &graphics_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let graphics_root_signature = device.create_root_signature(0, blob);

        // create vertex buffer
        let vertices = canvas_quad.as_vertices();
        let vertex_buffer_stride = mem::size_of::<Vertex>();
        let vertex_buffer_size = vertex_buffer_stride * vertices.len();
        let vertex_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };

        let vertex_buffer_resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: vertex_buffer_size as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..mem::zeroed()
        };

        let vertex_buffer = device.create_committed_resource(
            &vertex_buffer_heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &vertex_buffer_resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
        );
        vertex_buffer.upload_data_to_resource(vertices.len(), vertices.as_ptr());
        let vertex_buffer_view = d3d12::D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: vertex_buffer.get_gpu_virtual_address(),
            SizeInBytes: vertex_buffer_size as u32,
            StrideInBytes: vertex_buffer_stride as u32,
        };

        // load graphics shaders from byte string
        println!("compiling vertex shader...");
        let vertex_shader_target = String::from("vs_5_1");
        let graphics_vertex_shader_blob = dx12::ShaderByteCode::compile_from_file(
            vertex_shader_path,
            vertex_shader_target,
            vertex_entry,
            shader_compile_flags,
        );
        let graphics_vertex_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_vertex_shader_blob);

        println!("compiling fragment shader...");
        let fragment_shader_target = String::from("ps_5_1");
        let graphics_fragment_shader_blob = dx12::ShaderByteCode::compile_from_file(
            fragment_shader_path,
            fragment_shader_target,
            fragment_entry,
            shader_compile_flags,
        );
        let graphics_fragment_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_fragment_shader_blob);

        // create graphics pipeline state
        let position_ied = dx12::InputElementDesc {
            semantic_name: String::from("POSITION"),
            semantic_index: 0,
            format: dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            input_slot: 0,
            aligned_byte_offset: 0,
            input_slot_class: d3d12::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            instance_data_step_rate: 0,
        };

        let ieds = [position_ied.as_winapi_struct()];

        let graphics_ps_desc = d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: graphics_root_signature.0.as_raw(),
            VS: graphics_vertex_shader_bytecode.bytecode,
            PS: graphics_fragment_shader_bytecode.bytecode,
            DS: dx12::ShaderByteCode::empty().bytecode,
            HS: dx12::ShaderByteCode::empty().bytecode,
            GS: dx12::ShaderByteCode::empty().bytecode,
            StreamOutput: d3d12::D3D12_STREAM_OUTPUT_DESC {
                pSODeclaration: ptr::null(),
                NumEntries: 0,
                pBufferStrides: ptr::null(),
                NumStrides: 0,
                RasterizedStream: 0,
            },
            //TODO: confirm do nothing blend desc is correct
            BlendState: dx12::default_blend_desc(),
            SampleMask: std::u32::MAX,
            // TODO: could ..mem::zeroed() work here?
            RasterizerState: d3d12::D3D12_RASTERIZER_DESC {
                FillMode: d3d12::D3D12_FILL_MODE_SOLID,
                CullMode: d3d12::D3D12_CULL_MODE_NONE,
                FrontCounterClockwise: true as _,
                DepthBias: 0,
                DepthBiasClamp: 0.0,
                SlopeScaledDepthBias: 0.0,
                DepthClipEnable: false as _,
                MultisampleEnable: false as _,
                AntialiasedLineEnable: false as _,
                ForcedSampleCount: 0,
                ConservativeRaster: d3d12::D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            },
            // TODO: could ..mem::zeroed() work here?
            DepthStencilState: d3d12::D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: false as _,
                DepthWriteMask: 0,
                DepthFunc: 0,
                StencilEnable: false as _,
                StencilReadMask: 0,
                StencilWriteMask: 0,
                FrontFace: d3d12::D3D12_DEPTH_STENCILOP_DESC { ..mem::zeroed() },
                BackFace: d3d12::D3D12_DEPTH_STENCILOP_DESC { ..mem::zeroed() },
            },
            InputLayout: d3d12::D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: ieds.as_ptr(),
                NumElements: ieds.len() as u32,
            },
            IBStripCutValue: 0,
            PrimitiveTopologyType: d3d12::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            RTVFormats: [
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
            ],
            DSVFormat: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let graphics_pipeline_state = device.create_graphics_pipeline_state(&graphics_ps_desc);

        (
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
        )
    }

    unsafe fn create_pipeline_states(
        device: dx12::Device,
        ptcl_kernel_path: &Path,
        paint_kernel_path: &Path,
        vertex_shader_path: &Path,
        fragment_shader_path: &Path,
        per_tile_command_lists_entry: String,
        paint_entry: String,
        vertex_entry: String,
        fragment_entry: String,
        command_allocators: &Vec<dx12::CommandAllocator>,
        screen_quad: Quad,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::Resource,
        d3d12::D3D12_VERTEX_BUFFER_VIEW,
        dx12::GraphicsCommandList,
    ) {
        #[cfg(not(debug_assertions))]
        let shader_compile_flags: minwindef::DWORD = 0;

        #[cfg(debug_assertions)]
        let shader_compile_flags: minwindef::DWORD = winapi::um::d3dcompiler::D3DCOMPILE_DEBUG
            | winapi::um::d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION;

        let (
            per_tile_command_lists_root_signature,
            per_tile_command_lists_pipeline_state,
            paint_root_signature,
            paint_pipeline_state,
        ) = GpuState::create_compute_pipeline_states(
            device.clone(),
            ptcl_kernel_path,
            paint_kernel_path,
            shader_compile_flags,
            per_tile_command_lists_entry,
            paint_entry,
        );

        let (graphics_root_signature, graphics_pipeline_state, vertex_buffer, vertex_buffer_view) =
            GpuState::create_graphics_pipeline_state(
                device.clone(),
                vertex_shader_path,
                fragment_shader_path,
                shader_compile_flags,
                vertex_entry,
                fragment_entry,
                screen_quad,
            );

        // create command list
        let command_list = device.create_graphics_command_list(
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
            command_allocators[0].clone(),
            per_tile_command_lists_pipeline_state.clone(),
            0,
        );

        command_list.close();

        (
            per_tile_command_lists_root_signature,
            per_tile_command_lists_pipeline_state,
            paint_root_signature,
            paint_pipeline_state,
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
            command_list,
        )
    }

    unsafe fn create_timing_query_buffer(
        device: dx12::Device,
        num_expected_results: u32,
    ) -> dx12::Resource {
        let size_in_bytes = mem::size_of::<u64>()
            * ((num_expected_results * (TimingQueryPoints::Count as u32)) as usize);
        let timing_query_buffer_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };
        let timing_query_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_READBACK,
            //TODO: what should CPUPageProperty flag be?
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let resource_buffer = device.create_committed_resource(
            &timing_query_buffer_heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &timing_query_buffer_description,
            d3d12::D3D12_RESOURCE_STATE_COPY_DEST,
            ptr::null(),
        );

        device.create_committed_resource(
            &timing_query_buffer_heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &timing_query_buffer_description,
            d3d12::D3D12_RESOURCE_STATE_COPY_DEST,
            ptr::null(),
        )
    }

    pub unsafe fn print_stats(&mut self) {
        self.wait_for_gpu();

        let raw_timing_data: Vec<u64> =
            self.timing_query_buffer.download_data_from_resource::<u64>(
                (self.num_renders * TimingQueryPoints::Count as u32) as usize,
            );
        let tick_period_in_seconds = 1.0 / (self.command_queue.get_timestamp_frequency() as f64);

        let num_timepoints = (TimingQueryPoints::Count as u32) as f64;
        let num_recorded_renders = (raw_timing_data.len() as f64) / num_timepoints;
        //assert_eq!(self.num_renders, num_recorded_renders);
        println!("num_recorded_renders: {}", num_recorded_renders);

        println!("num recorded renders: {}", num_recorded_renders);
        let timing_data = interpret_timing_data_in_ms(
            self.num_renders as usize,
            tick_period_in_seconds,
            raw_timing_data,
        );
        println!(
            "average ptcl dispatch time (ms): {}",
            average_f64s(
                timing_data.ptcl_dispatch_ts.len(),
                &timing_data.ptcl_dispatch_ts
            )
        );
        println!(
            "average paint dispatch time (ms): {}",
            average_f64s(
                timing_data.paint_dispatch_ts.len(),
                &timing_data.paint_dispatch_ts
            )
        );
        println!(
            "average draw time (ms): {}",
            average_f64s(timing_data.draw_ts.len(), &timing_data.draw_ts)
        );
    }
}
