// W3C WebGPU standard types for strict TypeScript compilation

export type GPUBrand<K, T> = K & { readonly __brand: T };

export interface GPUObjectDescriptorBase {
  label?: string;
}

export interface GPUBufferDescriptor extends GPUObjectDescriptorBase {
  size: number;
  usage: number;
  mappedAtCreation?: boolean;
}

export interface GPUBuffer {
  destroy(): void;
}

export interface GPUQueue {
  writeBuffer(
    buffer: GPUBuffer,
    bufferOffset: number,
    data: BufferSource | SharedArrayBuffer,
    dataOffset?: number,
    size?: number,
  ): void;
  submit(commandBuffers: GPUCommandBuffer[]): void;
}

export interface GPURenderPassColorAttachment {
  view: GPUTextureView;
  resolveTarget?: GPUTextureView;
  clearValue?: { r: number; g: number; b: number; a: number };
  loadOp: 'load' | 'clear';
  storeOp: 'store' | 'discard';
}

export interface GPURenderPassDescriptor extends GPUObjectDescriptorBase {
  colorAttachments: GPURenderPassColorAttachment[];
}

export interface GPURenderPassEncoder {
  setPipeline(pipeline: GPURenderPipeline): void;
  setBindGroup(index: number, bindGroup: GPUBindGroup): void;
  draw(
    vertexCount: number,
    instanceCount?: number,
    firstVertex?: number,
    firstInstance?: number,
  ): void;
  end(): void;
}

export type GPUCommandBuffer = GPUBrand<object, 'GPUCommandBuffer'>;
export type GPUTextureView = GPUBrand<object, 'GPUTextureView'>;

export interface GPUTexture {
  createView(): GPUTextureView;
}

export interface GPUCanvasContext {
  configure(config: { device: GPUDevice; format: string; alphaMode?: string }): void;
  getCurrentTexture(): GPUTexture;
}

export type GPUBindGroup = GPUBrand<object, 'GPUBindGroup'>;
export type GPUBindGroupLayout = GPUBrand<object, 'GPUBindGroupLayout'>;

export interface GPURenderPipeline {
  getBindGroupLayout(index: number): GPUBindGroupLayout;
}

export type GPUShaderModule = GPUBrand<object, 'GPUShaderModule'>;

export interface GPUDevice {
  queue: GPUQueue;
  createBuffer(descriptor: GPUBufferDescriptor): GPUBuffer;
  createShaderModule(descriptor: { code: string }): GPUShaderModule;
  createRenderPipeline(descriptor: Record<string, unknown>): GPURenderPipeline;
  createBindGroup(descriptor: Record<string, unknown>): GPUBindGroup;
  createCommandEncoder(): GPUCommandEncoder;
  destroy(): void;
}

export interface GPUCommandEncoder {
  beginRenderPass(descriptor: GPURenderPassDescriptor): GPURenderPassEncoder;
  finish(): GPUCommandBuffer;
}

export interface GPUAdapter {
  requestDevice(descriptor?: Record<string, unknown>): Promise<GPUDevice>;
}

export interface GPU {
  requestAdapter(options?: { powerPreference?: string }): Promise<GPUAdapter | null>;
  getPreferredCanvasFormat(): string;
}

declare global {
  interface Navigator {
    gpu?: GPU;
  }

  const GPUBufferUsage: {
    readonly MAP_READ: number;
    readonly MAP_WRITE: number;
    readonly COPY_SRC: number;
    readonly COPY_DST: number;
    readonly INDEX: number;
    readonly VERTEX: number;
    readonly UNIFORM: number;
    readonly STORAGE: number;
    readonly INDIRECT: number;
    readonly QUERY_RESOLVE: number;
  };
}
