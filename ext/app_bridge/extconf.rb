# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

# # Bestäm Rust-target baserat på plattform
# rust_target = case RbConfig::CONFIG["host_os"]
#               when /darwin/
#                 RbConfig::CONFIG["host_cpu"] == "arm64" ? "aarch64-apple-darwin" : "x86_64-apple-darwin"
#               when /linux/
#                 case RbConfig::CONFIG["host_cpu"]
#                 when "x86_64"
#                   "x86_64-unknown-linux-gnu"
#                 when "aarch64"
#                   "aarch64-unknown-linux-gnu"
#                 else
#                   raise "Unknown Linux architecture: #{RbConfig::CONFIG["host_cpu"]}"
#                 end
#               else
#                 raise "Unsupported OS: #{RbConfig::CONFIG["host_os"]}"
#               end

# Sätt rätt Rust target innan byggning
# ENV["CARGO_BUILD_TARGET"] = rust_target

# Bygg Rust-biblioteket
create_rust_makefile("app_bridge/app_bridge")
