# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"

RSpec::Core::RakeTask.new(:spec)

require "rubocop/rake_task"

RuboCop::RakeTask.new

require "rb_sys/extensiontask"

task build: :compile

GEMSPEC = Gem::Specification.load("app_bridge.gemspec")

RbSys::ExtensionTask.new("app_bridge", GEMSPEC) do |ext|
  ext.lib_dir = "lib/app_bridge"
end

# Load all project specific rake tasks
Dir.glob(File.expand_path("tasks/**/*.rake", __dir__)).each { |file| load file }

task default: %i[fixtures compile rust:test spec rubocop]
