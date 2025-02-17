# frozen_string_literal: true

require "English"

namespace :fixtures do
  namespace :apps do
    desc "Clean up build artifacts"
    task :clean do
      # In context of the path spec/fixtures/components/example.
      # Execute cargo clean.
      #
      pwd = "spec/fixtures/components/example"
      pid = Process.spawn("cargo clean", chdir: pwd)
      Process.wait(pid)
      raise "Failed to clean build artifacts" unless $CHILD_STATUS.success?

      # Remove the built wasm artifact.
      pid = Process.spawn("rm example.wasm", chdir: "spec/fixtures/components")
      Process.wait(pid)
    end

    desc "Compile the fixture apps"
    task :compile do
      pwd = "spec/fixtures/components/example"
      compile_pid = Process.spawn("cargo clean && cargo build --release --target wasm32-wasip2",
                                  chdir: pwd)
      Process.wait(compile_pid)
      raise "Failed to build artifacts" unless $CHILD_STATUS.success?

      move_pid = Process.spawn("mv #{pwd}/target/wasm32-wasip2/release/example.wasm #{pwd}/../example.wasm")
      Process.wait(move_pid)
    end
  end
end

desc "Build all fixtures"
task fixtures: %i[fixtures:apps:clean fixtures:apps:compile]
