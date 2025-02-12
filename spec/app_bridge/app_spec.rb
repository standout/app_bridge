# frozen_string_literal: true

RSpec.describe AppBridge::App do
  let(:components_path) { File.join(File.dirname(__FILE__), "..", "fixtures", "components") }
  let(:component_path) { File.join(components_path, "example.wasm") }

  subject(:app) { AppBridge::App.new(component_path) }

  it "initializes within 10 milliseconds" do
    expect do
      AppBridge::App.new(component_path)
    end.to perform_under(10).ms.sample(10).times
  end

  describe "#triggers" do
    it "returns an array of trigger ids" do
      expect(app.triggers).to be_a(Array)
        .and include("new-invoice-payment", "new-invoice")
    end

    it "performs in less than 10 microseconds" do
      # Load the app, whe are intressted in the performance of the triggers
      # method only. Not the time to load the app.
      app

      expect { app.triggers }.to perform_under(10).us.sample(10).times
    end
  end
end
