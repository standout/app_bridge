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
      expect(response.store).to eq("10")
      expect(response.events).to include(
        have_attributes(id: "1", serialized_data: include("delectus aut autem")),
        have_attributes(id: "2", serialized_data: include("quis ut nam facilis et officia qui")),
        have_attributes(id: "3", serialized_data: include("fugiat veniam minus"))
      )
    end

    context "when polling takes to long" do
      before do
        allow(app).to receive(:polling_timeout).and_return(0.01)
        allow(app).to receive(:_rust_fetch_events).and_wrap_original do |_original_method, *_args|
          sleep(10) # Simulate a long-running operation
        end
      end

      it "raises a Timeout::Error with a message" do
        expect { app.fetch_events(context) }
          .to raise_error(AppBridge::TimeoutError, /Polling exceeded \d+(\.\d+)? seconds/)
      end
    end

    context "when context has more than 100 events" do
      let(:events) do
        101.times.map do |i|
          AppBridge::TriggerEvent.new(
            i.to_s,
            (Time.now.to_f * 1000).to_i,
            JSON.generate({ key: "value" })
          )
        end
      end

      let(:response) do
        AppBridge::TriggerResponse.new("some store", events)
      end

      before do
        allow(app).to receive(:_rust_fetch_events).and_return(response)
      end

      it "raises a TooManyEventsError" do
        expect { app.fetch_events(context) }
          .to raise_error(AppBridge::TooManyEventsError, /Maximum 100 events allowed/)
      end
    end

    describe "limit in store size" do
      let(:events) do
        2.times.map do |i|
          AppBridge::TriggerEvent.new(
            i.to_s,
            (Time.now.to_f * 1000).to_i,
            JSON.generate({ key: "value" })
          )
        end
      end

      let(:response) do
        AppBridge::TriggerResponse.new(store, events)
      end

      before do
        allow(app).to receive(:_rust_fetch_events).and_return(response)
      end

      context "when response store is 64 kilobytes" do
        let(:store) { "a" * 64 * 1024 }

        it "does not raise an error" do
          expect { app.fetch_events(context) }
            .not_to raise_error
        end
      end

      context "when response store is 1 byte over 64 kilobytes" do
        let(:store) { "a" * ((64 * 1024) + 1) }

        it "raises a StoreTooLargeError" do
          expect { app.fetch_events(context) }
            .to raise_error(AppBridge::StoreTooLargeError, /Store size exceeds 64 kB limit/)
        end
      end
    end
  end
end
