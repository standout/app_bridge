# frozen_string_literal: true

# Rake task to run Rust tests
namespace :rust do
  desc "Run Rust tests"
  task :test do
    puts "Running Rust tests..."
    system("cargo test")
    puts "Rust tests completed."
  end
end
