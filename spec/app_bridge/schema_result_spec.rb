# frozen_string_literal: true

RSpec.describe AppBridge::App, "#schema_result" do
  let(:component) { "rust_app_v6" }
  let(:app) { described_class.new(File.expand_path("../fixtures/components/#{component}.wasm", __dir__)) }
  let(:account) { AppBridge::Connection.new("1", "Tenant", "{}") }

  it "distinguishes complete, partial, and connection-required independently per direction" do
    expect(app.schema_result(:action, :input, key: "example")).to eq(["partial", "{}"])
    expect(app.schema_result(:action, :output, key: "example")).to eq(["complete", "{}"])
    expect(app.schema_result(:trigger, :input, key: "example")).to eq(["connection_required", nil])
    expect(app.schema_result(:trigger, :input, key: "example", connection: account)).to eq(["complete", "{}"])

    [nil, account].each do |connection|
      status, schema = app.schema_result(:trigger, :output, key: "example", connection: connection,
                                                            input: '{"page":1}', store: "previous")
      expect(status).to eq("complete")
      expect(JSON.parse(schema)).to eq("title" => "previous", "description" => '{"page":1}')
    end

    expect(app.schema_result(:action, :input, key: "example",
                                              connection: account)).to eq(["complete", '{"title":"Tenant"}'])
    expect(app.schema_result(:action, :output, key: "example", connection: account)).to eq(["complete", "{}"])
    expect(app.action_input_schema(AppBridge::ActionContext.new("example", account, "{}"))).to eq('{"title":"Tenant"}')
    expect { app.schema_result(:action, :input, key: "missing") }.to raise_error(AppBridge::MisconfiguredError)
  end

  context "with a v5 component" do
    let(:component) { "rust_app_v5" }

    it "uses original exports with an account and skips them without one" do
      { action: "http-get", trigger: "simple-trigger" }.each do |kind, key|
        %i[input output].each do |direction|
          expect(app.schema_result(kind, direction, key: key)).to eq(["connection_required", nil])
          status, schema = app.schema_result(kind, direction, key: key, connection: account)
          expect(status).to eq("complete")
          expect(JSON.parse(schema)).to include("type" => "object")
        end
      end
      expect(app.fetch_events(AppBridge::TriggerContext.new("simple-trigger", account, "", "{}")).events.size).to eq(1)
    end
  end
end
