# frozen_string_literal: true

require_relative "app_bridge/version"
require_relative "app_bridge/app"
require_relative "app_bridge/file_processor"

# Communication layer for Standout integration apps using WebAssembly components.
module AppBridge
  class Error < StandardError; end
  class TimeoutError < Error; end
  class TooManyEventsError < Error; end
  class StoreTooLargeError < Error; end
  class ActionResponseTooLargeError < Error; end
  class InternalError < Error; end

  class << self
    # Configurable file uploader callback.
    # The platform should set this to handle file uploads and return an ID.
    #
    # @example Configure with ActiveStorage
    #   AppBridge.file_uploader = ->(file_data) {
    #     content = Base64.decode64(file_data['base64'])
    #     blob = ActiveStorage::Blob.create_and_upload!(
    #       io: StringIO.new(content),
    #       filename: file_data['filename'],
    #       content_type: file_data['content_type'] || 'application/octet-stream'
    #     )
    #     blob.signed_id  # Returns just the ID
    #   }
    attr_accessor :file_uploader
  end

  # Default no-op uploader (returns nil - no file storage configured)
  # rubocop:disable Style/NilLambda
  self.file_uploader = ->(_file_data) { nil }
  # rubocop:enable Style/NilLambda

  # Represents a trigger event that is recieved from the app.
  class TriggerEvent
    def inspect
      "#<AppBridge::TriggerEvent(id: #{id.inspect}, timestamp: #{timestamp.inspect}, " \
        "serialized_data: #{serialized_data.inspect})>"
    end
  end
end

# Load using the Ruby version as the directory name
begin
  require "app_bridge/#{RUBY_VERSION.split(".").first(2).join(".")}/app_bridge"
rescue LoadError
  require "app_bridge/app_bridge"
end
