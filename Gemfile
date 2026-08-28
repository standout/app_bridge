# frozen_string_literal: true

source "https://rubygems.org"

# Specify your gem's dependencies in app_bridge.gemspec
gemspec

gem "rake", "~> 13.4"

gem "rake-compiler"

unless ENV["RUBY_TARGET"]
  group :development, :test do
    gem "rspec", "~> 3.0"
    gem "rspec-benchmark"

    gem "rubocop", "~> 1.81"

    gem "json"
  end
end
