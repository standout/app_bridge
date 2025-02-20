# frozen_string_literal: true

RSpec.describe AppBridge::App do
  let(:components_path) { File.join(File.dirname(__FILE__), "..", "fixtures", "components") }
  let(:component_path) { File.join(components_path, "example.wasm") }

  subject(:app) { AppBridge::App.new(component_path) }

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

  describe "#fetch_events(context)" do
    let(:context) do
      account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
      AppBridge::TriggerContext.new("new-invoice-payment", account, "world")
    end

    it "returns a response with new store" do
      response = app.fetch_events(context)
      expect(response).to be_a(AppBridge::TriggerResponse)
      expect(response.store).to be_a(String)
      expect(response.events).not_to be_empty
      expect(response.events).to all(be_a(AppBridge::TriggerEvent))
    end

    it "includes trigger events" do
      response = app.fetch_events(context)
      expect(response.store).to eq(context.store)
      expect(response.events).to include(
        have_attributes(id: "1", serialized_data: include("delectus aut autem")),
        have_attributes(id: "2", serialized_data: include("quis ut nam facilis et officia qui")),
        have_attributes(id: "3", serialized_data: include("fugiat veniam minus"))
      )
    end
  end
end
