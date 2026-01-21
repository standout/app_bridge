# frozen_string_literal: true

require "json"
require "timeout"

module AppBridge
  # An app that can be used to fetch events and execute actions.
  class App
    def initialize(component_path, environment_variables: {})
      @component_path = component_path
      @environment_variables = environment_variables
      _rust_initialize(component_path, environment_variables)
    rescue StandardError
      raise InternalError, "Incompatible WASM file version"
    end

    def fetch_events(context)
      response = request_events_with_timeout(context)

      validate_number_of_events!(response.events)
      validate_store_size!(response.store)

      response
    end

    def execute_action(context)
      response = request_action_with_timeout(context)

      # Process files: find file-output fields in schema and replace with blob IDs
      processed_output = process_files(response.serialized_output, context)

      validate_action_response_size!(processed_output)

      # Return new response with processed output
      response.with_output(processed_output)
    end

    def timeout_seconds
      30 # seconds
    end

    private

    def validate_number_of_events!(events)
      return if events.size <= 100

      raise TooManyEventsError, "Maximum 100 events allowed"
    end

    def validate_store_size!(store)
      return if store.size <= 64 * 1024

      raise StoreTooLargeError, "Store size exceeds 64 kB limit"
    end

    def validate_action_response_size!(serialized_output)
      return if serialized_output.size <= 64 * 1024

      raise ActionResponseTooLargeError, "Action response size exceeds 64 kB limit"
    end

    def request_action_with_timeout(context)
      Timeout.timeout(timeout_seconds, TimeoutError, "Action exceeded #{timeout_seconds} seconds") do
        _rust_execute_action(context)
      end
    end

    def request_events_with_timeout(context)
      Timeout.timeout(timeout_seconds, TimeoutError, "Polling exceeded #{timeout_seconds} seconds") do
        _rust_fetch_events(context)
      end
    end

    # Process files in the response based on output schema
    def process_files(serialized_output, context)
      data = JSON.parse(serialized_output)
      schema = parse_output_schema(context)
      JSON.generate(FileProcessor.call(data, schema))
    rescue JSON::ParserError
      # Schema parsing failed, return original output
      serialized_output
    end

    def parse_output_schema(context)
      JSON.parse(action_output_schema(context))
    end
  end
end
