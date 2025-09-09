# frozen_string_literal: true

RSpec.describe AppBridge::App do
  let(:components_path) { File.join(File.dirname(__FILE__), "..", "fixtures", "components") }

  shared_examples "example standout app" do |wasm_file|
    let(:component_path) { File.join(components_path, wasm_file) }
    subject(:app) { AppBridge::App.new(component_path) }

    describe "#trigger_ids" do
      it "returns an array of trigger ids" do
        expect(app.trigger_ids).to be_a(Array)
          .and include("new-posts",
                       "new-todos",
                       "new-photos",
                       "new-comments",
                       "new-users",
                       "new-albums")
      end

      it "performs in less than 300 microseconds" do
        # Load the app, we are interested in the performance of the trigger_ids
        # method only, not the time to load the app.
        app

        expect { app.trigger_ids }.to perform_under(300).us.sample(10).times
      end
    end

    describe "#fetch_events(context)" do
      let(:context) do
        account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
        AppBridge::TriggerContext.new("new-todos", account, "world", "{}")
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

      context "when polling takes too long" do
        before do
          allow(app).to receive(:timeout_seconds).and_return(0.01)
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

    describe "#action_ids" do
      it "returns an array of action ids" do
        expect(app.action_ids).to be_a(Array)
          .and include("http-get", "http-post", "complex-input")

        expect(app.action_ids).not_to include("new-posts")
      end

      it "performs in less than 300 microseconds" do
        # Load the app, we are interested in the performance of the action_ids
        # method only, not the time to load the app.
        app

        expect { app.action_ids }.to perform_under(300).us.sample(10).times
      end
    end

    describe "#trigger_input_schema" do
      let(:base_account) { AppBridge::Account.new("1", "Base Account", JSON.generate({})) }
      let(:custom_account) { AppBridge::Account.new("2", "Custom Account", JSON.generate({ custom: true })) }

      it "returns the input schema for new-todos trigger with base account" do
        context = AppBridge::TriggerContext.new("new-todos", base_account, "", "{}")
        schema = app.trigger_input_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["include_extra_data"]["type"]).to eq("boolean")
        expect(parsed_schema["properties"]["include_extra_data"]["description"]).to eq(
          "Whether to include additional data in the response"
        )
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("include_custom_data")
      end

      it "returns the input schema for new-posts trigger with custom account" do
        context = AppBridge::TriggerContext.new("new-posts", custom_account, "", "{}")
        schema = app.trigger_input_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["include_extra_data"]["type"]).to eq("boolean")
        # Custom account should have additional custom field
        expect(parsed_schema["properties"]["include_custom_data"]["type"]).to eq("boolean")
        expect(parsed_schema["properties"]["include_custom_data"]["description"])
          .to eq("Whether to include custom data for premium accounts")
      end

      it "raises an error for invalid trigger ID" do
        context = AppBridge::TriggerContext.new("invalid-trigger", base_account, "", "{}")
        expect { app.trigger_input_schema(context) }.to raise_error(AppBridge::Error)
      end
    end

    describe "#trigger_output_schema" do
      let(:base_account) { AppBridge::Account.new("1", "Base Account", JSON.generate({})) }
      let(:custom_account) { AppBridge::Account.new("2", "Custom Account", JSON.generate({ custom: true })) }

      it "returns the output schema for new-todos trigger with base account" do
        context = AppBridge::TriggerContext.new("new-todos", base_account, "", "{}")
        schema = app.trigger_output_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["events"]["type"]).to eq("array")
        expect(parsed_schema["properties"]["store"]["type"]).to eq("string")
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("custom_metadata")
      end

      it "returns the output schema for new-posts trigger with custom account" do
        context = AppBridge::TriggerContext.new("new-posts", custom_account, "", "{}")
        schema = app.trigger_output_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["events"]["type"]).to eq("array")
        # Custom account should have additional custom metadata field
        expect(parsed_schema["properties"]["custom_metadata"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["custom_metadata"]["description"]).to eq(
          "Additional metadata for premium accounts"
        )
      end

      it "raises an error for invalid trigger ID" do
        context = AppBridge::TriggerContext.new("invalid-trigger", base_account, "", "{}")
        expect { app.trigger_output_schema(context) }.to raise_error(AppBridge::Error)
      end
    end

    describe "#action_input_schema" do
      let(:base_account) { AppBridge::Account.new("1", "Base Account", JSON.generate({})) }
      let(:custom_account) { AppBridge::Account.new("2", "Custom Account", JSON.generate({ custom: true })) }

      it "returns the input schema for http-get action with base account" do
        context = AppBridge::ActionContext.new("http-get", base_account, "{}")
        schema = app.action_input_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["url"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["url"]["format"]).to eq("uri")
        expect(parsed_schema["required"]).to include("url")
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("custom_headers")
      end

      it "returns the input schema for http-post action with custom account" do
        context = AppBridge::ActionContext.new("http-post", custom_account, "{}")
        schema = app.action_input_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["url"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["url"]["format"]).to eq("uri")
        expect(parsed_schema["properties"]["body"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["body"]["format"]).to eq("code")
        expect(parsed_schema["required"]).to include("url")
        # Custom account should have additional custom headers field
        expect(parsed_schema["properties"]["custom_headers"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["custom_headers"]["description"]).to eq(
          "Custom headers for premium accounts"
        )
      end

      it "raises an error for invalid action ID" do
        context = AppBridge::ActionContext.new("invalid-action", base_account, "{}")
        expect { app.action_input_schema(context) }.to raise_error(AppBridge::Error)
      end

      it "returns the input schema for complex-input action with base account" do
        context = AppBridge::ActionContext.new("complex-input", base_account, "{}")
        schema = app.action_input_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["customer"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["customer"]["properties"]["status"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["customer"]["properties"]["status"]["enum"]).to eq(%w[active inactive
                                                                                                 pending])
        expect(parsed_schema["properties"]["customer"]["properties"]["orders"]["type"]).to eq("array")
        # Check nested properties exist and have correct types
        expect(parsed_schema.dig("properties", "customer", "properties", "orders", "items", "properties", "items",
                                 "items", "properties", "sku", "type")).to eq("string")
        expect(parsed_schema.dig("properties", "customer", "properties", "orders", "items", "properties", "items",
                                 "items", "properties", "quantity", "type")).to eq("integer")
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("custom_options")
      end
    end

    describe "#action_output_schema" do
      let(:base_account) { AppBridge::Account.new("1", "Base Account", JSON.generate({})) }
      let(:custom_account) { AppBridge::Account.new("2", "Custom Account", JSON.generate({ custom: true })) }

      it "returns the output schema for http-get action with base account" do
        context = AppBridge::ActionContext.new("http-get", base_account, "{}")
        schema = app.action_output_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["url"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["response"]["type"]).to eq("object")
        expect(parsed_schema["required"]).to include("url", "response")
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("custom_metadata")
      end

      it "returns the output schema for http-post action with custom account" do
        context = AppBridge::ActionContext.new("http-post", custom_account, "{}")
        schema = app.action_output_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["url"]["type"]).to eq("string")
        expect(parsed_schema["properties"]["body"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["response"]["type"]).to eq("object")
        expect(parsed_schema["required"]).to include("url", "body", "response")
        # Custom account should have additional custom metadata field
        expect(parsed_schema["properties"]["custom_metadata"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["custom_metadata"]["description"]).to eq(
          "Additional metadata for premium accounts"
        )
      end

      it "raises an error for invalid action ID" do
        context = AppBridge::ActionContext.new("invalid-action", base_account, "{}")
        expect { app.action_output_schema(context) }.to raise_error(AppBridge::Error)
      end

      it "returns the output schema for complex-input action with base account" do
        context = AppBridge::ActionContext.new("complex-input", base_account, "{}")
        schema = app.action_output_schema(context)
        expect(schema).to be_a(String)

        parsed_schema = JSON.parse(schema)
        expect(parsed_schema["$schema"]).to eq("https://json-schema.org/draft/2020-12/schema")
        expect(parsed_schema["type"]).to eq("object")
        expect(parsed_schema["properties"]["customer"]["type"]).to eq("object")
        expect(parsed_schema["properties"]["metadata"]["type"]).to eq("object")
        expect(parsed_schema["required"]).to include("customer")
        # Base schema should not have custom fields
        expect(parsed_schema["properties"]).not_to have_key("custom_analytics")
      end
    end

    describe "#execute_action(context)" do
      let(:context) do
        account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
        AppBridge::ActionContext.new("http-get", account, JSON.generate({ url: "https://httpbin.org/get" }))
      end

      it "returns a response with output" do
        response = app.execute_action(context)
        expect(response).to be_a(AppBridge::ActionResponse)
        expect(response.serialized_output).to be_a(String)
        expect(response.serialized_output).to include("url")
      end

      context "with invalid action ID" do
        let(:context) do
          account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
          AppBridge::ActionContext.new("invalid-action", account, "{}")
        end

        it "raises an error" do
          expect { app.execute_action(context) }.to raise_error(AppBridge::Error)
        end
      end

      context "with http-post action" do
        let(:context) do
          account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
          AppBridge::ActionContext.new("http-post", account, JSON.generate({
                                                                             url: "https://httpbin.org/post",
                                                                             body: JSON.generate({ test: "data" })
                                                                           }))
        end

        it "returns a response with output" do
          response = app.execute_action(context)
          expect(response).to be_a(AppBridge::ActionResponse)
          expect(response.serialized_output).to be_a(String)
          expect(response.serialized_output).to include("test")
        end
      end

      context "with complex-input action" do
        let(:context) do
          account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
          AppBridge::ActionContext.new("complex-input", account, JSON.generate({
                                                                                 customer: {
                                                                                   status: "active",
                                                                                   orders: [
                                                                                     {
                                                                                       items: [
                                                                                         { sku: "ABC123", quantity: 2 },
                                                                                         { sku: "DEF456", quantity: 1 }
                                                                                       ]
                                                                                     }
                                                                                   ]
                                                                                 }
                                                                               }))
        end

        it "returns a response with processed customer data" do
          response = app.execute_action(context)
          expect(response).to be_a(AppBridge::ActionResponse)
          expect(response.serialized_output).to be_a(String)

          output = JSON.parse(response.serialized_output)
          expect(output["customer"]["status"]).to eq("active")
          expect(output["customer"]["orders"][0]["items"]).to have_attributes(length: 2)
        end
      end

      context "when action response is too large" do
        let(:context) do
          account = AppBridge::Account.new("1", "John Doe", JSON.generate({ username: "john.doe", password: "foobar" }))
          AppBridge::ActionContext.new("http-get", account, JSON.generate({ url: "https://httpbin.org/get" }))
        end

        it "raises ActionResponseTooLargeError when response exceeds 64 kB" do
          # Mock the action to return a response that's too large
          allow(app).to receive(:_rust_execute_action).and_return(
            AppBridge::ActionResponse.new("a" * ((64 * 1024) + 1))
          )

          expect { app.execute_action(context) }
            .to raise_error(AppBridge::ActionResponseTooLargeError, /Action response size exceeds 64 kB limit/)
        end

        it "does not raise error when response is exactly 64 kB" do
          # Mock the action to return a response that's exactly 64 kB
          allow(app).to receive(:_rust_execute_action).and_return(
            AppBridge::ActionResponse.new("a" * (64 * 1024))
          )

          expect { app.execute_action(context) }
            .not_to raise_error
        end
      end
    end
  end

  describe "app built with javascript" do
    it_behaves_like "example standout app", "js_app.wasm"
  end

  describe "app built with rust" do
    it_behaves_like "example standout app", "rust_app.wasm"
  end
end
