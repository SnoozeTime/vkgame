thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: DeviceLost', src/libcore/result.rs:1009:5
note: Run with `RUST_BACKTRACE=1` for a backtrace.
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: DeviceLostError', src/libcore/result.rs:1009:5
stack backtrace:
   0:     0x55b48c212f63 - std::sys::unix::backtrace::tracing::imp::unwind_backtrace::h00d1e05a61bd440b
                               at src/libstd/sys/unix/backtrace/tracing/gcc_s.rs:49
   1:     0x55b48c20d418 - std::sys_common::backtrace::_print::hc0d53aca8da62f75
                               at src/libstd/sys_common/backtrace.rs:71
   2:     0x55b48c211d02 - std::panicking::default_hook::{{closure}}::h46d30bcc4bfff149
                               at src/libstd/sys_common/backtrace.rs:59
                               at src/libstd/panicking.rs:211
   3:     0x55b48c211a6d - std::panicking::default_hook::h017696c2a8b7b16f
                               at src/libstd/panicking.rs:227
   4:     0x55b48c212410 - std::panicking::rust_panic_with_hook::h8cbdfe43764887be
                               at src/libstd/panicking.rs:491
   5:     0x55b48c211f91 - std::panicking::continue_panic_fmt::h3d3c5a833c00a5e1
                               at src/libstd/panicking.rs:398
   6:     0x55b48c211e75 - rust_begin_unwind
                               at src/libstd/panicking.rs:325
   7:     0x55b48c22833c - core::panicking::panic_fmt::h4d67173bc68f6d5a
                               at src/libcore/panicking.rs:95
   8:     0x55b48b0d71eb - core::result::unwrap_failed::h0094b4647667e128
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/macros.rs:26
   9:     0x55b48b0c28b1 - <core::result::Result<T, E>>::unwrap::h0f0b80b8040f2bff
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/result.rs:808
  10:     0x55b48afbbf5a - <vulkano::swapchain::swapchain::SwapchainAcquireFuture<W> as core::ops::drop::Drop>::drop::h881fe6c5e1544f99
                               at /home/benoit/.cargo/registry/src/github.com-1ecc6299db9ec823/vulkano-0.11.1/src/swapchain/swapchain.rs:859
  11:     0x55b48ae4dd74 - core::ptr::real_drop_in_place::h94d6062c0dbd8d3a
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  12:     0x55b48ae4dd35 - core::ptr::real_drop_in_place::h940c9a845d049b0b
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  13:     0x55b48ae50336 - core::ptr::real_drop_in_place::hab14308c23470418
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  14:     0x55b48ae5047c - core::ptr::real_drop_in_place::hab583398800d1a39
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  15:     0x55b48ae50336 - core::ptr::real_drop_in_place::hab14308c23470418
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  16:     0x55b48ae5297d - core::ptr::real_drop_in_place::hc1fa6e1ea8912598
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libcore/ptr.rs:204
  17:     0x55b48b0a795e - <vulkano::sync::future::fence_signal::FenceSignalFuture<F>>::flush_impl::h2b37ad086debb2f9
                               at /home/benoit/.cargo/registry/src/github.com-1ecc6299db9ec823/vulkano-0.11.1/src/sync/future/fence_signal.rs:296
  18:     0x55b48b0a6d85 - <vulkano::sync::future::fence_signal::FenceSignalFuture<F> as vulkano::sync::future::GpuFuture>::flush::h55e9f25467b694ca
                               at /home/benoit/.cargo/registry/src/github.com-1ecc6299db9ec823/vulkano-0.11.1/src/sync/future/fence_signal.rs:347
  19:     0x55b48afc1417 - vulkano::sync::future::GpuFuture::then_signal_fence_and_flush::h6b0b6eb70530c3d5
                               at /home/benoit/.cargo/registry/src/github.com-1ecc6299db9ec823/vulkano-0.11.1/src/sync/future/mod.rs:228
  20:     0x55b48b55c5e3 - twgraph::renderer::Renderer::render::h8448d4c070f7ac00
                               at src/renderer/mod.rs:398
  21:     0x55b48b34e97b - twgraph::ecs::systems::RenderingSystem::render::h5fea4cc3b0e91d5a
                               at src/ecs/systems.rs:143
  22:     0x55b48ac50b01 - editor::main::hc5d54c0de2c7eb68
                               at src/bin/editor.rs:74
  23:     0x55b48ac532df - std::rt::lang_start::{{closure}}::h77b7a11c4cbf8d84
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libstd/rt.rs:74
  24:     0x55b48c211e12 - std::panicking::try::do_call::h69790245ac2d03fe
                               at src/libstd/rt.rs:59
                               at src/libstd/panicking.rs:310
  25:     0x55b48c222099 - __rust_maybe_catch_panic
                               at src/libpanic_unwind/lib.rs:102
  26:     0x55b48c2128a3 - std::rt::lang_start_internal::h540c897fe52ba9c5
                               at src/libstd/panicking.rs:289
                               at src/libstd/panic.rs:398
                               at src/libstd/rt.rs:58
  27:     0x55b48ac532b8 - std::rt::lang_start::hcecf3b4d24dcaa6f
                               at /rustc/9fda7c2237db910e41d6a712e9a2139b352e558b/src/libstd/rt.rs:74
  28:     0x55b48ac512c9 - main
  29:     0x7f1c0d66682f - __libc_start_main
  30:     0x55b48ac48bd8 - _start
  31:                0x0 - <unknown>
thread panicked while panicking. aborting.
[1]    10617 illegal hardware instruction (core dumped)  cargo run --bin editor -- -s assets/levels/shadow.json

