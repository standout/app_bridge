# frozen_string_literal: true

require "json"

module AppBridge
  # Processes file data from WASM component responses.
  #
  # Uses the output schema to find fields with format: "file-output" and
  # replaces the file data (base64, content_type, filename) with blob IDs
  # via the configured file_uploader.
  #
  # The WASM component should use file.read to normalize any input format
  # (URL, data URI, raw base64) into a consistent hash structure before output.
  class FileProcessor
    FILE_OUTPUT_FORMAT = "file-output"

    # Process file data in a response hash.
    #
    # @param data [Hash] The response data from a WASM component
    # @param schema [Hash] The JSON schema for the response data
    # @return [Hash] The processed data with file IDs instead of file data
    def self.call(data, schema)
      new(data, schema).call
    end

    # @param data [Hash] The response data from a WASM component
    # @param schema [Hash] The JSON schema for the response data
    def initialize(data, schema)
      @data = deep_dup(data)
      @schema = schema
    end

    # Processes the data, uploading files and replacing file data with IDs.
    #
    # @return [Hash] The processed data with file IDs
    def call
      process_node(@data, @schema)
      @data
    end

    private

    def process_node(data_node, schema_node)
      return unless data_node.is_a?(Hash) && schema_node.is_a?(Hash)

      properties = schema_node["properties"]
      return unless properties.is_a?(Hash)

      properties.each do |key, sub_schema|
        process_property(data_node, key, sub_schema)
      end
    end

    def process_property(data_node, key, sub_schema)
      data_value, actual_key = find_data_value(data_node, key)
      return if data_value.nil?

      if sub_schema["format"] == FILE_OUTPUT_FORMAT
        process_file_field(data_node, actual_key, data_value)
      elsif array_schema?(sub_schema)
        process_array_field(data_value, sub_schema)
      elsif data_value.is_a?(Hash)
        process_node(data_value, sub_schema)
      end
    end

    def find_data_value(data_node, key)
      str_key = key.to_s
      return [nil, nil] unless data_node.key?(str_key)

      [data_node[str_key], str_key]
    end

    def process_file_field(data_node, actual_key, data_value)
      return unless file_data?(data_value)

      blob_id = upload_file(data_value)
      data_node[actual_key] = blob_id
    end

    def array_schema?(sub_schema)
      sub_schema["type"] == "array" && sub_schema["items"].is_a?(Hash)
    end

    def process_array_field(data_value, sub_schema)
      return unless data_value.is_a?(Array)

      items_schema = sub_schema["items"]
      data_value.each_with_index do |item, index|
        if items_schema["format"] == FILE_OUTPUT_FORMAT && file_data?(item)
          data_value[index] = upload_file(item)
        else
          process_node(item, items_schema)
        end
      end
    end

    def file_data?(value)
      return false unless value.is_a?(Hash)

      present?(value["base64"]) && present?(value["filename"])
    end

    def upload_file(file_data)
      result = AppBridge.file_uploader.call(file_data)
      # If uploader returns nil (no uploader configured), leave data unchanged
      result.nil? ? file_data : result
    rescue StandardError => e
      raise AppBridge::InternalError, "Failed to upload '#{file_data["filename"]}': #{e.message}"
    end

    def present?(value)
      !value.nil? && !value.to_s.strip.empty?
    end

    # Deep duplicates the object and normalizes all hash keys to strings
    def deep_dup(obj)
      case obj
      when Hash then obj.transform_keys(&:to_s).transform_values { |v| deep_dup(v) }
      when Array then obj.map { |v| deep_dup(v) }
      else safe_dup(obj)
      end
    end

    def safe_dup(obj)
      obj.dup
    rescue TypeError
      obj
    end
  end
end
