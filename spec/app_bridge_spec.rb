# frozen_string_literal: true

RSpec.describe AppBridge do
  let(:components_path) { File.join(File.dirname(__FILE__), "fixtures", "components") }

  it "has a version number" do
    expect(AppBridge::VERSION).not_to be nil
  end

  describe "AppBridge::App" do
    let(:component_path) { File.join(components_path, "example_app.wasm") }

    subject(:app) { AppBridge::App.new(component_path) }

    it "should be a class" do
      expect(AppBridge::App).to be_a(Class)
    end

    describe "#get_triggers" do
      it "should return an array of trigger ids" do
        expect(app.get_triggers).to be_a(Array)
          .and include("new-invoice-payment", "new-invoice")
      end
    end
  end
end
