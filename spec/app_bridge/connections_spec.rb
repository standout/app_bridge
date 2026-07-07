# frozen_string_literal: true

RSpec.describe "v5 connections API" do
  let(:component_path) { File.join(__dir__, "..", "fixtures", "components", "rust_app_v5.wasm") }
  let(:app) { AppBridge::App.new(component_path, environment_variables: {}) }

  describe "v5 component (built against WIT 5.0.0)" do
    it "reports wit version 5.0.0" do
      expect(app.wit_version).to eq("5.0.0")
    end

    it "reports connections as supported" do
      expect(app.connections_supported?).to be(true)
    end

    it "returns connection config JSON with platform validation" do
      config = JSON.parse(app.connection_config)

      expect(config["default_strategy"]).to eq("default")
      expect(config["strategies"].first["type"]).to eq("oauth2")
      expect(config.dig("runtime", "base_url")).to eq(
        "template" => "https://{{tenant}}.api.example.com"
      )
      expect(config.dig("connection_schema", "properties", "tenant", "type")).to eq("string")
      expect(config.dig("post_connect", "request", "path")).to eq("/health")
    end

    it "still supports actions and triggers" do
      expect(app.action_ids).to eq(["http-get"])
      expect(app.trigger_ids).to eq(["simple-trigger"])
    end
  end

  describe "v4 component without connections interface" do
    let(:v4_path) { File.join(__dir__, "..", "fixtures", "components", "rust_app.wasm") }
    let(:app) { AppBridge::App.new(v4_path, environment_variables: {}) }

    it "reports connections as not supported" do
      expect(app.connections_supported?).to be(false)
    end

    it "raises when calling connection_config" do
      expect { app.connection_config }.to raise_error(AppBridge::UnsupportedError)
    end
  end
end
