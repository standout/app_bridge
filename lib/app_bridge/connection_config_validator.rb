# frozen_string_literal: true

require "json"
require "json_schemer"

module AppBridge
  # Validates connection-config JSON against the bundled JSON Schema.
  class ConnectionConfigValidator
    SCHEMA_PATH = File.expand_path(
      "../../ext/app_bridge/docs/connection-config-schema.json",
      __dir__
    )

    class << self
      def validate!(data)
        new.validate!(data)
      end

      def validate(data)
        new.validate(data)
      end

      def schema
        @schema ||= JSON.parse(File.read(SCHEMA_PATH))
      end

      def schemer
        @schemer ||= JSONSchemer.schema(schema)
      end
    end

    def validate!(data)
      errors = validate(data)
      return if errors.empty?

      raise AppBridge::ConnectionConfigError, format_errors(errors)
    end

    def validate(data)
      parsed = parse_data(data)
      self.class.schemer.validate(parsed).to_a
    end

    private

    def parse_data(data)
      case data
      when Hash
        data
      when String
        JSON.parse(data)
      else
        raise AppBridge::ConnectionConfigError, "connection-config must be a JSON object"
      end
    rescue JSON::ParserError => e
      raise AppBridge::ConnectionConfigError, "Invalid connection-config JSON: #{e.message}"
    end

    def format_errors(errors)
      errors.map { |error| format_error(error) }.join("\n")
    end

    def format_error(error)
      pointer = error["data_pointer"]
      pointer = "/" if pointer.nil? || pointer.empty?
      "#{pointer}: #{error_detail(error)}"
    end

    def error_detail(error)
      message = error["error"]
      return message if message.is_a?(String) && !message.empty?

      details = error["details"]
      return schemer_details_message(details) if details.is_a?(Hash)

      details || error["type"] || "validation failed"
    end

    def schemer_details_message(details)
      if details["missing_keys"]
        missing = Array(details["missing_keys"]).join(", ")
        return "missing required properties: #{missing}"
      end

      details.map { |key, value| "#{key}: #{value}" }.join(", ")
    end
  end
end
