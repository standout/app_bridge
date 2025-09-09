# frozen_string_literal: true

require "timeout"

module AppBridge
  # An app that can be used to fetch events and execute actions.
  class App
    def initialize(component_path)
      @component_path = component_path
      initialize_bridge
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

      validate_action_response_size!(response.serialized_output)

      response
    end

    def action_input_schema(action_id, context)
      _rust_action_input_schema(action_id, context)
    end

    def action_output_schema(action_id, context)
      _rust_action_output_schema(action_id, context)
    end

    def trigger_input_schema(trigger_id, context)
      _rust_trigger_input_schema(trigger_id, context)
    end

    def trigger_output_schema(trigger_id, context)
      _rust_trigger_output_schema(trigger_id, context)
    end

    def timeout_seconds
      30 # seconds
    end

    private

    def initialize_bridge
      _rust_initialize(@component_path)
    end

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
  end
end
