# frozen_string_literal: true

require "English"

namespace :fixtures do
  namespace :apps do
    desc "Clean up build artifacts"
    task :clean do
      # In context of the path spec/fixtures/components/rust_app.
      # Execute cargo clean.
      #
      pwd = "spec/fixtures/components/rust_app"
      pid = Process.spawn("cargo clean", chdir: pwd)
      Process.wait(pid)
      raise "Failed to clean build artifacts" unless $CHILD_STATUS.success?

      # Remove the built wasm artifact.
      pid = Process.spawn("rm rust_app.wasm", chdir: "spec/fixtures/components")
      Process.wait(pid)
    end

    desc "Compile the fixture apps"
    task :compile_rust do
      pwd = "spec/fixtures/components/rust_app"
      compile_pid = Process.spawn(
        "CARGO_TARGET_DIR=target cargo clean && CARGO_TARGET_DIR=target cargo build --release --target wasm32-wasip2",
        chdir: pwd
      )
      Process.wait(compile_pid)
      raise "Failed to build artifacts" unless $CHILD_STATUS.success?

      move_pid = Process.spawn("cp #{pwd}/target/wasm32-wasip2/release/rust_app.wasm #{pwd}/../rust_app.wasm")
      Process.wait(move_pid)
      raise "Failed to copy rust_app.wasm" unless $CHILD_STATUS.success?
    end

    task :compile_js do
      pwd = "spec/fixtures/components/js_app"
      pid = Process.spawn("npm run build", chdir: pwd)
      Process.wait(pid)
      raise "Failed to build artifacts" unless $CHILD_STATUS.success?
    end

    desc "Compile the v3 fixture app (for backward compatibility testing)"
    task :compile_rust_v3 do
      pwd = "spec/fixtures/components/rust_app_v3"
      next unless File.exist?(pwd)

      compile_pid = Process.spawn("cargo clean && cargo build --release --target wasm32-wasip2",
                                  chdir: pwd)
      Process.wait(compile_pid)
      raise "Failed to build v3 artifacts" unless $CHILD_STATUS.success?

      move_pid = Process.spawn("mv #{pwd}/target/wasm32-wasip2/release/rust_app_v3.wasm #{pwd}/../rust_app_v3.wasm")
      Process.wait(move_pid)
    end

    desc "Compile the v5 fixture app (connections interface)"
    task :compile_rust_v5 do
      pwd = "spec/fixtures/components/rust_app_v5"
      next unless File.exist?(pwd)

      compile_pid = Process.spawn(
        "CARGO_TARGET_DIR=target cargo clean && CARGO_TARGET_DIR=target cargo build --release --target wasm32-wasip2",
        chdir: pwd
      )
      Process.wait(compile_pid)
      raise "Failed to build v5 artifacts" unless $CHILD_STATUS.success?

      move_pid = Process.spawn("cp #{pwd}/target/wasm32-wasip2/release/rust_app_v5.wasm #{pwd}/../rust_app_v5.wasm")
      Process.wait(move_pid)
      raise "Failed to copy rust_app_v5.wasm" unless $CHILD_STATUS.success?
    end
  end
end

desc "Build all fixtures"
task fixtures: %i[fixtures:apps:clean fixtures:apps:compile_rust fixtures:apps:compile_js fixtures:apps:compile_rust_v3
                  fixtures:apps:compile_rust_v5]
