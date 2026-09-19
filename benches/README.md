Collection of CPU benchmarks for `wgpu`.

These benchmarks are designed as a first line of defence against performance regressions and generally approximate the performance for users.

## Usage

```sh
# Run all benchmarks
cargo bench -p wgpu-benchmark
# Run a specific benchmarks that contains "filter" in its name
cargo bench -p wgpu-benchmark -- "filter"
```

Use `WGPU_BACKEND` and `WGPU_ADAPTER_NAME` to adjust which device the benchmarks use. [More info on env vars](../README.md#environment-variables).

## Comparing Against a Baseline

To compare the current benchmarks against a baseline, you can use the `--save-baseline` and `--baseline` flags.

For example, to compare v28 against trunk, you could run the following:

```sh
git checkout v28
# Run the baseline benchmarks
cargo bench -p wgpu-benchmark -- --save-baseline "v28"

git checkout trunk
# Run the current benchmarks
cargo bench -p wgpu-benchmark -- --baseline "v28"
```

The current benchmarking framework was added before v28, so comparisons only work after it was added. Before that the same commands will work, but comparison will be done using `criterion`.

## Integration with Profilers

The benchmarks can be run with a profiler to get more detailed information about where time is being spent.
Integrations are available for `tracy` and `superluminal`.

#### Tracy

Tracy is available prebuilt for Windows on [github](https://github.com/wolfpld/tracy/releases/latest/).

```sh
# Once this is running, you can connect to it with the Tracy Profiler
cargo bench -p wgpu-benchmark --features tracy,profiling/profile-with-tracy
```

#### Superluminal

Superluminal is a paid product for windows available [here](https://superluminal.eu/).

```sh
# This command will build the benchmarks, and display the path to the executable
cargo bench -p wgpu-benchmark --features profiling/profile-with-superluminal -- -h

# Have Superluminal run the following command (replacing with the path to the executable)
<path_to_exe> --bench "filter"
```

#### `perf` and others

You can follow the same pattern as above to run the benchmarks with other profilers.
For example, the command line tool `perf` can be used to profile the benchmarks.

```sh
# This command will build the benchmarks, and display the path to the executable
cargo bench -p wgpu-benchmark -- -h

# Run the benchmarks with perf
perf record <path_to_exe> --bench "filter"
```

## Benchmarks

#### `Texture Initialization`

Measures repeated binding of initialized texture views, including views covering only part of a
texture, attachment textures, and groups mixing attachment and non-attachment textures. Two warm-up
submissions precede timing. Encoding and queue submission are measured separately, excluding GPU
waits. An unsubmitted-work case measures repeated recording without initializing the textures, and
a creation case measures the cost of creating bind groups for existing texture views. Groups without
textures provide a control for the cost of checking an empty texture initialization list.
Each encoding iteration binds 1,000 groups ten times, for 10,000 dispatches, keeping the number of
live texture subresources bounded. The creation case creates 1,000 groups per iteration.
Binding entries, descriptors, and space for the resulting groups are allocated before creation
timing; destruction and GPU waits are excluded.
Two shared-resource cases cycle the groups over 16 texture pairs: attachment textures with
identical views, and mixed attachment/non-attachment pairs with overlapping two-mip views.

#### `Texture Init Render`

Measures attachment writes followed by sampling in ten render passes, with 1,000 bind groups per
pass sharing 16 texture pairs. The cases cover shared attachments, overlapping two-mip views of
mixed attachment/uploaded textures, and a discard between sampling passes. A control uses empty
bind groups and a solid-color shader; it still has the render output attachment.
Each case is a separate benchmark so it can run in its own process. Eight submissions warm up the
workload, then a pixel readback checks the rendered result before timing. Encoding and submission
are timed separately; GPU waits and readback are excluded. Bind-group creation is measured
separately with descriptors and destination capacity prepared in advance.

#### `Renderpass Encoding`

This benchmark measures the performance of recording and submitting a render pass with a large
number of draw calls and resources, emulating an intense, more traditional graphics application.
By default it measures 10k draw calls, with 90k total resources.

Within this benchmark, both single threaded and multi-threaded recording are tested, as well as splitting
the render pass into multiple passes over multiple command buffers.
If available, it also tests a bindless approach, binding all textures at once instead of switching
the bind group for every draw call.

#### `Computepass Encoding`

This benchmark measures the performance of recording and submitting a compute pass with a large
number of dispatches and resources.
By default it measures 10k dispatch calls, with 60k total resources, emulating an unusually complex and sequential compute workload.

Within this benchmark, both single threaded and multi-threaded recording are tested, as well as splitting
the compute pass into multiple passes over multiple command buffers.
If available, it also tests a bindless approach, binding all resources at once instead of switching
the bind group for every draw call.
TODO(https://github.com/gfx-rs/wgpu/issues/5766): The bindless version uses only 1k dispatches with 6k resources since it would be too slow for a reasonable benchmarking time otherwise.

#### `Device::create_buffer`

This benchmark measures the performance of creating large buffers.

#### `Device::create_bind_group`

This benchmark measures the performance of creating large bind groups of 5 to 50,000 resources.

#### `naga::back`, `naga::compact`, `naga::front`, and `naga::valid`

These benchmark measures the performance of naga parsing, validating, and generating shaders.
