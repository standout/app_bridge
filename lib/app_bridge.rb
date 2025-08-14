# frozen_string_literal: true

require_relative "app_bridge/version"
require_relative "app_bridge/app"

module AppBridge
  class Error < StandardError; end
  class TimeoutError < Error; end
  class TooManyEventsError < Error; end
  class StoreTooLargeError < Error; end
  class ActionResponseTooLargeError < Error; end

  # Represents a trigger event that is recieved from the app.
  class TriggerEvent
    def inspect
      "#<AppBridge::TriggerEvent(id: #{id.inspect}, timestamp: #{timestamp.inspect}, " \
        "serialized_data: #{serialized_data.inspect})>"
    end
  end
end

# Load the extension after the module is defined
require_relative "app_bridge/app_bridge"
