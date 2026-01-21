# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Backward Compatibility" do
  describe "v3 component (built against WIT 3.0.0)" do
    let(:app) do
      AppBridge::App.new(
        "spec/fixtures/components/rust_app_v3.wasm",
        environment_variables: {}
      )
    end

    let(:connection) do
      AppBridge::Connection.new(
        "conn-123",
        "Test Connection",
        '{"api_key": "test-key"}'
      )
    end

    context "#trigger_ids" do
      it "returns an array of trigger ids" do
        expect(app.trigger_ids).to eq(["simple-trigger"])
      end
    end

    context "#action_ids" do
      it "returns an array of action ids" do
        expect(app.action_ids).to eq(["http-get"])
      end
    end

    context "#action_input_schema" do
      it "returns the input schema for http-get action" do
        context = AppBridge::ActionContext.new(
          "http-get",
          connection,
          '{"url": "https://httpbin.org/get"}'
        )
        schema = app.action_input_schema(context)
        expect(JSON.parse(schema)).to include("type" => "object")
      end
    end

    context "#trigger_input_schema" do
      it "returns the input schema for simple-trigger" do
        context = AppBridge::TriggerContext.new(
          "simple-trigger",
          connection,
          "{}",
          "{}"
        )
        schema = app.trigger_input_schema(context)
        expect(JSON.parse(schema)).to include("type" => "object")
      end
    end

    context "#fetch_events" do
      it "returns trigger events from v3 component" do
        context = AppBridge::TriggerContext.new(
          "simple-trigger",
          connection,
          "{}",
          "{}"
        )
        response = app.fetch_events(context)
        expect(response.events.length).to eq(1)
        expect(response.events.first.id).to eq("event-1")

        data = JSON.parse(response.events.first.serialized_data)
        expect(data["message"]).to eq("Hello from v3 connector")
      end
    end
  end

  describe "v4 component (built against WIT 4.0.0)" do
    let(:app) do
      AppBridge::App.new(
        "spec/fixtures/components/rust_app.wasm",
        environment_variables: {}
      )
    end

    let(:connection) do
      AppBridge::Connection.new(
        "conn-123",
        "Test Connection",
        '{"api_key": "test-key"}'
      )
    end

    context "#trigger_ids" do
      it "returns an array of trigger ids" do
        expect(app.trigger_ids).to include("new-todos", "new-posts")
      end
    end

    context "#action_ids" do
      it "returns an array of action ids including file-normalize (v4 feature)" do
        expect(app.action_ids).to include("file-normalize")
      end
    end
  end
end
